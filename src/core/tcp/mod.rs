use alloc::borrow::Cow;
use core::fmt::{Debug, Display};

pub use errors::*;
#[cfg(all(feature = "std", feature = "tls"))]
pub use std_tcp::FromTokio;
#[cfg(feature = "std")]
pub use std_tcp::TcpStream;

pub mod errors;
pub mod tls;

pub trait Connect<'a> {
    type Error: Debug + Display;

    async fn connect(&self, ip: &Cow<'a, str>) -> Result<(), Self::Error>;
}

#[cfg(feature = "std")]
mod std_tcp {
    use alloc::borrow::Cow;
    use core::borrow::BorrowMut;
    use core::cell::RefCell;
    use core::pin::Pin;
    use core::task::{Context, Poll};

    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::net;

    #[cfg(feature = "tls")]
    pub use adapters::FromTokio;

    use crate::core::framed::IoError;
    use crate::core::io;

    use super::errors::TcpError;
    use super::Connect;

    pub struct TcpStream<T> {
        pub(crate) stream: RefCell<Option<T>>,
    }

    impl<T> TcpStream<T> {
        pub fn new() -> Self {
            Self {
                stream: RefCell::new(None),
            }
        }
    }

    impl<'a> Connect<'a> for TcpStream<net::TcpStream> {
        type Error = TcpError;

        async fn connect(&self, ip: &Cow<'a, str>) -> Result<(), TcpError> {
            let result = net::TcpStream::connect(&**ip).await;

            match result {
                Ok(tcp_stream) => {
                    self.stream.replace(Some(tcp_stream));
                    Ok(())
                }
                Err(_) => Err(TcpError::UnableToConnect),
            }
        }
    }

    impl io::AsyncRead for TcpStream<net::TcpStream> {
        type Error = IoError;

        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            match self.stream.borrow_mut().as_mut() {
                None => Poll::Ready(Err(IoError::ReadNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_read(cx, buf) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err(IoError::UnableToRead)),
                    },
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                },
            }
        }
    }

    impl io::AsyncWrite for TcpStream<net::TcpStream> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => Poll::Ready(Err(IoError::WriteNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_write(cx, buf) {
                    Poll::Ready(result) => match result {
                        Ok(ok) => Poll::Ready(Ok(ok)),
                        Err(_) => Poll::Ready(Err(IoError::UnableToWrite)),
                    },
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                },
            }
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => Poll::Ready(Err(IoError::FlushNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_flush(cx) {
                    Poll::Ready(result) => match result {
                        Ok(ok) => Poll::Ready(Ok(ok)),
                        Err(_) => Poll::Ready(Err(IoError::UnableToFlush)),
                    },
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                },
            }
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => Poll::Ready(Err(IoError::ShutdownNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_shutdown(cx) {
                    Poll::Ready(result) => match result {
                        Ok(ok) => Poll::Ready(Ok(ok)),
                        Err(_) => Poll::Ready(Err(IoError::UnableToClose)),
                    },
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                },
            }
        }
    }

    #[cfg(feature = "tls")]
    /// An implementation of FromTokio:
    /// `<https://github.com/embassy-rs/embedded-io/blob/master/src/adapters/tokio.rs>`
    mod adapters {
        use alloc::borrow::Cow;
        use core::pin::Pin;
        use core::task::Poll;

        use embedded_io::asynch::{Read, Write};
        use embedded_io::Io;

        use crate::core::framed::IoError;
        use crate::core::io;
        use crate::core::tcp::{Connect, TcpError};
        use crate::Err;

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

        impl<'a, T: Connect<'a>> Connect<'a> for FromTokio<T> {
            type Error = anyhow::Error;

            async fn connect(&self, ip: &Cow<'a, str>) -> anyhow::Result<()> {
                match self.inner.connect(&ip).await {
                    Err(err) => Err!(err),
                    Ok(_) => Ok(()),
                }
            }
        }

        impl<T> Io for FromTokio<T> {
            type Error = IoError;
        }

        impl<T: io::AsyncRead + Unpin> Read for FromTokio<T> {
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
                poll_fn::poll_fn(|cx| {
                    let mut buf = tokio::io::ReadBuf::new(buf);
                    match Pin::new(&mut self.inner).poll_read(cx, &mut buf) {
                        Poll::Ready(r) => match r {
                            Ok(()) => Poll::Ready(Ok(buf.filled().len())),
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
    }
}
