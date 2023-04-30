use std::future::poll_fn;
use std::pin::Pin;

use tokio::net;
use em_as_net::core::io::AsyncWrite;
use em_as_net::core::tcp::{Connect, TcpStream};

#[tokio::main]
async fn main() {
    // Connect to an echo server.
    let mut stream: TcpStream<net::TcpStream> = TcpStream::new();
    stream.connect("tcpbin.com:4242".into()).await.unwrap();
    println!("created stream");
    // write to tcp stream
    poll_fn(|cx| { Pin::new(&mut stream).poll_write(cx, b"Hello, world!") }).await.unwrap();
    println!("sent Hello, world!");
}