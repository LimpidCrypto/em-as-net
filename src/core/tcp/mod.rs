pub mod exceptions;

#[cfg(not(feature = "std"))]
pub use _embassy::*;
#[cfg(feature = "std")]
pub use _tokio::*;

#[cfg(not(feature = "std"))]
mod _embassy {
    use embassy_net::tcp::TcpSocket as EmbassyTcpSocket;

    pub type TcpSocket<'a> = EmbassyTcpSocket<'a>;
}

#[cfg(feature = "std")]
mod _tokio {
    use super::exceptions::TcpException;
    use crate::{utils::derive_tcp_url, Err};
    use anyhow::Result;
    use embedded_io_adapters::tokio_1::FromTokio;
    use embedded_io_async::{ErrorType, Read, Write};
    use tokio::{
        io::{AsyncRead, AsyncWrite},
        net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream},
    };
    use url::Url;

    pub struct TcpStream(FromTokio<TokioTcpStream>);

    impl TcpStream {
        pub async fn connect(url: &Url) -> Result<TcpStream> {
            let stream = TokioTcpStream::connect(derive_tcp_url(url, None)?)
                .await
                .map_err(|e| TcpException::IoError(e).into())?;
            Ok(TcpStream(FromTokio::new(stream)))
        }
    }

    impl ErrorType for TcpStream {
        type Error = TcpException;
    }

    impl Read for TcpStream {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.0.read(buf).await.map_err(|e| TcpException::IoError(e))
        }
    }

    impl Write for TcpStream {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.0
                .write(buf)
                .await
                .map_err(|e| TcpException::IoError(e))
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.0.flush().await.map_err(|e| TcpException::IoError(e))
        }
    }

    impl AsyncRead for TcpStream {
        fn poll_read(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> core::task::Poll<alloc::io::Result<()>> {
            core::pin::Pin::new(self.get_mut().0.inner_mut()).poll_read(cx, buf)
        }
    }

    impl AsyncWrite for TcpStream {
        fn poll_write(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
            buf: &[u8],
        ) -> core::task::Poll<alloc::io::Result<usize>> {
            core::pin::Pin::new(self.get_mut().0.inner_mut()).poll_write(cx, buf)
        }

        fn poll_flush(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<alloc::io::Result<()>> {
            core::pin::Pin::new(self.get_mut().0.inner_mut()).poll_flush(cx)
        }

        fn poll_shutdown(
            self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<alloc::io::Result<()>> {
            core::pin::Pin::new(self.get_mut().0.inner_mut()).poll_shutdown(cx)
        }
    }

    pub struct TcpListener(TokioTcpListener);

    impl TcpListener {
        pub async fn bind(url: &Url) -> Result<TcpListener> {
            match TokioTcpListener::bind(derive_tcp_url(url, None)?).await {
                Ok(listener) => Ok(TcpListener(listener)),
                Err(e) => Err!(TcpException::IoError(e)),
            }
        }

        pub async fn accept(&self) -> Result<(TcpStream, alloc::net::SocketAddr)> {
            match self.0.accept().await {
                Ok((stream, addr)) => Ok((TcpStream(FromTokio::new(stream)), addr)),
                Err(e) => Err!(TcpException::IoError(e)),
            }
        }
    }
}
