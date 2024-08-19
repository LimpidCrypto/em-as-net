use anyhow::anyhow;
use embedded_io_async::ErrorKind;
use rustls::pki_types::InvalidDnsNameError;
use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum TlsException {
    #[error("I/O error: {0}")]
    IoError(alloc::io::Error),
    #[error("No domain")]
    NoDomain,
    #[error("Embedded IO async error")]
    EmbeddedIoAsyncError(ErrorKind),
    #[error("Invalid server name: {0}")]
    InvalidServerName(InvalidDnsNameError),
}

impl From<alloc::io::Error> for TlsException {
    fn from(e: alloc::io::Error) -> Self {
        TlsException::IoError(e)
    }
}

impl Into<anyhow::Error> for TlsException {
    fn into(self) -> anyhow::Error {
        anyhow!(self)
    }
}

impl embedded_io_async::Error for TlsException {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            TlsException::EmbeddedIoAsyncError(e) => *e,
            _ => ErrorKind::Other,
        }
    }
}
