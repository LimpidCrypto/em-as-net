use em_as_net::{
    client::websocket::{
        AsyncWebSocketClient, ReadResult, WebSocketRead, WebSocketSendMessageType, WebSocketWrite,
    },
    core::{tcp::TcpStream, tls::TlsStream},
};
use rand::thread_rng;
use url::Url;

#[tokio::main]
async fn main() {
    let uri = Url::parse("wss://ws.vi-server.org:443/mirror/").unwrap();
    let stream = TcpStream::connect(&uri).await.unwrap();
    println!("TCP Connected");
    let mut tls_stream = TlsStream::connect(stream, &uri).await.unwrap();
    println!("TLS Handshake Done");
    let mut buffer = [0u8; 4096];
    let rng = thread_rng();
    let mut websocket =
        AsyncWebSocketClient::open(&mut tls_stream, &mut buffer, &uri, rng, None, None)
            .await
            .unwrap();
    println!("WebSocket Connected");
    websocket
        .write(
            &mut tls_stream,
            &mut buffer,
            WebSocketSendMessageType::Text,
            true,
            "Hello World".as_bytes(),
        )
        .await
        .unwrap();
    println!("Message Sent");
    loop {
        let message = websocket
            .try_read(&mut tls_stream, &mut buffer)
            .await
            .unwrap()
            .unwrap();
        match message {
            ReadResult::Text(text) => {
                assert_eq!("Hello World".to_string(), text);
                println!("Received message: {}", text);
            }
            _ => panic!("Expected 'Hello World' as text message."),
        }
        break;
    }
}
