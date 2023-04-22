use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TcpError {
    #[error("Unable to connect to host")]
    UnableToConnect,
}

#[cfg(feature = "std")]
impl alloc::error::Error for TcpError {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TlsError {
    #[error("Tls is not connected (`inner` is not defined)")]
    NotConnected,
    #[error("Failed to establish tls handshake.")]
    FailedToOpen,
}

#[cfg(feature = "std")]
impl alloc::error::Error for TlsError {}
