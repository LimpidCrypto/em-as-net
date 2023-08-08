use thiserror_no_std::Error;

#[derive(Debug, Clone, Error)]
pub enum IoError {
    #[error("Failed to encode message while starting send.")]
    EncodeWhileSendError,
    #[error("Failed to flush message. Bytes remain on stream.")]
    FailedToFlush,
    #[error("Unable to decode bytes from stream.")]
    DecodeError,
    #[error("Unable to read from stream. Bytes remain on stream.")]
    DecodeWhileReadError,
    #[error("Tried to write but the stream is not connected.")]
    WriteNotConnected,
    #[error("Tried to flush but the stream is not connected.")]
    FlushNotConnected,
    #[error("Tried to shutdown but the stream is not connected.")]
    ShutdownNotConnected,
    #[error("Tried to read but the stream is not connected.")]
    ReadNotConnected,

    // TlsConnection errors
    #[error("TLS: Tried to write but the stream is not connected.")]
    TlsWriteNotConnected,
    #[error("TLS: Tried to flush but the stream is not connected.")]
    TlsFlushNotConnected,
    #[error("TLS: Tried to shutdown but the stream is not connected.")]
    TlsShutdownNotConnected,
    #[error("TLS: Tried to read but the stream is not connected.")]
    TlsReadNotConnected,

    // FromTokio errors
    #[error("FromTokio: Tried to write but the stream is not connected.")]
    AdapterTokioWriteNotConnected,
    #[error("FromTokio: Tried to flush but the stream is not connected.")]
    AdapterTokioFlushNotConnected,
    #[error("FromTokio: Tried to shutdown but the stream is not connected.")]
    AdapterTokioShutdownNotConnected,
    #[error("FromTokio: Tried to read but the stream is not connected.")]
    AdapterTokioReadNotConnected,

    // AsyncRead errors
    #[error("Error occured while reading from stream")]
    UnableToRead,

    // AsyncWrite errors
    #[error("Error occured while writing to stream")]
    UnableToWrite,
    #[error("Error occured while flushing stream")]
    UnableToFlush,
    #[error("Error occured while closing stream")]
    UnableToClose,

    // embedded_io errors
    #[error("{0:?}")]
    Io(embedded_io::ErrorKind),

    // Tls errors during IO
    #[cfg(feature = "tls")]
    #[error("TLS: {0:?}")]
    TlsRead(embedded_tls::TlsError),
}

impl embedded_io::Error for IoError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            Self::Io(k) => *k,
            _ => embedded_io::ErrorKind::Other,
        }
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for IoError {}
