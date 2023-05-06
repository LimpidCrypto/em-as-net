use core::fmt::Debug;
use core::str::Utf8Error;
use embedded_websocket::framer_async::FramerError;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum WebsocketError<E: Debug> {
    #[error("Stream is not connected.")]
    NotConnected,

    // FramerError
    #[error("I/O error: {0:?}")]
    Io(E),
    #[error("Frame too large (size: {0:?})")]
    FrameTooLarge(usize),
    #[error("Failed to interpret u8 to string (error: {0:?})")]
    Utf8(Utf8Error),
    #[error("Invalid HTTP header")]
    HttpHeader,
    #[error("Websocket error: {0:?}")]
    WebSocket(embedded_websocket::Error),
    #[error("Disconnected")]
    Disconnected,
    #[error("Read buffer is too small (size: {0:?})")]
    RxBufferTooSmall(usize),
}

impl<E: Debug> From<FramerError<E>> for WebsocketError<E> {
    fn from(value: FramerError<E>) -> Self {
        match value {
            FramerError::Io(e) => {WebsocketError::Io(e)}
            FramerError::FrameTooLarge(e) => {WebsocketError::FrameTooLarge(e)}
            FramerError::Utf8(e) => {WebsocketError::Utf8(e)}
            FramerError::HttpHeader(_) => {WebsocketError::HttpHeader}
            FramerError::WebSocket(e) => {WebsocketError::WebSocket(e)}
            FramerError::Disconnected => {WebsocketError::Disconnected}
            FramerError::RxBufferTooSmall(e) => {WebsocketError::RxBufferTooSmall(e)}
        }
    }
}

#[cfg(feature = "std")]
impl<E: Debug> alloc::error::Error for WebsocketError<E> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AddrsError<'a> {
    #[error(transparent)]
    InvalidFormat(#[from] url::ParseError),
    #[error("Scheme must be either 'ws' or 'wss'. (found: {0:?})")]
    InvalidScheme(&'a str),
}

#[cfg(feature = "std")]
impl alloc::error::Error for AddrsError<'_> {}
