// use tokio::net;
// use em_as_net::core::tcp::{TcpConnect, TcpSocket};
// use em_as_net::core::tcp::adapters::TcpAdapterTokio;
// use em_as_net::core::tls::{TlsConnection, Aes256GcmSha384, Aes128GcmSha256};

#[tokio::main]
async fn main() {
    // let tokio_adapter = TcpAdapterTokio::new();
    // let tcp_socket = TcpSocket::new(tokio_adapter);
    // let mut read_record_buffer = [0u8; 17000];
    // let mut write_record_buffer = [0u8; 17000];
    // let tls_connection: TlsConnection<TcpSocket<TcpAdapterTokio>, Aes256GcmSha384> = TlsConnection::new(tcp_socket, &mut read_record_buffer, &mut write_record_buffer);
    //
    // tls_connection.connect("104.131.203.210:443".into()).await.unwrap();
    // adapter.connect().await.unwrap();
    //
    //
    // // Write to stream
    // adapter.write(b"Hello!").await.unwrap();
    // println!("written");
    // adapter.flush().await.unwrap();
    // println!("flushed");
    //
    // while let Ok(s) = adapter.read(&mut buffer).await {
    //     println!("{:?}", s);
    //     adapter.shutdown().await.unwrap();
    //     println!("shutdown");
    //     break;
    // }
}
