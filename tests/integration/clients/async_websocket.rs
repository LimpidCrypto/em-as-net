use crate::common::connect_to_embedded_websocket_tokio_ws_echo;
use crate::common::{
    codec::Codec, connect_to_tungstenite_wss_echo, connect_to_ws_tungstenite_echo,
    ECHO_WS_AS_IP_SERVER,
};

use em_as_net::client::websocket::{
    EmbeddedWebsocketOptions, EmbeddedWebsocketReadMessageType, EmbeddedWebsocketSendMessageType,
    TungsteniteMessage,
};
use futures::{SinkExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::test]
async fn test_websocket_non_tls() {
    let mut websocket = connect_to_ws_tungstenite_echo().await;
    websocket
        .send(TungsteniteMessage::Text("Hello World".to_string()))
        .await
        .unwrap();

    loop {
        let message = websocket.try_next().await.unwrap().unwrap();
        match message {
            TungsteniteMessage::Text(text) => {
                assert_eq!("Hello World".to_string(), text)
            }
            _ => panic!("Expected 'Hello World' as text message."),
        }
        break;
    }
}

#[tokio::test]
async fn test_websocket_tls() {
    let mut websocket = connect_to_tungstenite_wss_echo().await;
    websocket
        .send(TungsteniteMessage::Text("Hello World".to_string()))
        .await
        .unwrap();

    loop {
        let message = websocket.try_next().await.unwrap().unwrap();
        match message {
            TungsteniteMessage::Text(text) => {
                assert_eq!("Hello World".to_string(), text)
            }
            _ => panic!("Expected 'Hello World' as text message."),
        }
        break;
    }
}

#[tokio::test]
async fn test_websocket_embedded_ws_tokio() {
    let stream = TcpStream::connect(ECHO_WS_AS_IP_SERVER).await.unwrap();
    let mut framed = Framed::new(stream, Codec::new());
    let mut buffer = [0u8; 4096];
    let websocket_options = EmbeddedWebsocketOptions {
        path: "/mirror",
        host: "ws.vi-server.org",
        origin: "http://ws.vi-server.org:80",
        sub_protocols: None,
        additional_headers: None,
    };
    let mut websocket =
        connect_to_embedded_websocket_tokio_ws_echo(&mut framed, &mut buffer, &websocket_options)
            .await;
    websocket
        .send(
            &mut framed,
            &mut buffer,
            EmbeddedWebsocketSendMessageType::Binary,
            false,
            b"Hello World",
        )
        .await
        .unwrap();

    loop {
        let message = websocket
            .next(&mut framed, &mut buffer)
            .await
            .unwrap()
            .unwrap();
        match message {
            EmbeddedWebsocketReadMessageType::Text(text) => {
                println!("Text: {:?}", text)
            }
            EmbeddedWebsocketReadMessageType::Binary(msg) => {
                assert_eq!(b"Hello World", msg);
                break;
            }
            EmbeddedWebsocketReadMessageType::Pong(t) => println!("Pong: {:?}", t),
            EmbeddedWebsocketReadMessageType::Ping(t) => println!("Ping: {:?}", t),
            EmbeddedWebsocketReadMessageType::Close(_) => println!("Close:"),
        }
    }
}
