use std::future::poll_fn;
use std::pin::Pin;
use std::task::Poll;
use embedded_io::asynch::{Read, Write};

use tokio::net;
use em_as_net::core::framed::IoError;
use em_as_net::core::io::{AsyncRead, AsyncWrite};
use em_as_net::core::tcp::{TcpConnect, TcpSocket};
use em_as_net::core::tcp::adapters::TcpTokio;

#[tokio::main]
async fn main() {
    // Connect to an echo server.
    let mut socket = TcpSocket::<TcpTokio>::connect("tcpbin.com:4242".into()).await.unwrap();
    // write to echo server
    poll_fn(|cx| { Pin::new(&mut socket).poll_write(cx, b"Hello, world") }).await.unwrap();
    // flush buffer
    poll_fn(|cx| { Pin::new(&mut socket).poll_flush(cx) }).await.unwrap();
    // read from stream
    let mut buffer = [0u8; 4096];
    let mut read_buffer = tokio::io::ReadBuf::new(&mut buffer);
    poll_fn(|cx| { Pin::new(&mut socket).poll_read(cx, &mut read_buffer) }).await.unwrap();
    // close connection
    poll_fn(|cx| { Pin::new(&mut socket).poll_shutdown(cx) }).await.unwrap();
}
