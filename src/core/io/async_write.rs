use alloc::boxed::Box;
use core::ops::DerefMut;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::core::framed::IoError;

use super::io_slice::IoSlice;

pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, IoError>>;

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, IoError>> {
        let buf = bufs
            .iter()
            .find(|b| !b.is_empty())
            .map_or(&[][..], |b| &**b);
        self.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>>;

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>>;

    fn is_write_vectored(&self) -> bool {
        false
    }
}

macro_rules! deref_async_write {
    () => {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, IoError>> {
            Pin::new(&mut **self).poll_write(cx, buf)
        }

        fn poll_write_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[IoSlice<'_>],
        ) -> Poll<Result<usize, IoError>> {
            Pin::new(&mut **self).poll_write_vectored(cx, bufs)
        }

        fn is_write_vectored(&self) -> bool {
            (**self).is_write_vectored()
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            Pin::new(&mut **self).poll_flush(cx)
        }

        fn poll_shutdown(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), IoError>> {
            Pin::new(&mut **self).poll_shutdown(cx)
        }
    };
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for Box<T> {
    deref_async_write!();
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for &mut T {
    deref_async_write!();
}

impl<P> AsyncWrite for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, IoError>> {
        self.get_mut().as_mut().poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, IoError>> {
        self.get_mut().as_mut().poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        self.get_mut().as_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        self.get_mut().as_mut().poll_shutdown(cx)
    }

    fn is_write_vectored(&self) -> bool {
        (**self).is_write_vectored()
    }
}

// TODO: implement if needed, otherwise delete
// impl AsyncWrite for Vec<u8> {
//     fn poll_write(
//         self: Pin<&mut Self>,
//         _cx: &mut Context<'_>,
//         buf: &[u8],
//     ) -> Poll<Result<usize, AsyncWriteException>> {
//         self.get_mut().extend_from_slice(buf);
//         Poll::Ready(Ok(buf.len()))
//     }
//
//     fn poll_write_vectored(
//         mut self: Pin<&mut Self>,
//         _: &mut Context<'_>,
//         bufs: &[IoSlice<'_>],
//     ) -> Poll<Result<usize, AsyncWriteException>> {
//         Poll::Ready(<dyn AsyncWrite>::write_vectored(&mut *self, bufs))
//     }
//
//     fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), AsyncWriteException>> {
//         Poll::Ready(Ok(()))
//     }
//
//     fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), AsyncWriteException>> {
//         Poll::Ready(Ok(()))
//     }
//
//     fn is_write_vectored(&self) -> bool {
//         true
//     }
// }
