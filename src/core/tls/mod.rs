mod exceptions;

pub use exceptions::*;

use anyhow::Result;
use embedded_io_async::{Read, Write};

#[cfg(not(feature = "std"))]
pub use rustls_stream::*;

#[cfg(not(feature = "std"))]
mod rustls_stream {
    use embedded_io_async::{Read, Write};
    use embedded_websocket::Result;

    pub struct TlsStream<S: Read + Write + Unpin>(S);

    impl<S: Read + Write + Unpin> TlsStream<S> {
        pub async fn connect(stream: S, _url: &url::Url) -> Result<Self> {
            Ok(TlsStream(stream))
        }

        pub async fn accept(_stream: S, _url: &url::Url) -> Result<Self> {
            todo!("Implement accept as TlsListener");
        }
    }
}

#[cfg(feature = "std")]
pub use tokio_tls_stream::*;

#[cfg(feature = "std")]
mod tokio_tls_stream {
    use alloc::{borrow::Cow, string::String, sync::Arc};
    use embedded_io_adapters::tokio_1::FromTokio;
    use embedded_io_async::ErrorType;
    use rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio_rustls::client::TlsStream as TokioRustlsTlsStream;
    use tokio_rustls::{TlsAcceptor, TlsConnector};
    use url::Url;

    use super::*;
    use crate::{utils::ws_to_http, Err};

    pub struct TlsStream<S>(FromTokio<TokioRustlsTlsStream<S>>);

    impl<S> TlsStream<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        pub async fn connect(stream: S, url: &Url) -> Result<TlsStream<S>> {
            let mut root_cert_store = RootCertStore::empty();
            root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();
            let connector = TlsConnector::from(Arc::new(config));
            let url_with_http = ws_to_http(url);
            let dns_name = Self::get_dns_name(&url_with_http)?;
            let server_name = match ServerName::try_from(String::from(dns_name)) {
                Ok(server_name) => server_name,
                Err(e) => return Err!(TlsException::InvalidServerName(e)),
            };

            let stream = match connector.connect(server_name, stream).await {
                Ok(stream) => stream,
                Err(e) => return Err!(TlsException::IoError(e)),
            };

            Ok(TlsStream(FromTokio::new(stream)))
        }

        fn get_dns_name(url: &Url) -> Result<Cow<str>> {
            match url.host_str() {
                Some(host) => Ok(host.into()),
                None => Err!(TlsException::NoDomain),
            }
        }
    }

    impl<S> TlsStream<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        pub async fn accept(_stream: S, _url: &Url) -> Result<Self> {
            todo!("Implement accept as TlsListener");
        }
    }

    impl<S> ErrorType for TlsStream<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        type Error = TlsException;
    }

    impl<S> Read for TlsStream<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
            match self.0.read(buf).await {
                Ok(n) => Ok(n),
                Err(e) => Err(TlsException::IoError(e)),
            }
        }
    }

    impl<S> Write for TlsStream<S>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
            match self.0.write(buf).await {
                Ok(n) => Ok(n),
                Err(e) => Err(TlsException::IoError(e)),
            }
        }

        async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
            match self.0.flush().await {
                Ok(()) => Ok(()),
                Err(e) => Err(TlsException::IoError(e)),
            }
        }
    }

    pub struct TlsListener(TlsAcceptor);

    // impl TlsListener {
    //     pub async fn bind(_url: &Url) -> Result<Self> {
    //         todo!("Implement bind as TlsListener");
    //     }

    //     pub async fn accept(&self, _stream: S) -> Result<TlsStream<S>> {
    //         todo!("Implement accept as TlsListener");
    //     }
    // }
}
