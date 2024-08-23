use core::fmt::Debug;

use embedded_io_async::ErrorKind;
use rustls::pki_types::InvalidDnsNameError;
use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum TlsException {
    #[cfg(not(feature = "std"))]
    #[error("I/O error")]
    IoError,
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    IoError(alloc::io::Error),
    #[error("No domain")]
    NoDomain,
    #[error("Embedded IO async error")]
    EmbeddedIoAsyncError(ErrorKind),
    #[error("Invalid server name: {0}")]
    InvalidServerName(InvalidDnsNameError),
}

impl embedded_io_async::Error for TlsException {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            TlsException::EmbeddedIoAsyncError(e) => *e,
            _ => ErrorKind::Other,
        }
    }
}
