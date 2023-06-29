use std::borrow::Cow;

use em_as_net::client::websocket::{WebsocketClient, ReadResult, WebsocketClientIo, WebsocketClientConnect, WebsocketOptions};
use em_as_net::core::tcp::TcpSocket;
use em_as_net::core::tcp::adapters::TcpAdapterTokio;

#[tokio::main]
async fn main() {
    let mut buffer = [0u8; 4096];
    let mut websocket = WebsocketClient::new(Cow::from("ws://ws.vi-server.org/mirror/"), &mut buffer);

    let tokio_adapter = TcpAdapterTokio::new();
    let tcp_socket = TcpSocket::new(tokio_adapter);
    let rng = rand::thread_rng();

    websocket.connect(tcp_socket, None, rng).await.unwrap();

    websocket
        .write(
            "Hello World".into(),
            None,
        )
        .await.unwrap();

    while let Some(Ok(read_result)) = websocket.read().await {
        match read_result {
            ReadResult::Text(text) => {
                let expected = "Hello World";
                assert_eq!(expected, text);

                websocket.close().await.unwrap()
            }
            _ => {
            }
        }
    }
}
