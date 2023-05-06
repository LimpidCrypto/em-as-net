use std::future::poll_fn;
use std::pin::Pin;
use std::task::Poll;
use embedded_io::asynch::{Read, Write};

use tokio::net;
use em_as_net::core::framed::IoError;
use em_as_net::core::io::{AsyncRead, AsyncWrite};
use em_as_net::core::tcp::{Connect, TcpStream};

#[tokio::main]
async fn main() {
    // Connect to an echo server.
    let mut stream: TcpStream<net::TcpStream> = TcpStream::new();
    stream.connect("tcpbin.com:4242".into()).await.unwrap();
    // write to echo server
    poll_fn(|cx| match Pin::new(&mut stream).poll_write(cx, b"Hello, world") {
        Poll::Ready(r) => {
            Poll::Ready(r)
        }
        Poll::Pending => Poll::Pending
    })
        .await.unwrap();
    println!("written");
    // flush buffer
    poll_fn(|cx| match Pin::new(&mut stream).poll_flush(cx) {
        Poll::Ready(r) => {
            Poll::Ready(r)
        }
        Poll::Pending => Poll::Pending
    })
        .await.unwrap();
    println!("flushed");
    // read from stream
    let mut buffer = [0u8; 4096];
    let mut read_buffer = tokio::io::ReadBuf::new(&mut buffer);
    let size = poll_fn(|cx| match Pin::new(&mut stream).poll_read(cx, &mut read_buffer) {
        Poll::Ready(r) => {
            Poll::Ready(r)
        }
        Poll::Pending => Poll::Pending
    })
        .await.unwrap();
    // println!("created stream");
    //
    // // write to tcp stream
    // poll_fn(|cx| { Pin::new(&mut stream).poll_write(cx, b"Hello, world!") }).await.unwrap();
    // println!("sent Hello, world!");
}
