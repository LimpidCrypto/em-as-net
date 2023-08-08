pub mod codec;
mod constants;

pub use constants::*;
use em_as_net::client::websocket::{
    AsyncWebsocketClientEmbeddedWebsocketTokio, AsyncWebsocketClientTungstenite,
    EmbeddedWebsocketOptions, WebsocketOpen,
};
use rand::{rngs::ThreadRng, thread_rng};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub async fn connect_to_ws_tungstenite_echo<'a>() -> AsyncWebsocketClientTungstenite<WebsocketOpen>
{
    let websocket = AsyncWebsocketClientTungstenite::open(ECHO_WS_SERVER.parse().unwrap())
        .await
        .unwrap();
    assert!(websocket.is_open());

    websocket
}

pub async fn connect_to_tungstenite_wss_echo<'a>() -> AsyncWebsocketClientTungstenite<WebsocketOpen>
{
    let websocket = AsyncWebsocketClientTungstenite::open(ECHO_WSS_SERVER.parse().unwrap())
        .await
        .unwrap();
    assert!(websocket.is_open());

    websocket
}

pub async fn connect_to_embedded_websocket_tokio_ws_echo<'a>(
    stream: &'a mut Framed<TcpStream, codec::Codec>,
    buffer: &'a mut [u8],
    websocket_options: &'a EmbeddedWebsocketOptions<'a>,
) -> AsyncWebsocketClientEmbeddedWebsocketTokio<ThreadRng, WebsocketOpen> {
    let rng = thread_rng();

    let websocket =
        AsyncWebsocketClientEmbeddedWebsocketTokio::open(stream, buffer, rng, websocket_options)
            .await
            .unwrap();

    assert!(websocket.is_open());

    websocket
}
