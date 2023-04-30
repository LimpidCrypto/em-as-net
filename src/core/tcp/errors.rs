use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TcpError {
    #[error("Unable to connect to host")]
    UnableToConnect,
}

#[cfg(feature = "std")]
impl alloc::error::Error for TcpError {}
