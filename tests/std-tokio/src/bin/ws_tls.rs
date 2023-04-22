#![feature(type_alias_impl_trait)]

use std::borrow::Cow;
use embedded_tls::{Aes128GcmSha256, TlsConfig};

// TODO: Replace with own integrations
use embedded_websocket::{WebSocketClient, WebSocketCloseStatusCode, WebSocketOptions, WebSocketSendMessageType};
use embedded_websocket::framer_async::{Framer, ReadResult};

use tokio::net;
use em_as_net::core::framed::{Codec, Framed};
use em_as_net::core::tcp::{FromTokio, TcpStream};
use em_as_net::core::tcp::Connect;
use em_as_net::core::tcp::tls::TlsConnection;

#[tokio::main]
async fn main() {
    let stream: TcpStream<net::TcpStream> = TcpStream::new();
    stream.connect(&Cow::from("limpidcrypto.de:6005")).await.unwrap();
    let mut read_record_buffer = [0; 16384];
    let mut write_record_buffer = [0; 16384];
    let mut tls: TlsConnection<FromTokio<TcpStream<net::TcpStream>>, Aes128GcmSha256> = TlsConnection::new();
    tls.open("limpidcrypto.de".into(), FromTokio::new(stream), &mut read_record_buffer, &mut write_record_buffer);

    let mut stream = Framed::new(tls, Codec::new());

    let rng = rand::thread_rng();
    let ws = WebSocketClient::new_client(rng);

    let websocket_options = WebSocketOptions {
        path: "/",
        host: "limpidcrypto.de",
        origin: "http://limpidcrypto.de:6005",
        sub_protocols: None,
        additional_headers: None,
    };

    let mut buffer = [0u8; 4096];
    let mut framer = Framer::new(ws);
    framer.connect(&mut stream, &mut buffer, &websocket_options).await.unwrap();

    // framer
    //     .write(
    //         &mut stream,
    //         &mut buffer,
    //         WebSocketSendMessageType::Text,
    //         true,
    //         r#"{"method": "ping"}"#.as_bytes(),
    //     )
    //     .await.unwrap();

    // while let Some(read_result) = framer.read(&mut stream, &mut buffer).await {
    //     let read_result = read_result.unwrap();
    //     match read_result {
    //         ReadResult::Text(text) => {
    //             let expected = r#"{"result":{},"status":"success","type":"response"}"#;
    //             assert_eq!(expected, text);
    //
    //             framer
    //                 .close(
    //                     &mut stream,
    //                     &mut buffer,
    //                     WebSocketCloseStatusCode::NormalClosure,
    //                     None,
    //                 )
    //                 .await.unwrap()
    //         }
    //         _ => {
    //         }
    //     }
    // }
}