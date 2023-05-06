use embedded_tls::Aes128GcmSha256;

use tokio::net;
use em_as_net::core::tcp::{TcpStream};
use em_as_net::core::tls::{FromTokio, TlsConnection};

#[tokio::main]
async fn main() {
    let mut read_buffer = [0u8; 16384];
    let mut write_buffer = [0u8; 16384];

    let stream: TcpStream<net::TcpStream> = TcpStream::new();
    let adapter = FromTokio::new(stream);
    let tls: TlsConnection<FromTokio<TcpStream<net::TcpStream>>, Aes128GcmSha256> = TlsConnection::new();
    tls.open("limpidcrypto.de:6004".into(), adapter, &mut read_buffer, &mut write_buffer).await.unwrap();
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
