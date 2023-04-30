use alloc::borrow::Cow;
use core::fmt::{Debug, Display};

pub mod errors;
#[cfg(feature = "std")]
pub use std_tcp::TcpStream;


pub trait Connect<'a> {
    type Error: Debug + Display;

    async fn connect(&self, ip: Cow<'a, str>) -> Result<(), Self::Error>;
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

    use crate::core::framed::IoError;
    use crate::core::io;

    use super::errors::TcpError;
    use super::Connect;

    #[derive(Debug)]
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

    impl<T> Default for TcpStream<T> {
        fn default() -> Self {
            TcpStream::new()
        }
    }

    impl<'a> Connect<'a> for TcpStream<net::TcpStream> {
        type Error = TcpError;

        async fn connect(&self, ip: Cow<'a, str>) -> Result<(), TcpError> {
            let result = net::TcpStream::connect(&*ip).await;

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
                    Poll::Pending => Poll::Pending,
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
                        Ok(size) => Poll::Ready(Ok(size)),
                        Err(_) => Poll::Ready(Err(IoError::UnableToWrite)),
                    },
                    Poll::Pending => Poll::Pending,
                },
            }
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => Poll::Ready(Err(IoError::FlushNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_flush(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err(IoError::UnableToFlush)),
                    },
                    Poll::Pending => Poll::Pending,
                },
            }
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => Poll::Ready(Err(IoError::ShutdownNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_shutdown(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err(IoError::UnableToClose)),
                    },
                    Poll::Pending => Poll::Pending,
                },
            }
        }
    }
}
