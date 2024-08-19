use embedded_io_async::ErrorKind;
use strum_macros::Display;
use thiserror_no_std::Error;

#[derive(Debug, Error, Display)]
pub enum TcpException {
    IoError(#[from] alloc::io::Error),
    EmbeddedIoAsyncError(embedded_io_async::ErrorKind),
}

impl embedded_io_async::Error for TcpException {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            TcpException::EmbeddedIoAsyncError(e) => *e,
            _ => ErrorKind::Other,
        }
    }
}

impl Into<anyhow::Error> for TcpException {
    fn into(self) -> anyhow::Error {
        anyhow::anyhow!(self)
    }
}
