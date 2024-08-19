mod constants;

pub use constants::*;
use em_as_net::client::websocket::{AsyncWebSocketClient, WebSocketOpen};
use embedded_io_async::{Read, Write};
use rand::{rngs::ThreadRng, thread_rng};
use url::Url;

pub async fn connect_ws<S: Read + Write + Unpin>(
    uri: &Url,
    stream: &mut S,
    buffer: &mut [u8],
) -> AsyncWebSocketClient<ThreadRng, WebSocketOpen> {
    let rng = thread_rng();

    let websocket = AsyncWebSocketClient::open(stream, buffer, uri, rng, None, None)
        .await
        .unwrap();

    assert!(websocket.is_open());

    websocket
}
