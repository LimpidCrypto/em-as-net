use core::fmt::Debug;
use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WebsocketError {
    #[error("Unable to read.")] // TODO: specify error
    ReadError,
}

#[cfg(feature = "std")]
impl alloc::error::Error for WebsocketError {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AddrsError<'a> {
    #[error(transparent)]
    InvalidFormat(#[from] url::ParseError),
    #[error("Scheme must be either 'ws' or 'wss'. (found: {0:?})")]
    InvalidScheme(&'a str),
}

#[cfg(feature = "std")]
impl alloc::error::Error for AddrsError<'_> {}
