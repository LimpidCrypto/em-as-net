use std::borrow::Cow;
use rand::rngs::ThreadRng;

use em_as_net::client::websocket::{WebsocketClient, ReadResult, WebsocketClientIo, WebsocketClientConnect};
use em_as_net::core::tcp::TcpSocket;
use em_as_net::core::tcp::adapters::TcpAdapterTokio;

#[tokio::main]
async fn main() {
    let mut buffer = [0u8; 4096];
    let mut websocket: WebsocketClient<TcpSocket<TcpAdapterTokio>, ThreadRng> = WebsocketClient::new(Cow::from("ws://limpidcrypto.de:6004"), &mut buffer);

    let tokio_adapter = TcpAdapterTokio::new();
    let tcp_socket = TcpSocket::new(tokio_adapter);
    let rng = rand::thread_rng();
    websocket.connect(tcp_socket, None, rng).await.unwrap();

    websocket
        .write(
            r#"{"method": "ping"}"#.into(),
            None,
        )
        .await.unwrap();

    while let Some(Ok(read_result)) = websocket.read().await {
        match read_result {
            ReadResult::Text(text) => {
                let expected = r#"{"result":{},"status":"success","type":"response"}"#;
                assert_eq!(expected, text);

                websocket.close().await.unwrap()
            }
            _ => {
            }
        }
    }
}
