//! A no_std implementation of https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/codec/framed_impl.rs

use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use core::borrow::{Borrow, BorrowMut};
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::{ready, Sink, Stream};
use pin_project_lite::pin_project;
use super::super::io::{AsyncWrite, AsyncRead, io_slice::IoSlice};
use super::codec::{Encoder, Decoder};
use super::errors::IoError;

#[cfg(feature = "std")]
use tokio::io::ReadBuf;
#[cfg(not(feature = "std"))]
use crate::core::io::ReadBuf;
use crate::Err;

const INITIAL_CAPACITY: usize = 8 * 1024;

pin_project! {
    #[derive(Debug)]
    pub(crate) struct FramedImpl<T, C, State>
    {
        #[pin]
        pub(crate) inner: T,
        pub(crate) state: State,
        pub(crate) codec: C,
    }
}

#[derive(Debug)]
pub(crate) struct ReadFrame {
    pub(crate) eof: bool,
    pub(crate) is_readable: bool,
    pub(crate) buffer: BytesMut,
    pub(crate) has_errored: bool,
}

pub(crate) struct WriteFrame {
    pub(crate) buffer: BytesMut,
    pub(crate) backpressure_boundary: usize,
}

#[derive(Default)]
pub(crate) struct RWFrames {
    pub(crate) read: ReadFrame,
    pub(crate) write: WriteFrame,
}

impl Default for ReadFrame {
    fn default() -> Self {
        Self {
            eof: false,
            is_readable: false,
            buffer: BytesMut::with_capacity(INITIAL_CAPACITY),
            has_errored: false,
        }
    }
}

impl Default for WriteFrame {
    fn default() -> Self {
        Self {
            buffer: BytesMut::with_capacity(INITIAL_CAPACITY),
            backpressure_boundary: INITIAL_CAPACITY,
        }
    }
}

impl From<BytesMut> for ReadFrame {
    fn from(mut buffer: BytesMut) -> Self {
        let size = buffer.capacity();
        if size < INITIAL_CAPACITY {
            buffer.reserve(INITIAL_CAPACITY - size);
        }

        Self {
            buffer,
            is_readable: size > 0,
            eof: false,
            has_errored: false,
        }
    }
}

impl From<BytesMut> for WriteFrame {
    fn from(mut buffer: BytesMut) -> Self {
        let size = buffer.capacity();
        if size < INITIAL_CAPACITY {
            buffer.reserve(INITIAL_CAPACITY - size);
        }

        Self { buffer, backpressure_boundary: INITIAL_CAPACITY, }
    }
}

impl Borrow<ReadFrame> for RWFrames {
    fn borrow(&self) -> &ReadFrame {
        &self.read
    }
}
impl BorrowMut<ReadFrame> for RWFrames {
    fn borrow_mut(&mut self) -> &mut ReadFrame {
        &mut self.read
    }
}
impl Borrow<WriteFrame> for RWFrames {
    fn borrow(&self) -> &WriteFrame {
        &self.write
    }
}
impl BorrowMut<WriteFrame> for RWFrames {
    fn borrow_mut(&mut self) -> &mut WriteFrame {
        &mut self.write
    }
}

impl<T, U, R> Stream for FramedImpl<T, U, R>
    where
        T: AsyncRead,
        U: Decoder,
        R: BorrowMut<ReadFrame>,
{
    type Item = Result<U::Item, anyhow::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut pinned = self.project();
        let state: &mut ReadFrame = pinned.state.borrow_mut();

        loop {
            if state.has_errored {
                state.is_readable = false;
                state.has_errored = false;
                return Poll::Ready(None);
            }

            if state.is_readable {
                if state.eof {
                    match pinned.codec.decode_eof(&mut state.buffer) {
                        Err(err) => {
                            state.has_errored = true;
                            return Poll::Ready(Some(Err!(err)));
                        }
                        Ok(frame) => {
                            if frame.is_none() {
                                state.is_readable = false;
                            }
                            return Poll::Ready(frame.map(Ok));
                        }
                    }
                }

                if let Some(frame) = match pinned.codec.decode(&mut state.buffer) {
                    Err(err) => {
                        state.has_errored = true;
                        return Poll::Ready(Some(Err!(err)))
                    }
                    Ok(ok) => {
                        ok
                    }
                } { return Poll::Ready(Some(Ok(frame))) }

                state.is_readable = false;
            }

            state.buffer.reserve(1);
            match poll_read_buf(pinned.inner.as_mut(), cx, &mut state.buffer) {
                Poll::Pending => { return Poll::Pending; },
                Poll::Ready(bytect_res) => match bytect_res {
                    Err(err) => {
                        return Poll::Ready(Some(Err!(err)));
                    }
                    Ok(bytect) => {
                        if bytect == 0 {
                            if state.eof {
                                return Poll::Ready(None);
                            }
                            state.eof = true;
                        } else {
                            state.eof = false;
                        }

                        state.is_readable = true;
                    }
                }
            };

        }
    }
}

impl<T, I, U, W> Sink<I> for FramedImpl<T, U, W>
    where
        T: AsyncWrite,
        U: Encoder<I>,
        W: BorrowMut<WriteFrame>,
{
    type Error = anyhow::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        if self.state.borrow().buffer.len() >= self.state.borrow().backpressure_boundary {
            self.as_mut().poll_flush(cx)
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<()> {
        let pinned = self.project();
        match pinned
            .codec
            .encode(item, &mut pinned.state.borrow_mut().buffer) {
            Ok(_) => { Ok(()) }
            Err(_) => { Err!(IoError::EncodeWhileSendError) }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let mut pinned = self.project();

        while !pinned.state.borrow_mut().buffer.is_empty() {
            let WriteFrame { buffer, .. } = pinned.state.borrow_mut();

            match ready!(poll_write_buf(pinned.inner.as_mut(), cx, buffer)) {
                Err(e) => { return Poll::Ready(Err!(e)); }
                Ok(n) => {
                    if n == 0 {
                        return Poll::Ready(Err!(IoError::FailedToFlush));
                    } else {
                        return Poll::Ready(Ok(()));
                    }
                }
            }
        }

        match ready!(pinned.inner.poll_flush(cx)) {
            Err(e) => { return Poll::Ready(Err!(e)) }
            Ok(_) => { return Poll::Ready(Ok(())) }
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match ready!(self.as_mut().poll_flush(cx)) {
            Err(e) => { return Poll::Ready(Err!(e)) }
            Ok(_) => { return Poll::Ready(Ok(())) }
        }

        match ready!(self.project().inner.poll_shutdown(cx)) {
            Err(e) => { return Poll::Ready(Err!(e)) }
            Ok(_) => { return Poll::Ready(Ok(())) }
        }

        Poll::Ready(Ok(()))
    }
}

pub fn poll_read_buf<T: AsyncRead, B: BufMut>(
    io: Pin<&mut T>,
    cx: &mut Context<'_>,
    buf: &mut B,
) -> Poll<Result<usize, T::Error>> {
    if !buf.has_remaining_mut() {
        return Poll::Ready(Ok(0));
    }

    let n = {
        let dst = buf.chunk_mut();

        // Safety: `chunk_mut()` returns a `&mut UninitSlice`, and `UninitSlice` is a
        // transparent wrapper around `[MaybeUninit<u8>]`.
        let dst = unsafe { &mut *(dst as *mut _ as *mut [MaybeUninit<u8>]) };
        let mut buf = ReadBuf::uninit(dst);
        let ptr = buf.filled().as_ptr();
        ready!(io.poll_read(cx, &mut buf)?);

        // Ensure the pointer does not change from under us
        assert_eq!(ptr, buf.filled().as_ptr());
        buf.filled().len()
    };

    // Safety: This is guaranteed to be the number of initialized (and read)
    // bytes due to the invariants provided by `ReadBuf::filled`.
    unsafe {
        buf.advance_mut(n);
    }

    Poll::Ready(Ok(n))
}

pub fn poll_write_buf<T: AsyncWrite, B: Buf>(
    io: Pin<&mut T>,
    cx: &mut Context<'_>,
    buf: &mut B,
) -> Poll<Result<usize, IoError>> {
    const MAX_BUFS: usize = 64;

    if !buf.has_remaining() {
        return Poll::Ready(Ok(0));
    }

    let n = if io.is_write_vectored() {
        let mut slices = [IoSlice::new(&[]); MAX_BUFS];
        let cnt = chunks_vectored(&buf, &mut slices);
        ready!(io.poll_write_vectored(cx, &slices[..cnt]))?
    } else {
        ready!(io.poll_write(cx, buf.chunk()))?
    };

    buf.advance(n);

    Poll::Ready(Ok(n))
}

fn chunks_vectored<'a, B: Buf>(buf: &'a B, dst: &mut [IoSlice<'a>]) -> usize
{
    if dst.is_empty() {
        return 0;
    }

    if buf.has_remaining() {
        dst[0] = IoSlice::new(buf.chunk());
        1
    } else {
        0
    }
}
