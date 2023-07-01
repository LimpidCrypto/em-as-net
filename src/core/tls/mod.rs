pub mod errors;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use anyhow::Result;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use embedded_io::asynch::{Read, Write};
use embedded_tls::{NoVerify, TlsCipherSuite, TlsConfig, TlsContext};
use rand_core::OsRng;

#[cfg(not(feature = "std"))]
use crate::core::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

use crate::core::framed::IoError;
use crate::core::io;
use crate::core::tcp::TcpConnect;
use errors::TlsError;

use crate::Err;

pub use embedded_tls::blocking::{Aes128GcmSha256, Aes256GcmSha384};

pub struct TlsConnection<'a, S, C>
where
    S: Read + Write + 'a,
    C: TlsCipherSuite + 'static,
{
    tls: RefCell<Option<embedded_tls::TlsConnection<'a, S, C>>>,
    socket: RefCell<Option<S>>,
    read_buf: RefCell<&'a mut [u8]>,
    write_buf: RefCell<&'a mut [u8]>,
}

impl<'a, S, C> TlsConnection<'a, S, C>
where
    S: Read + Write + TcpConnect<'a> + 'a,
    C: TlsCipherSuite + 'static,
{
    pub fn new(socket: S, read_buf: &'a mut [u8], write_buf: &'a mut [u8]) -> Self {
        Self {
            tls: RefCell::new(None),
            socket: RefCell::new(Some(socket)),
            read_buf: RefCell::new(read_buf),
            write_buf: RefCell::new(write_buf),
        }
    }
}

impl<'a, S, C> TcpConnect<'a> for TlsConnection<'a, S, C>
where
    S: Read + Write + TcpConnect<'a> + 'a,
    C: TlsCipherSuite + 'static,
{
    async fn connect(&self, ip: Cow<'a, str>) -> Result<()> {
        match self.socket.borrow_mut().take() {
            None => Err!(TlsError::FailedToOpen),
            Some(socket) => {
                socket.connect(ip).await?;
                Ok(self.tls.replace(Some(embedded_tls::TlsConnection::new(
                    socket,
                    self.read_buf.take(),
                    self.write_buf.take(),
                ))))
            }
        }
        .map_err(|err| err)?;

        match self.tls.borrow_mut().as_mut() {
            None => {
                return Err!(TlsError::NotConnected);
            }
            Some(tls) => {
                let mut rng = OsRng; // use rng core generic
                let config = TlsConfig::new().with_server_name("vi-server.org"); // TODO: This is just for testing; TLS currently not working anyway
                if let Err(err) = tls
                    .open::<OsRng, NoVerify>(TlsContext::new(&config, &mut rng))
                    .await
                {
                    return Err!(TlsError::Other(err));
                }
            }
        }

        Ok(())
    }
}

impl<'a, S, C> io::AsyncRead for TlsConnection<'a, S, C>
where
    S: Read + Write + 'a,
    C: TlsCipherSuite + 'static,
{
    type Error = IoError;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        match self.tls.borrow_mut().as_mut() {
            None => Poll::Ready(Err(IoError::TlsReadNotConnected)),
            Some(stream) => {
                match Pin::new(&mut Box::pin(stream.read(buf.filled_mut()))).poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(0) => {
                            // no data ready
                            Poll::Pending
                        }
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(e) => return Poll::Ready(Err(IoError::TlsRead(e))),
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

impl<'a, S, C> io::AsyncWrite for TlsConnection<'a, S, C>
where
    S: Read + Write + 'a,
    C: TlsCipherSuite + 'static,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        match self.tls.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::TlsWriteNotConnected)),
            Some(stream) => match Pin::new(&mut Box::pin(stream.write(buf))).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(size) => Poll::Ready(Ok(size)),
                    Err(_) => Poll::Ready(Err!(IoError::UnableToWrite)),
                },
                Poll::Pending => Poll::Pending,
            },
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.tls.borrow_mut().as_mut() {
            None => Poll::Ready(Err!(IoError::TlsFlushNotConnected)),
            Some(stream) => {
                let mut fut = Box::pin(stream.flush());
                let fut_pinned = Pin::new(&mut fut);
                match fut_pinned.poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err!(IoError::UnableToFlush)),
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.tls.take() {
            None => Poll::Ready(Err!(IoError::TlsShutdownNotConnected)),
            Some(stream) => match Pin::new(&mut Box::pin(stream.close())).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(_) => Poll::Ready(Err!(IoError::UnableToClose)),
                },
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
