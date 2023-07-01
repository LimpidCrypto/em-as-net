use alloc::borrow::Cow;
use anyhow::Result;

#[cfg(feature = "std")]
pub use std_adapters::TcpAdapterTokio;

#[cfg(feature = "std")]
mod std_adapters {
    use crate::core::framed::IoError;
    use crate::core::io;
    use crate::core::tcp::adapters::AdapterConnect;
    use crate::core::tcp::errors::TcpError;
    use crate::Err;
    use alloc::borrow::Cow;
    use anyhow::Result;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use tokio::io::ReadBuf;
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::net::TcpStream;

    #[derive(Debug, Default)]
    pub struct TcpAdapterTokio {
        inner: Option<TcpStream>,
    }

    impl TcpAdapterTokio {
        pub fn new() -> Self {
            Self {
                inner: None
            }
        }
    }

    impl<'a> AdapterConnect<'a> for TcpAdapterTokio {
        async fn connect(&mut self, ip: Cow<'a, str>) -> Result<()> {
            match TcpStream::connect(&*ip).await {
                Err(_) => Err!(TcpError::UnableToConnect), // TODO: return the error returned by `tokio::net::TcpStream`
                Ok(stream) => {
                    self.inner = Some(stream);
                    Ok(())
                }
            }
        }
    }

    impl io::AsyncRead for TcpAdapterTokio {
        type Error = anyhow::Error;

        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            match Pin::new(&mut self.inner).get_mut() {
                None => { Poll::Ready(Err!(IoError::ReadNotConnected)) }
                Some(socket) => match Pin::new(socket).poll_read(cx, buf) {
                    Poll::Ready(result) => match result {
                        Ok(()) => { Poll::Ready(Ok(())) }
                        Err(error) => { Poll::Ready(Err!(error)) }
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }

    impl io::AsyncWrite for TcpAdapterTokio {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            match Pin::new(&mut self.inner).get_mut() {
                None => { Poll::Ready(Err!(IoError::ReadNotConnected)) }
                Some(socket) => match Pin::new(socket).poll_write(cx, buf) {
                    Poll::Ready(result) => match result {
                        Ok(size) => { Poll::Ready(Ok(size)) }
                        Err(error) => { Poll::Ready(Err!(error)) }
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            match Pin::new(&mut self.inner).get_mut() {
                None => { Poll::Ready(Err!(IoError::ReadNotConnected)) }
                Some(socket) => match Pin::new(socket).poll_flush(cx) {
                    Poll::Ready(result) => match result {
                        Ok(()) => { Poll::Ready(Ok(())) }
                        Err(error) => { Poll::Ready(Err!(error)) }
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            match Pin::new(&mut self.inner).get_mut() {
                None => { Poll::Ready(Err!(IoError::ReadNotConnected)) }
                Some(socket) => match Pin::new(socket).poll_shutdown(cx) {
                    Poll::Ready(result) => match result {
                        Ok(()) => { Poll::Ready(Ok(())) }
                        Err(error) => { Poll::Ready(Err!(error)) }
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

// mod no_std_adapters {
//     use alloc::borrow::Cow;
//     use core::cell::RefCell;
//     use core::pin::Pin;
//     use core::task::{Context, Poll};
//     use core::borrow::BorrowMut;
//     use anyhow::Result;
//     use embassy_net::tcp::TcpSocket;
//     use embassy_net_driver::Driver;
//     use super::AdapterConnect;
//     use crate::core::io::ReadBuf;
//     use crate::core::framed::IoError;
//     use crate::core::io;
//     use crate::core::tcp::errors::TcpError;
//     use crate::Err;
//
//     pub struct TcpAdapterEmbassy<'a> {
//         inner: RefCell<Option<TcpSocket<'a>>>,
//     }
//
//     impl<'a> TcpAdapterEmbassy<'a> {
//         pub fn new(socket: TcpSocket<'a>) -> Self {
//             Self { inner: socket }
//         }
//     }
//
//     impl<'a> AdapterConnect<'a> for TcpAdapterEmbassy<'a>  {
//         async fn connect(&self, ip: Cow<'a, str>) -> Result<()> {
//             match self.inner.borrow_mut().as_mut() {
//                 None => Err!(TcpError::UnableToConnect),
//                 Ok(socket) => {
//                     socket.connect
//                 }
//             }
//         }
//     }
// }

pub trait AdapterConnect<'a> {
    /// Defines and connects the `inner` of an adapter to the host
    async fn connect(&mut self, ip: Cow<'a, str>) -> Result<()>;
}
