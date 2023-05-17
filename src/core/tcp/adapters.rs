use alloc::borrow::Cow;
use anyhow::Result;
use embassy_net_driver::Driver;

pub trait AdaptConnect<'a> {
    /// Defines and connects the `inner` of an adapter to the host
    async fn connect(ip: Cow<'a, str>) -> Result<Self> where Self: Sized;
}

#[cfg(feature = "std")]
pub use std_adapters::TcpTokio;

#[cfg(feature = "std")]
mod std_adapters {
    use alloc::borrow::Cow;
    use core::cell::RefCell;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use core::borrow::BorrowMut;
    use anyhow::Result;
    use tokio::io::ReadBuf;
    use tokio::net::TcpStream;
    use tokio::io::{AsyncRead, AsyncWrite};
    use crate::core::framed::IoError;
    use crate::core::io;
    use crate::core::tcp::adapters::AdaptConnect;
    use crate::core::tcp::errors::TcpError;
    use crate::Err;

    #[derive(Debug)]
    pub struct TcpTokio {
        inner: RefCell<Option<TcpStream>>,
    }

    impl<'a> AdaptConnect<'a> for TcpTokio {
        async fn connect(ip: Cow<'a, str>) -> Result<Self> {
            match TcpStream::connect(&*ip).await {
                Err(_) => Err!(TcpError::UnableToConnect),
                Ok(stream) => Ok( Self { inner: RefCell::new(Some(stream)) } ),
            }
        }
    }

    impl io::AsyncRead for TcpTokio {
        type Error = anyhow::Error;

        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            match self.inner.borrow_mut().as_mut() {
                None => Poll::Ready(Err!(IoError::ReadNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_read(cx, buf) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err!(IoError::UnableToRead)),
                    },
                    Poll::Pending => Poll::Pending,
                },
            }
        }
    }

    impl io::AsyncWrite for TcpTokio {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            match self.inner.borrow_mut().as_mut() {
                None => Poll::Ready(Err!(IoError::WriteNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_write(cx, buf) {
                    Poll::Ready(result) => match result {
                        Ok(size) => Poll::Ready(Ok(size)),
                        Err(_) => Poll::Ready(Err!(IoError::UnableToWrite)),
                    },
                    Poll::Pending => Poll::Pending,
                },
            }
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            match self.inner.borrow_mut().as_mut() {
                None => Poll::Ready(Err!(IoError::FlushNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_flush(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err!(IoError::UnableToFlush)),
                    },
                    Poll::Pending => Poll::Pending,
                },
            }
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            match self.inner.borrow_mut().as_mut() {
                None => Poll::Ready(Err!(IoError::ShutdownNotConnected)),
                Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_shutdown(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err!(IoError::UnableToClose)),
                    },
                    Poll::Pending => Poll::Pending,
                },
            }
        }
    }
}

mod no_std_adapters {
    // use alloc::borrow::Cow;
    // use core::cell::RefCell;
    // use core::pin::Pin;
    // use core::task::{Context, Poll};
    // use core::borrow::BorrowMut;
    // use anyhow::Result;
    // use tokio::io::ReadBuf;
    // use embassy_net::tcp::TcpSocket;
    // use embassy_net_driver::Driver;
    // use crate::core::framed::IoError;
    // use crate::core::io;
    // use crate::core::tcp::adapters::AdaptConnect;
    // use crate::core::tcp::errors::TcpError;
    // use crate::Err;
    //
    // #[derive(Debug)]
    // pub struct TcpEmbassy<'a> {
    //     inner: RefCell<Option<TcpSocket<'a>>>,
    // }
    //
    // impl<'a> TcpEmbassy<'a> {
    //     pub fn new(socket: TcpSocket<'a>) -> Self {
    //         Self { inner: socket }
    //     }
    // }
    //
    // impl<'a> AdaptConnect<'a> for TcpEmbassy<'a>  {
    //     async fn connectx(ip: Cow<'a, str>) -> Result<Self> {
    //         match TcpSocket::connect(&*ip).await {
    //             Err(_) => Err!(TcpError::UnableToConnect),
    //             Ok(stream) => Ok( Self { inner: RefCell::new(Some(stream)) } ),
    //         }
    //     }
    // }

    // impl<'a> io::AsyncRead for TcpEmbassy<'a>  {
    //     type Error = anyhow::Error;
    //
    //     fn poll_read(
    //         self: Pin<&mut Self>,
    //         cx: &mut Context<'_>,
    //         buf: &mut ReadBuf<'_>,
    //     ) -> Poll<Result<(), Self::Error>> {
    //         match self.inner.borrow_mut().as_mut() {
    //             None => Poll::Ready(Err!(IoError::ReadNotConnected)),
    //             Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_read(cx, buf) {
    //                 Poll::Ready(result) => match result {
    //                     Ok(_) => Poll::Ready(Ok(())),
    //                     Err(_) => Poll::Ready(Err!(IoError::UnableToRead)),
    //                 },
    //                 Poll::Pending => Poll::Pending,
    //             },
    //         }
    //     }
    // }
    //
    // impl<'a> io::AsyncWrite for TcpEmbassy<'a>  {
    //     fn poll_write(
    //         self: Pin<&mut Self>,
    //         cx: &mut Context<'_>,
    //         buf: &[u8],
    //     ) -> Poll<Result<usize>> {
    //         match self.inner.borrow_mut().as_mut() {
    //             None => Poll::Ready(Err!(IoError::WriteNotConnected)),
    //             Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_write(cx, buf) {
    //                 Poll::Ready(result) => match result {
    //                     Ok(size) => Poll::Ready(Ok(size)),
    //                     Err(_) => Poll::Ready(Err!(IoError::UnableToWrite)),
    //                 },
    //                 Poll::Pending => Poll::Pending,
    //             },
    //         }
    //     }
    //
    //     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
    //         match self.inner.borrow_mut().as_mut() {
    //             None => Poll::Ready(Err!(IoError::FlushNotConnected)),
    //             Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_flush(cx) {
    //                 Poll::Ready(result) => match result {
    //                     Ok(_) => Poll::Ready(Ok(())),
    //                     Err(_) => Poll::Ready(Err!(IoError::UnableToFlush)),
    //                 },
    //                 Poll::Pending => Poll::Pending,
    //             },
    //         }
    //     }
    //
    //     fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
    //         match self.inner.borrow_mut().as_mut() {
    //             None => Poll::Ready(Err!(IoError::ShutdownNotConnected)),
    //             Some(stream) => match Pin::new(stream).borrow_mut().as_mut().poll_shutdown(cx) {
    //                 Poll::Ready(result) => match result {
    //                     Ok(_) => Poll::Ready(Ok(())),
    //                     Err(_) => Poll::Ready(Err!(IoError::UnableToClose)),
    //                 },
    //                 Poll::Pending => Poll::Pending,
    //             },
    //         }
    //     }
    // }
}
