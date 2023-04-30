use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("Tls is not connected (`inner` is not defined)")]
    NotConnected,
    #[error("Failed to establish tls handshake.")]
    FailedToOpen,
    #[error("{0:?}")]
    Other(embedded_tls::TlsError),
}

#[cfg(feature = "std")]
impl alloc::error::Error for TlsError {}
