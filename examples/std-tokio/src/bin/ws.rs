#![feature(type_alias_impl_trait)]

use std::borrow::Cow;

use em_as_net::client::websocket::{WebsocketClient, ReadResult, WebsocketClientIo};

#[tokio::main]
async fn main() {
    let mut buffer = [0u8; 4096];
    let mut websocket = WebsocketClient::new(Cow::from("ws://limpidcrypto.de:6004"), &mut buffer);
    websocket.connect(None).await;

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

                websocket.close().await
            }
            _ => {
            }
        }
    }
}
