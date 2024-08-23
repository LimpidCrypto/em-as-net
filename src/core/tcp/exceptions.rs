use core::fmt::Debug;

use embedded_io_async::ErrorKind;
use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum TcpException {
    #[cfg(not(feature = "std"))]
    #[error("I/O error")]
    IoError,
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    IoError(alloc::io::Error),
    #[error("Embedded IO async error")]
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
