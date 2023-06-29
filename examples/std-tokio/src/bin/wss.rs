use std::borrow::Cow;
use em_as_net::client::websocket::{WebsocketClient, WebsocketClientConnect};
use em_as_net::core::tcp::adapters::TcpAdapterTokio;
use em_as_net::core::tcp::TcpSocket;
use em_as_net::core::tls::{TlsConnection, Aes128GcmSha256};

#[tokio::main]
async fn main() {
    let mut buffer = [0u8; 4096];
    let mut websocket = WebsocketClient::new(Cow::from("wss://ws.vi-server.org/mirror/"), &mut buffer);

    let tokio_adapter = TcpAdapterTokio::new();
    let tcp_socket = TcpSocket::new(tokio_adapter);
    let mut read_record_buffer = [0u8; 16384];
    let mut write_record_buffer = [0u8; 16384];
    let tls_connection: TlsConnection<TcpSocket<TcpAdapterTokio>, Aes128GcmSha256> = TlsConnection::new(tcp_socket, &mut read_record_buffer, &mut write_record_buffer);
    let rng = rand::thread_rng();
    websocket.connect(tls_connection, None, rng).await.unwrap();
}
