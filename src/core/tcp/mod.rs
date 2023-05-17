use crate::core::framed::IoError;
use crate::core::io;
use crate::core::tcp::adapters::AdaptConnect;
use crate::Err;
use alloc::borrow::Cow;
use anyhow::Result;
use core::cell::RefCell;
use core::fmt::Debug;
use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(not(feature = "std"))]
use crate::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

pub mod adapters;
pub mod errors;

#[derive(Debug)]
pub struct TcpSocket<T> {
    pub(crate) stream: RefCell<Option<T>>,
}

impl<'a, T> TcpConnect<'a> for TcpSocket<T>
where
    T: AdaptConnect<'a>,
{
    async fn connect(ip: Cow<'a, str>) -> Result<Self> {
        match T::connect(ip).await {
            Ok(adapter) => Ok(Self {
                stream: RefCell::new(Some(adapter)),
            }),
            Err(conn_err) => Err!(conn_err),
        }
    }
}

impl<'a, T> io::AsyncRead for TcpSocket<T>
where
    T: io::AsyncRead + Unpin,
{
    type Error = anyhow::Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        match self.stream.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::ReadNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_read(cx, buf),
        }
    }
}

impl<'a, T> io::AsyncWrite for TcpSocket<T>
where
    T: io::AsyncWrite + Unpin,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        match self.stream.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::WriteNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.stream.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::FlushNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.stream.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::ShutdownNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_shutdown(cx),
        }
    }
}

pub trait TcpConnect<'a> {
    async fn connect(ip: Cow<'a, str>) -> Result<Self>
    where
        Self: Sized;
}
