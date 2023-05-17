//! An implementation of FromTokio:
//! `<https://github.com/embassy-rs/embedded-io/blob/master/src/adapters/tokio.rs>`

use alloc::borrow::Cow;
use anyhow::Result;
use core::pin::Pin;
use core::task::Poll;

#[cfg(not(feature = "std"))]
use crate::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

use embedded_io::asynch::{Read, Write};
use embedded_io::Io;

use crate::core::framed::IoError;
use crate::core::io;
use crate::core::tcp::TcpConnect;
use crate::Err;

/// An adapter to implement `embedded::io::{Io, Read, Write}` for `T`.
#[derive(Debug)]
pub struct FromTokio<T> {
    inner: T,
}

impl<T> FromTokio<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> FromTokio<T> {
    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<'a, T: TcpConnect<'a>> TcpConnect<'a> for FromTokio<T> {
    async fn connect(ip: Cow<'a, str>) -> Result<Self> {
        match T::connect(ip).await {
            Err(conn_err) => Err(conn_err),
            Ok(stream) => Ok( Self { inner: stream } )
        }
    }
}

impl<T> Io for FromTokio<T> {
    type Error = IoError;
}

impl<T: io::AsyncRead + Unpin> Read for FromTokio<T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        poll_fn::poll_fn(|cx| {
            let mut buf = ReadBuf::new(buf);
            match Pin::new(&mut self.inner).poll_read(cx, &mut buf) {
                Poll::Ready(r) => match r {
                    Ok(_) => Poll::Ready(Ok(buf.filled().len())),
                    Err(_) => Poll::Ready(Err(IoError::AdapterTokioReadNotConnected)),
                },
                Poll::Pending => Poll::Pending,
            }
        })
        .await
    }
}

impl<T: io::AsyncWrite + Unpin> Write for FromTokio<T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        poll_fn::poll_fn(|cx| match Pin::new(&mut self.inner).poll_write(cx, buf) {
            Poll::Ready(r) => match r {
                Ok(size) => Poll::Ready(Ok(size)),
                Err(_) => Poll::Ready(Err(IoError::AdapterTokioWriteNotConnected)),
            },
            Poll::Pending => Poll::Pending,
        })
        .await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        poll_fn::poll_fn(|cx| match Pin::new(&mut self.inner).poll_flush(cx) {
            Poll::Ready(r) => match r {
                Ok(_) => Poll::Ready(Ok(())),
                Err(_) => Poll::Ready(Err(IoError::AdapterTokioFlushNotConnected)),
            },
            Poll::Pending => Poll::Pending,
        })
        .await
    }
}

impl<T: io::AsyncWrite + Unpin> FromTokio<T> {
    pub async fn shutdown(&mut self) -> Result<(), IoError> {
        poll_fn::poll_fn(|cx| match Pin::new(&mut self.inner).poll_shutdown(cx) {
            Poll::Ready(r) => match r {
                Ok(_) => Poll::Ready(Ok(())),
                Err(_) => Poll::Ready(Err(IoError::AdapterTokioShutdownNotConnected)),
            },
            Poll::Pending => Poll::Pending,
        })
        .await
    }
}

mod poll_fn {
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};

    struct PollFn<F> {
        f: F,
    }

    impl<F> Unpin for PollFn<F> {}

    pub fn poll_fn<T, F>(f: F) -> impl Future<Output = T>
    where
        F: FnMut(&mut Context<'_>) -> Poll<T>,
    {
        PollFn { f }
    }

    impl<T, F> Future for PollFn<F>
    where
        F: FnMut(&mut Context<'_>) -> Poll<T>,
    {
        type Output = T;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
            (&mut self.f)(cx)
        }
    }
}
