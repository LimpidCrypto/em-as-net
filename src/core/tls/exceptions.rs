use anyhow::anyhow;
use embedded_io_async::ErrorKind;
use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum TlsException {
    #[error("I/O error: {0}")]
    IoError(alloc::io::Error),
    #[error("No domain")]
    NoDomain,
    #[error("Embedded IO async error")]
    EmbeddedIoAsyncError(ErrorKind),
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
            TlsException::IoError(_) => ErrorKind::Other,
            TlsException::NoDomain => ErrorKind::Other,
            TlsException::EmbeddedIoAsyncError(e) => *e,
        }
    }
}
