pub mod errors;

mod adapters;
#[cfg(feature = "std")]
pub use std_adapters::*;

mod std_adapters {
    pub use super::adapters::FromTokio;
}

use alloc::borrow::Cow;
use alloc::boxed::Box;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use embedded_io::asynch::{Read, Write};
use embedded_tls::{NoVerify, TlsCipherSuite, TlsConfig, TlsContext};
use rand_core::OsRng;

#[cfg(not(feature = "std"))]
use crate::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

use crate::core::framed::IoError;
use crate::core::io;
use crate::core::tcp::Connect;
use errors::TlsError;

use crate::Err;

pub struct TlsConnection<'a, S, C>
    where
        S: Read + Write + 'a,
        C: TlsCipherSuite + 'static,
{
    inner: RefCell<Option<embedded_tls::TlsConnection<'a, S, C>>>,
}

impl<'a, S, C> TlsConnection<'a, S, C>
    where
        S: Read + Write + Connect<'a> + 'a,
        C: TlsCipherSuite + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(None),
        }
    }

    pub async fn open(
        &self,
        server_name: Cow<'a, str>,
        delegate: S,
        read_buffer: &'a mut [u8],
        write_buffer: &'a mut [u8],
    ) -> anyhow::Result<()> {
        let tls = match delegate.connect(server_name.clone()).await {
            Ok(_) => embedded_tls::TlsConnection::new(delegate, read_buffer, write_buffer),
            Err(err) => {
                return Err!(err);
            }
        };

        self.inner.replace(Some(tls));
        match self.inner.borrow_mut().as_mut() {
            None => {
                return Err!(TlsError::NotConnected);
            }
            Some(tls) => {
                let mut rng = OsRng;
                let config = TlsConfig::new().with_server_name("limpidcrypto.de");
                if let Err(err) = tls
                    .open::<OsRng, NoVerify>(TlsContext::new(&config, &mut rng))
                    .await
                {
                    return Err!(TlsError::Other(err));
                }
            }
        }
        assert!(self.inner.borrow_mut().as_mut().is_some());
        Ok(())
    }
}

impl<'a, S, C> Default for TlsConnection<'a, S, C>
    where
        S: Read + Write + Connect<'a> + 'a,
        C: TlsCipherSuite + 'static,
{
    fn default() -> Self {
        TlsConnection::new()
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
        match self.inner.borrow_mut().as_mut() {
            None => Poll::Ready(Err(IoError::TlsReadNotConnected)),
            Some(stream) => {
                match Pin::new(&mut Box::pin(stream.read(buf.filled_mut()))).poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err(IoError::DecodeWhileReadError)),
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
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, IoError>> {
        match self.inner.borrow_mut().as_mut() {
            None => Poll::Ready(Err(IoError::TlsWriteNotConnected)),
            Some(stream) => match Pin::new(&mut Box::pin(stream.write(buf))).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(size) => Poll::Ready(Ok(size)),
                    Err(_) => Poll::Ready(Err(IoError::UnableToWrite)),
                },
                Poll::Pending => Poll::Pending,
            },
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        match self.inner.borrow_mut().as_mut() {
            None => Poll::Ready(Err(IoError::TlsFlushNotConnected)),
            Some(stream) => {
                let mut fut = Box::pin(stream.flush());
                let fut_pinned = Pin::new(&mut fut);
                match fut_pinned.poll(cx) {
                    Poll::Ready(result) => match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(_) => Poll::Ready(Err(IoError::UnableToFlush)),
                    },
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        match self.inner.take() {
            None => Poll::Ready(Err(IoError::TlsShutdownNotConnected)),
            Some(stream) => match Pin::new(&mut Box::pin(stream.close())).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(_) => Poll::Ready(Err(IoError::UnableToClose)),
                },
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
