use alloc::borrow::Cow;

pub use exceptions::*;
#[cfg(feature = "std")]
pub use std_tcp::TcpStream;

pub mod tls;
pub mod exceptions;

pub trait Connect<'a> {
    type Error;

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

    use crate::core::framed::FramedException;
    use crate::core::io;

    use super::Connect;
    use super::exceptions::TcpException;

    pub struct TcpStream<T> {
        stream: RefCell<Option<T>>,
    }

    impl<T> TcpStream<T> {
        pub fn new() -> Self {
            Self {
                stream: RefCell::new(None),
            }
        }
    }

    impl<'a> Connect<'a> for TcpStream<net::TcpStream> {
        type Error = TcpException;

        async fn connect(&self, ip: &Cow<'a, str>) -> Result<(), TcpException> {
            let result = net::TcpStream::connect(&**ip).await;

            match result {
                Ok(tcp_stream) => {
                    self.stream
                        .replace(Some(tcp_stream));
                    Ok(())
                }
                Err(_) => {
                    Err(TcpException::UnableToConnect)
                }
            }
        }
    }

    impl io::AsyncRead for TcpStream<net::TcpStream> {
        type Error = FramedException;

        fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<Result<(), Self::Error>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(FramedException::UnableToRead))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_read(cx, buf) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(_) => {Poll::Ready(Ok(()))}
                                Err(_) => {Poll::Ready(Err(FramedException::UnableToRead))}
                            }
                        },
                        Poll::Pending => {return Poll::Pending;}
                    }
                }
            }
        }
    }

    impl io::AsyncWrite for TcpStream<net::TcpStream> {
        fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, FramedException>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(FramedException::UnableToWrite))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_write(cx, buf) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(ok) => {Poll::Ready(Ok(ok))}
                                Err(_) => {Poll::Ready(Err(FramedException::UnableToWrite))}
                            }
                        }
                        Poll::Pending => {return Poll::Pending;}
                    }
                }
            }
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), FramedException>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(FramedException::UnableToFlush))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_flush(cx) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(ok) => {Poll::Ready(Ok(ok))}
                                Err(_) => {Poll::Ready(Err(FramedException::UnableToFlush))}
                            }
                        }
                        Poll::Pending => {return Poll::Pending;}
                    }
                }
            }
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), FramedException>> {
            match self.stream.borrow_mut().as_mut() {
                None => {Poll::Ready(Err(FramedException::UnableToClose))}
                Some(stream) => {
                    match Pin::new(stream).borrow_mut().as_mut().poll_shutdown(cx) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(ok) => {Poll::Ready(Ok(ok))}
                                Err(_) => {Poll::Ready(Err(FramedException::UnableToClose))}
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

