use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TcpError {
    #[error("Unable to connect to host")]
    UnableToConnect,
}

#[cfg(feature = "std")]
impl alloc::error::Error for TcpError {}

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("Tls is not connected (`inner` is not defined)")]
    NotConnected,
    #[error("Failed to establish tls handshake.")]
    FailedToOpen,
    #[error("{0:?}")]
    Other(embedded_tls::TlsError)
}

#[cfg(feature = "std")]
impl alloc::error::Error for TlsError {}
