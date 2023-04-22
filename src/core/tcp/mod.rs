use alloc::borrow::Cow;
use core::fmt::{Debug, Display};

pub use errors::*;
#[cfg(all(feature = "std", feature = "tls"))]
pub use std_tcp::FromTokio;
#[cfg(feature = "std")]
pub use std_tcp::TcpStream;

pub mod tls;
pub mod errors;

pub trait Connect<'a> {
    type Error: Debug + Display;

    async fn connect(&self, ip: &Cow<'a, str>) -> Result<(), Self::Error>;
}

#[cfg(feature = "std")]
mod std_tcp {
    use alloc::borrow::Cow;
    use core::borrow::BorrowMut;
    use core::cell::RefCell;
    use core::pin::Pin;
    use core::task::{Context, Poll};

    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::net;

    #[cfg(feature = "tls")]
    pub use adapters::FromTokio;

    use crate::core::framed::IoError;
    use crate::core::io;

    use super::Connect;
    use super::errors::TcpError;

    pub struct TcpStream<T> {
        pub(crate) stream: RefCell<Option<T>>,
    }

    impl<T> TcpStream<T> {
        pub fn new() -> Self {
            Self {
                stream: RefCell::new(None),
            }
        }
    }

    impl<'a> Connect<'a> for TcpStream<net::TcpStream> {
        type Error = TcpError;

        async fn connect(&self, ip: &Cow<'a, str>) -> Result<(), TcpError> {
            let result = net::TcpStream::connect(&**ip).await;

            match result {
                Ok(tcp_stream) => {
                    self.stream
                        .replace(Some(tcp_stream));
                    Ok(())
                }
                Err(_) => {
                    Err(TcpError::UnableToConnect)
                }
            }
        }
    }

    impl io::AsyncRead for TcpStream<net::TcpStream> {
        type Error = IoError;

        fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<Result<(), Self::Error>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(IoError::ReadNotConnected))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_read(cx, buf) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(_) => {Poll::Ready(Ok(()))}
                                Err(_) => {Poll::Ready(Err(IoError::UnableToRead))}
                            }
                        },
                        Poll::Pending => {return Poll::Pending;}
                    }
                }
            }
        }
    }

    impl io::AsyncWrite for TcpStream<net::TcpStream> {
        fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(IoError::WriteNotConnected))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_write(cx, buf) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(ok) => {Poll::Ready(Ok(ok))}
                                Err(_) => {Poll::Ready(Err(IoError::UnableToWrite))}
                            }
                        }
                        Poll::Pending => {return Poll::Pending;}
                    }
                }
            }
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(IoError::FlushNotConnected))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_flush(cx) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(ok) => {Poll::Ready(Ok(ok))}
                                Err(_) => {Poll::Ready(Err(IoError::UnableToFlush))}
                            }
                        }
                        Poll::Pending => {return Poll::Pending;}
                    }
                }
            }
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(IoError::ShutdownNotConnected))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_shutdown(cx) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(ok) => {Poll::Ready(Ok(ok))}
                                Err(_) => {Poll::Ready(Err(IoError::UnableToClose))}
                            }
                        }
                        Poll::Pending => {return Poll::Pending;}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test_stream {
    use alloc::borrow::Cow;

    use embedded_websocket::{WebSocketClient, WebSocketCloseStatusCode, WebSocketOptions, WebSocketSendMessageType};
    use embedded_websocket::framer_async::{Framer, ReadResult};
    use rand_core::RngCore;
    use tokio::net;
    use tokio::runtime::Runtime;

    use crate::core::tcp::Connect;

    use super::super::framed::{codec::Codec, Framed};
    use super::TcpStream;

    #[test]
    fn test() {
        async fn main_task() {
            // define an empty `TcpStream`
            let stream: TcpStream<net::TcpStream> = TcpStream::new();
            stream.connect(&Cow::from("ws.vi-server.org:80")).await.unwrap();
            let mut stream = Framed::new(stream, Codec::new());

            let rng = rand::thread_rng();
            let ws = WebSocketClient::new_client(rng);

            let websocket_options = WebSocketOptions {
                path: "/mirror",
                host: "ws.vi-server.org",
                origin: "http://ws.vi-server.org:80",
                sub_protocols: None,
                additional_headers: None,
            };

            let mut buffer = [0u8; 4096];
            let mut framer = Framer::new(ws);
            framer.connect(&mut stream, &mut buffer, &websocket_options).await.unwrap();

            framer
                .write(
                    &mut stream,
                    &mut buffer,
                    WebSocketSendMessageType::Text,
                    true,
                    "Hello, world".as_bytes(),
                )
                .await.unwrap();

            while let Some(read_result) = framer.read(&mut stream, &mut buffer).await {
                let read_result = read_result.unwrap();
                match read_result {
                    ReadResult::Text(text) => {
                        framer
                            .close(
                                &mut stream,
                                &mut buffer,
                                WebSocketCloseStatusCode::NormalClosure,
                                None,
                            )
                            .await.unwrap()
                    }
                    _ => { // ignore other kinds of messages
                    }
                }
            }
        }

        let runtime = Runtime::new().unwrap();
        runtime.block_on(main_task())
    }
}

