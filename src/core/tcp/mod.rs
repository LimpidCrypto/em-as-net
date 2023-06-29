use crate::core::framed::IoError;
use crate::core::tcp::adapters::AdapterConnect;
use crate::Err;
use alloc::borrow::Cow;
use anyhow::Result;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::fmt::Debug;
use core::future::poll_fn;
use core::pin::Pin;
use core::task::{Context, Poll};
use embedded_io::asynch::{Read, Write};
use embedded_io::Io;

use crate::core::io::{AsyncRead, AsyncWrite};
use crate::core::tcp::errors::TcpError;
#[cfg(not(feature = "std"))]
use crate::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

pub mod adapters;
pub mod errors;

// TODO: utilize to check `state`
pub struct Socket;
pub struct Stream;

#[derive(Debug)]
pub struct TcpSocket<T> {
    pub(crate) socket: RefCell<Option<T>>,
}

impl<T> TcpSocket<T> {
    pub fn new(socket: T) -> Self {
        Self {
            socket: RefCell::new(Some(socket)),
        }
    }
}

impl<'a, T> TcpConnect<'a> for TcpSocket<T>
where
    T: AdapterConnect<'a>,
{
    async fn connect(&self, socket_address: Cow<'a, str>) -> Result<()> {
        // TODO: `socket_address` should be of type `SocketAddr`
        match self.socket.borrow_mut().as_mut() {
            None => Err!(TcpError::UnableToConnect),
            Some(s) => Ok(s.connect(socket_address).await?),
        }
    }
}

impl<'a, T> AsyncRead for TcpSocket<T>
where
    T: AsyncRead + Unpin,
{
    type Error = anyhow::Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        match self.socket.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::ReadNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_read(cx, buf),
        }
    }
}

impl<'a, T> AsyncWrite for TcpSocket<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        match self.socket.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::WriteNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.socket.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::FlushNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.socket.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::ShutdownNotConnected)),
            Some(mut stream) => Pin::new(&mut stream).poll_shutdown(cx),
        }
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
        match self.socket.borrow_mut().as_mut() {
            None => Err(IoError::ReadNotConnected),
            Some(mut stream) => {
                poll_fn(|cx| Pin::new(&mut stream).poll_read(cx, ReadBuf::new(buf).borrow_mut()))
                    .await
                    .map_err(|_| IoError::UnableToRead)?;

                Ok(size)
            }
        }
    }
}

impl<'a, T> Write for TcpSocket<T>
where
    T: AsyncWrite + Unpin,
{
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        let size = buf.len();
        match self.socket.borrow_mut().as_mut() {
            None => Err(IoError::WriteNotConnected),
            Some(mut stream) => {
                poll_fn(|cx| Pin::new(&mut stream).poll_write(cx, buf))
                    .await
                    .map_err(|_| IoError::UnableToWrite)?;
                poll_fn(|cx| Pin::new(&mut stream).poll_flush(cx))
                    .await
                    .map_err(|_| IoError::UnableToFlush)?;

                Ok(size)
            }
        }
    }
}

pub trait TcpConnect<'a> {
    async fn connect(&self, ip: Cow<'a, str>) -> Result<()>;
}
