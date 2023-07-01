use crate::core::framed::IoError;
use crate::core::tcp::adapters::AdapterConnect;
use crate::Err;
use alloc::borrow::Cow;
use anyhow::Result;
use core::borrow::BorrowMut;
use core::fmt::Debug;
use core::future::poll_fn;
use core::pin::Pin;
use core::task::{Context, Poll};
use embedded_io::asynch::{Read, Write};
use embedded_io::Io;

use crate::core::io::{AsyncRead, AsyncWrite};
#[cfg(not(feature = "std"))]
use crate::core::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

pub mod adapters;
pub mod errors;

// TODO: utilize to check `state`
pub struct Socket;
pub struct Stream;

#[derive(Debug)]
pub struct TcpSocket<T> {
    pub(crate) socket: T,
}

impl<T> TcpSocket<T> {
    pub fn new(socket: T) -> Self {
        Self {
            socket,
        }
    }
}

impl<'a, T> TcpConnect<'a> for TcpSocket<T>
where
    T: AdapterConnect<'a>,
{
    async fn connect(&mut self, socket_address: Cow<'a, str>) -> Result<()> {
        // TODO: `socket_address` should be of type `SocketAddr`
        self.socket.connect(socket_address).await
    }
}

impl<T> AsyncRead for TcpSocket<T>
where
    T: AsyncRead + Unpin,
{
    type Error = anyhow::Error;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        match Pin::new(&mut self.socket).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> AsyncWrite for TcpSocket<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.socket).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.socket).poll_flush(cx)

    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.socket).poll_shutdown(cx)

    }
}

impl<T> Io for TcpSocket<T> {
    type Error = IoError;
}

impl<'a, T> Read for TcpSocket<T>
where
    T: AsyncRead + Unpin,
{
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        let size = buf.len();
        poll_fn(|cx| Pin::new(&mut self.socket).poll_read(cx, ReadBuf::new(buf).borrow_mut()))
            .await
            .map_err(|_| IoError::UnableToRead)?;

        Ok(size)
    }
}

impl<'a, T> Write for TcpSocket<T>
where
    T: AsyncWrite + Unpin,
{
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        let size = buf.len();
        poll_fn(|cx| Pin::new(&mut self.socket).poll_write(cx, buf))
            .await
            .map_err(|_| IoError::UnableToWrite)?;
        poll_fn(|cx| Pin::new(&mut self.socket).poll_flush(cx))
            .await
            .map_err(|_| IoError::UnableToFlush)?;

        Ok(size)
    }
}

pub trait TcpConnect<'a> {
    async fn connect(&mut self, ip: Cow<'a, str>) -> Result<()>;
}
