use super::async_websocket_client::EmbeddedWebsocketFramerError;
use core::fmt::Debug;
use core::str::Utf8Error;
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

impl<E: Debug> From<EmbeddedWebsocketFramerError<E>> for WebsocketError<E> {
    fn from(value: EmbeddedWebsocketFramerError<E>) -> Self {
        match value {
            EmbeddedWebsocketFramerError::Io(e) => WebsocketError::Io(e),
            EmbeddedWebsocketFramerError::FrameTooLarge(e) => WebsocketError::FrameTooLarge(e),
            EmbeddedWebsocketFramerError::Utf8(e) => WebsocketError::Utf8(e),
            EmbeddedWebsocketFramerError::HttpHeader(_) => WebsocketError::HttpHeader,
            EmbeddedWebsocketFramerError::WebSocket(e) => WebsocketError::WebSocket(e),
            EmbeddedWebsocketFramerError::Disconnected => WebsocketError::Disconnected,
            EmbeddedWebsocketFramerError::RxBufferTooSmall(e) => {
                WebsocketError::RxBufferTooSmall(e)
            }
        }
    }
}

#[cfg(feature = "std")]
impl<E: Debug> alloc::error::Error for WebsocketError<E> {}
