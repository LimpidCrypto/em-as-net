use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FramedException {
    #[error("Failed to write to transport")]
    WriteToTransport,
    #[error("Reached EOF: Path not readable anymore")]
    EOF,
    #[error("Failed to encode message")]
    UnableToEncode,
    #[error("Failed to flush message")]
    FailedToFlush,
    #[error("Unable to decode bytes from stream. Bytes remain on stream")]
    DecodeError,
    #[error("AsyncRead")]
    UnableToRead,
    #[error("AsyncWrite")]
    UnableToWrite,
    #[error("AsyncWrite")]
    UnableToFlush,
    #[error("AsyncWrite")]
    UnableToClose,
}
