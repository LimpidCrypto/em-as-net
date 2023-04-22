//! A no_std version of `tokio::Framed`

use anyhow::Result;
pub mod codec;
pub use codec::Codec;

mod framed_impl;
use framed_impl::{FramedImpl, RWFrames, ReadFrame, WriteFrame};

pub mod errors;
pub use errors::*;

use bytes::BytesMut;
use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::{Sink, Stream};
use pin_project_lite::pin_project;

use super::io::{AsyncRead, AsyncWrite};

pub use errors::IoError;

use codec::{Decoder, Encoder};

pin_project! {
    pub struct Framed<T, U> {
        #[pin]
        pub(crate) inner: FramedImpl<T, U, RWFrames>
    }
}

impl<T, U> Framed<T, U>
where
    T: AsyncRead + AsyncWrite,
{
    pub fn new(inner: T, codec: U) -> Framed<T, U> {
        Framed {
            inner: FramedImpl {
                inner,
                codec,
                state: Default::default(),
            },
        }
    }

    pub fn with_capacity(inner: T, codec: U, capacity: usize) -> Framed<T, U> {
        Framed {
            inner: FramedImpl {
                inner,
                codec,
                state: RWFrames {
                    read: ReadFrame {
                        eof: false,
                        is_readable: false,
                        buffer: BytesMut::with_capacity(capacity),
                        has_errored: false,
                    },
                    write: WriteFrame::default(),
                },
            },
        }
    }
}

impl<T, U> Framed<T, U> {
    pub fn get_ref(&self) -> &T {
        &self.inner.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner.inner
    }

    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        self.project().inner.project().inner
    }

    pub fn codec(&self) -> &U {
        &self.inner.codec
    }

    pub fn codec_mut(&mut self) -> &mut U {
        &mut self.inner.codec
    }

    pub fn codec_pin_mut(self: Pin<&mut Self>) -> &mut U {
        self.project().inner.project().codec
    }

    pub fn read_buffer(&self) -> &BytesMut {
        &self.inner.state.read.buffer
    }

    pub fn read_buffer_mut(&mut self) -> &mut BytesMut {
        &mut self.inner.state.read.buffer
    }

    pub fn write_buffer(&self) -> &BytesMut {
        &self.inner.state.write.buffer
    }

    pub fn write_buffer_mut(&mut self) -> &mut BytesMut {
        &mut self.inner.state.write.buffer
    }

    pub fn backpressure_boundary(&self) -> usize {
        self.inner.state.write.backpressure_boundary
    }

    pub fn set_backpressure_boundary(&mut self, boundary: usize) {
        self.inner.state.write.backpressure_boundary = boundary;
    }

    pub fn into_inner(self) -> T {
        self.inner.inner
    }
}

// This impl just defers to the underlying FramedImpl
impl<T, U> Stream for Framed<T, U>
where
    T: AsyncRead,
    U: Decoder,
{
    type Item = Result<U::Item, anyhow::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}

// This impl just defers to the underlying FramedImpl
impl<T, I, U> Sink<I> for Framed<T, U>
where
    T: AsyncWrite,
    U: Encoder<I>,
{
    type Error = anyhow::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<()> {
        self.project().inner.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_close(cx)
    }
}

impl<T, U> fmt::Debug for Framed<T, U>
where
    T: fmt::Debug,
    U: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Framed")
            .field("io", self.get_ref())
            .field("codec", self.codec())
            .finish()
    }
}
