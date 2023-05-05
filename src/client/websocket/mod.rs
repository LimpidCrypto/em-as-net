pub mod errors;

pub use embedded_websocket::{WebSocketOptions, framer_async::ReadResult};

#[cfg(feature = "std")]
pub use if_std::WebsocketClient;

#[cfg(feature = "std")]
mod if_std {
    use anyhow::Result;
    use alloc::borrow::{Cow, ToOwned};
    use core::cell::RefCell;
    use embedded_websocket::{Client, WebSocketCloseStatusCode, WebSocketSendMessageType};
    use embedded_websocket::framer_async::{Framer, ReadResult};
    use tokio::net;
    use crate::core::framed::{Codec, Framed};
    use crate::core::tcp::{Connect, TcpStream};

    use embedded_websocket::WebSocketOptions;
    use rand::rngs::ThreadRng;
    use crate::client::websocket::errors::WebsocketError;
    use crate::Err;

    pub struct WebsocketClient<'a> {
        uri: Cow<'a, str>,
        buffer: &'a mut [u8],
        stream: RefCell<Option<Framed<TcpStream<net::TcpStream>, Codec>>>,
        framer: RefCell<Option<Framer<ThreadRng, Client>>>,
    }

    impl<'a> WebsocketClient<'a> {
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
            framer.connect(&mut self.stream.borrow_mut().as_mut().unwrap(), self.buffer, &options).await.unwrap();

            self.framer.replace(Some(framer));
        }
    }

    impl<'a> WebsocketClient<'a> {
        pub async fn read(&mut self) -> Option<Result<ReadResult<'_>>> {
            let read_result = self.framer.borrow_mut().as_mut().unwrap().read( // TODO: handle unwrap
                self.stream.borrow_mut().as_mut().unwrap(),
                self.buffer
            ).await;

            match read_result {
                Some(Err(_)) => {
                    Some(Err!(WebsocketError::ReadError))
                },
                Some(Ok(read_res)) => Some(Ok(read_res)),
                None => None
            }
        }
        pub async fn write(&mut self, message: Cow<'a, str> ) {
            self.framer.borrow_mut().as_mut().unwrap().write( // TODO: handle unwrap
                self.stream.borrow_mut().as_mut().unwrap(),
                self.buffer,
                WebSocketSendMessageType::Text,
                true,
                message.as_ref().as_bytes()
            ).await.unwrap() // TODO: Return `Result`
        }
        pub async fn close(&mut self) {
            self.framer.borrow_mut().as_mut().unwrap().close( // TODO: handle unwrap
                self.stream.borrow_mut().as_mut().unwrap(),
                self.buffer,
                WebSocketCloseStatusCode::NormalClosure,
                None,
            ).await.unwrap() // TODO: Return `Result`
        }
    }
}
