pub mod errors;

use alloc::boxed::Box;
use anyhow::Result;
use core::future::Future;
use core::net::SocketAddr;
use core::pin::Pin;
use core::task::{Context, Poll};

use embedded_io::asynch::{Read, Write};
use embedded_tls::{TlsCipherSuite, TlsConnection, TlsContext, TlsVerifier};
use rand_core::{CryptoRng, RngCore};

#[cfg(not(feature = "std"))]
use crate::core::io::ReadBuf;
#[cfg(feature = "std")]
use tokio::io::ReadBuf;

use crate::core::framed::IoError;
use crate::core::io;
use crate::core::tcp::TcpConnect;
use errors::TlsError;

use crate::Err;

// exports
pub use embedded_tls::{
    blocking::{Aes128GcmSha256, Aes256GcmSha384},
    webpki::CertVerifier,
    NoVerify, TlsConfig,
};

#[derive(Default)]
pub struct TlsSocket<'a, Socket, Cipher>
where
    Socket: Read + Write + 'a,
    Cipher: TlsCipherSuite + 'static,
{
    // TODO: This is just optional so that the `shutdown` works.
    // TODO: `Option` should be required when I found an elegant solution to `shutdown` the TLS connection
    inner: Option<TlsConnection<'a, Socket, Cipher>>,
}

impl<'a, Socket, Cipher> TlsSocket<'a, Socket, Cipher>
where
    Socket: Read + Write + TcpConnect + 'a,
    Cipher: TlsCipherSuite + 'static,
{
    pub async fn connect<Rng: CryptoRng + RngCore, Verifier: TlsVerifier<'a, Cipher>>(
        mut socket: Socket,
        record_read_buf: &'a mut [u8],
        record_write_buf: &'a mut [u8],
        rng: &'a mut Rng,
        config: &'a TlsConfig<'a, Cipher>,
        socket_addr: SocketAddr,
    ) -> Result<Self> {
        socket.connect(socket_addr).await?;
        let mut tls_connection = TlsConnection::new(socket, record_read_buf, record_write_buf);
        if let Err(err) = tls_connection
            .open::<Rng, Verifier>(TlsContext::new(config, rng))
            .await
        {
            return Err!(TlsError::Other(err));
        }

        Ok(Self {
            inner: Some(tls_connection),
        })
    }
}

impl<'a, Socket, Cipher> io::AsyncRead for TlsSocket<'a, Socket, Cipher>
where
    Socket: Read + Write + Unpin + 'a,
    Cipher: TlsCipherSuite + Unpin + 'static,
    <Cipher as TlsCipherSuite>::Hash: Unpin,
    <<<Cipher as TlsCipherSuite>::Hash as crypto_common::OutputSizeUser>::OutputSize as generic_array::ArrayLength<u8>>::ArrayType: Unpin,
    <<<Cipher as TlsCipherSuite>::Hash as crypto_common::BlockSizeUser>::BlockSize as generic_array::ArrayLength<u8>>::ArrayType: Unpin,
{
    type Error = IoError;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        match self.inner.as_mut() {
            None => { Poll::Ready(Err(IoError::ReadNotConnected)) }
            Some(tls_connection) => match Pin::new(&mut Box::pin(tls_connection.read(buf.filled_mut()))).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(0) => {
                        // no data ready
                        Poll::Pending
                    }
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(e) => Poll::Ready(Err(IoError::TlsRead(e))),
                },
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

impl<'a, Socket, Cipher> io::AsyncWrite for TlsSocket<'a, Socket, Cipher>
where
    Socket: Read + Write + Unpin + 'a,
    Cipher: TlsCipherSuite + Unpin + 'static,
    <Cipher as TlsCipherSuite>::Hash: Unpin,
    <<<Cipher as TlsCipherSuite>::Hash as crypto_common::OutputSizeUser>::OutputSize as generic_array::ArrayLength<u8>>::ArrayType: Unpin,
    <<<Cipher as TlsCipherSuite>::Hash as crypto_common::BlockSizeUser>::BlockSize as generic_array::ArrayLength<u8>>::ArrayType: Unpin,
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        match self.inner.as_mut() {
            None => { Poll::Ready(Err!(IoError::WriteNotConnected)) }
            Some(tls_connection) => match Pin::new(&mut Box::pin(tls_connection.write(buf))).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(size) => Poll::Ready(Ok(size)),
                    Err(_) => Poll::Ready(Err!(IoError::UnableToWrite)),
                },
                Poll::Pending => Poll::Pending,
            }
        }

    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.inner.as_mut() {
            None => { Poll::Ready(Err!(IoError::WriteNotConnected)) }
            Some(tls_connection) => match Pin::new(&mut Box::pin(tls_connection.flush())).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(_) => Poll::Ready(Err!(IoError::UnableToFlush)),
                },
                Poll::Pending => Poll::Pending,
            }
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        let tls_connection = core::mem::take(&mut self.inner).unwrap();

        // TODO: Find an elegant solution
        let _ = tls_connection.close();
        Poll::Ready(Ok(()))
    }
}
