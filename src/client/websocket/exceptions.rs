use anyhow::anyhow;
use core::fmt::Debug;
use core::str::Utf8Error;
use embedded_websocket::framer_async::FramerError;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum WebSocketException<E: Debug> {
    #[error("Invalid domain")]
    InvalidDomain,
    #[error("Invalid scheme")]
    InvalidScheme,
    // FramerError
    #[error("I/O error: {0:?}")]
    Io(E),
    #[error("Frame too large (size: {0:?})")]
    FrameTooLarge(usize),
    #[error("Failed to interpret u8 to string (error: {0:?})")]
    Utf8(Utf8Error),
    #[error("Invalid HTTP header")]
    HttpHeader,
    #[error("WebSocket error: {0:?}")]
    WebSocket(embedded_websocket::Error),
    #[error("Disconnected")]
    Disconnected,
    #[error("Read buffer is too small (size: {0:?})")]
    RxBufferTooSmall(usize),
}

impl<E: Debug> From<FramerError<E>> for WebSocketException<E> {
    fn from(e: FramerError<E>) -> Self {
        match e {
            FramerError::Io(e) => WebSocketException::Io(e),
            FramerError::FrameTooLarge(size) => WebSocketException::FrameTooLarge(size),
            FramerError::Utf8(e) => WebSocketException::Utf8(e),
            FramerError::HttpHeader(_) => WebSocketException::HttpHeader,
            FramerError::WebSocket(e) => WebSocketException::WebSocket(e),
            FramerError::Disconnected => WebSocketException::Disconnected,
            FramerError::RxBufferTooSmall(size) => WebSocketException::RxBufferTooSmall(size),
        }
    }
}

impl<E: Debug> Into<anyhow::Error> for WebSocketException<E> {
    fn into(self) -> anyhow::Error {
        anyhow!(self)
    }
}

#[cfg(feature = "std")]
impl<E: Debug> alloc::error::Error for WebSocketException<E> {}
