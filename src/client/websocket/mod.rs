pub mod errors;

use crate::core::framed::{Codec, Framed};
use crate::core::tcp::TcpStream;
use alloc::borrow::Cow;
use anyhow::Result;
use core::cell::RefCell;
use embedded_websocket::framer_async::Framer;
pub use embedded_websocket::{framer_async::ReadResult, WebSocketOptions};
use embedded_websocket::{Client, WebSocketSendMessageType};
use rand_core::RngCore;

pub struct WebsocketClient<'a, T, U: RngCore> {
    uri: Cow<'a, str>,
    buffer: &'a mut [u8],
    stream: RefCell<Option<Framed<TcpStream<T>, Codec>>>,
    framer: RefCell<Option<Framer<U, Client>>>,
}

#[cfg(feature = "std")]
mod if_std {
    use crate::core::framed::{Codec, Framed};
    use crate::core::tcp::{Connect, TcpStream};
    use alloc::borrow::{Cow, ToOwned};
    use anyhow::Result;
    use core::cell::RefCell;
    use embedded_websocket::framer_async::{Framer, ReadResult};
    use embedded_websocket::{WebSocketCloseStatusCode, WebSocketSendMessageType};
    use tokio::net;

    use crate::client::websocket::errors::WebsocketError;
    use crate::client::websocket::{WebsocketClient, WebsocketClientIo};
    use crate::Err;
    use embedded_websocket::WebSocketOptions;
    use rand::rngs::ThreadRng;

    impl<'a> WebsocketClient<'a, net::TcpStream, ThreadRng> {
        pub fn new(uri: Cow<'a, str>, buffer: &'a mut [u8]) -> Self {
            Self {
                uri,
                buffer,
                stream: RefCell::new(None),
                framer: RefCell::new(None),
            }
        }

        pub async fn connect(&mut self, options: WebSocketOptions<'a>) {
            // Connect TCP
            let tcp_stream: TcpStream<net::TcpStream> = TcpStream::new();
            tcp_stream.connect(self.uri.to_owned()).await.unwrap(); // TODO: handle error
            let framed = Framed::new(tcp_stream, Codec::new());
            self.stream.replace(Some(framed));

            let rng = rand::thread_rng();
            let ws_client = embedded_websocket::WebSocketClient::new_client(rng);
            // Connect Websocket
            let mut framer = Framer::new(ws_client);
            framer
                .connect(
                    &mut self.stream.borrow_mut().as_mut().unwrap(),
                    self.buffer,
                    &options,
                )
                .await
                .unwrap();

            self.framer.replace(Some(framer));
        }
    }

    impl<'a> WebsocketClientIo<'a> for WebsocketClient<'a, net::TcpStream, ThreadRng> {
        async fn read(&mut self) -> Option<Result<ReadResult<'_>>> {
            let read_result = self
                .framer
                .borrow_mut()
                .as_mut()
                .unwrap()
                .read(
                    // TODO: handle unwrap
                    self.stream.borrow_mut().as_mut().unwrap(),
                    self.buffer,
                )
                .await;

            match read_result {
                Some(Err(err)) => Some(Err!(WebsocketError::<anyhow::Error>::from(err))),
                Some(Ok(read_res)) => Some(Ok(read_res)),
                None => None,
            }
        }

        async fn write(
            &mut self,
            message: Cow<'a, str>,
            send_msg_type: Option<WebSocketSendMessageType>,
        ) -> Result<()> {
            return match self.framer
                .borrow_mut()
                .as_mut()
                .unwrap()
                .write(
                    // TODO: handle unwrap0
                    match self.stream.borrow_mut().as_mut() {
                        None => { return Err!(WebsocketError::<anyhow::Error>::NotConnected); },
                        Some(stream) => stream
                    },
                    self.buffer,
                    send_msg_type.unwrap_or(WebSocketSendMessageType::Text),
                    true,
                    message.as_ref().as_bytes(),
                )
                .await {
                Ok(()) => Ok(()),
                Err(err) => Err!(WebsocketError::from(err))
            }
        }

        async fn close(&mut self) {
            self.framer
                .borrow_mut()
                .as_mut()
                .unwrap()
                .close(
                    // TODO: handle unwrap
                    self.stream.borrow_mut().as_mut().unwrap(),
                    self.buffer,
                    WebSocketCloseStatusCode::NormalClosure,
                    None,
                )
                .await
                .unwrap() // TODO: Return `Result`
        }
    }
}

pub trait WebsocketClientIo<'a> {
    async fn read(&mut self) -> Option<Result<ReadResult<'_>>>;
    async fn write(
        &mut self,
        message: Cow<'a, str>,
        send_msg_type: Option<WebSocketSendMessageType>,
    ) -> Result<()>;
    async fn close(&mut self);
}
