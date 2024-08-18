mod exceptions;

use embedded_io_adapters::tokio_1::FromTokio;
pub use exceptions::*;
use tokio_rustls::client::TlsStream;

use anyhow::Result;
use embedded_io_async::{Read, Write};

#[cfg(not(feature = "std"))]
use rustls::{ClientConnection, ServerConnection};
#[cfg(feature = "std")]
use tokio_rustls::TlsConnector;

#[cfg(not(feature = "std"))]
pub struct TlsSocketClient(ClientConnection);
#[cfg(not(feature = "std"))]
pub struct TlsSocketServer(ServerConnection);

#[cfg(feature = "std")]
pub struct TlsSocket<S>(FromTokio<TlsStream<S>>);

#[cfg(feature = "std")]
mod tokio_tls_client {
    use alloc::sync::Arc;
    use embedded_io_async::ErrorType;
    use rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
    use tokio::io::{AsyncRead, AsyncWrite};
    use url::Url;

    use crate::core::tcp::TcpStream;

    use super::*;

    impl<S> TlsSocket<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        pub async fn connect<'a>(url: Url) -> Result<TlsSocket<S>> {
            let stream = TcpStream::new(inner)
            let mut root_cert_store = RootCertStore::empty();
            root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();
            let connector = TlsConnector::from(Arc::new(config));

            let stream = connector
                .connect(server_name, stream)
                .await
                .map_err(|e| TlsException::IoError(e).into())?;

            Ok(TlsSocket(FromTokio::new(stream)))
        }
    }

    impl<S> TlsSocket<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        pub async fn accept(stream: S, url: &Url) -> Result<Self> {
            todo!("Implement accept as TlsListener");
        }
    }

    impl<S> ErrorType for TlsSocket<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        type Error = TlsException;
    }

    impl<S> Read for TlsSocket<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
            self.0
                .read(buf)
                .await
                .map_err(|e| TlsException::IoError(e).into())
        }
    }

    impl<S> Write for TlsSocket<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
            self.0
                .write(buf)
                .await
                .map_err(|e| TlsException::IoError(e).into())
        }

        async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
            self.0
                .flush()
                .await
                .map_err(|e| TlsException::IoError(e).into())
        }
    }
}
