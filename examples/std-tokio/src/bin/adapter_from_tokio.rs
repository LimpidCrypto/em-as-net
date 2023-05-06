use embedded_io::asynch::{Read, Write};

use tokio::net;
use em_as_net::core::tcp::{Connect, TcpStream};
use em_as_net::core::tls::{FromTokio, TlsConnection};

#[tokio::main]
async fn main() {
    // Connect to an echo server.
    let stream: TcpStream<net::TcpStream> = TcpStream::new();
    let mut adapter = FromTokio::new(stream);
    adapter.connect("tcpbin.com:4242".into()).await.unwrap();


    // Write to stream
    adapter.write(b"Hello!").await.unwrap();
    println!("written");
    adapter.flush().await.unwrap();
    println!("flushed");

    let mut buffer = [0u8; 4096];
    while let Ok(s) = adapter.read(&mut buffer).await {
        println!("{:?}", s);
        adapter.shutdown().await.unwrap();
        println!("shutdown");
        break;
    }
}
