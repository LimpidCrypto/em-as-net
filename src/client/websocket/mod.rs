pub mod errors;

use crate::core::framed::{Codec, Framed};
use crate::core::tcp::TcpSocket;
use alloc::borrow::Cow;
use anyhow::Result;
use core::cell::RefCell;
use embedded_websocket::framer_async::Framer;
use embedded_websocket::Client;
use rand_core::RngCore;
// exports
pub use embedded_websocket::{
    framer_async::ReadResult, WebSocketOptions as WebsocketOptions,
    WebSocketSendMessageType as WebsocketSendMessageType, WebSocketState as WebsocketState,
};

pub struct WebsocketClient<'a, T, U: RngCore> {
    uri: Cow<'a, str>,
    buffer: &'a mut [u8],
    stream: RefCell<Option<Framed<TcpSocket<T>, Codec>>>,
    framer: RefCell<Option<Framer<U, Client>>>,
}

#[cfg(feature = "std")]
mod if_std {
    use crate::core::framed::{Codec, Framed};
    use crate::core::tcp::{adapters::TcpTokio, TcpConnect, TcpSocket};
    use alloc::borrow::Cow;
    use alloc::string::{String, ToString};
    use anyhow::Result;
    use core::cell::RefCell;

    use embedded_websocket::framer_async::{Framer, ReadResult};
    use embedded_websocket::{WebSocketCloseStatusCode, WebSocketSendMessageType};

    use crate::client::websocket::errors::{AddrsError, WebsocketError};
    use crate::client::websocket::{WebsocketClient, WebsocketClientIo};
    use crate::Err;
    use embedded_websocket::WebSocketOptions as WebsocketOptions;
    use rand::rngs::ThreadRng;
    use url::Url;

    impl<'a> WebsocketClient<'a, TcpTokio, ThreadRng> {
        pub fn new(uri: Cow<'a, str>, buffer: &'a mut [u8]) -> Self {
            Self {
                uri,
                buffer,
                stream: RefCell::new(None),
                framer: RefCell::new(None),
            }
        }

        pub async fn connect(&mut self, options: Option<WebsocketOptions<'a>>) -> Result<()> {
            // parse uri
            let url = match Url::parse(&self.uri) {
                Err(_) => return Err!(AddrsError::ParseUrlError(&self.uri)),
                Ok(url) => url,
            };
            let domain = match url.domain() {
                None => return Err!(AddrsError::ParseDomainError(&self.uri)),
                Some(domain) => domain,
            };
            let port = match url.port() {
                None => String::new(),
                Some(port) => String::from_iter([":", port.to_string().as_str()]),
            };
            let (uri_path, opt_path) = match url.path() {
                "/" => ("", ("/")),
                path => (path, path),
            };
            let query = url.query().unwrap_or("");

            // get websocket options
            let options = match options {
                Some(options) => options,
                None => WebsocketOptions {
                    path: opt_path,
                    host: domain,
                    origin: &self.uri,
                    sub_protocols: None,
                    additional_headers: None,
                },
            };

            // Connect TCP
            let tcp_socket = match TcpSocket::<TcpTokio>::connect(Cow::from(String::from_iter([
                domain,
                port.as_str(),
                uri_path,
                query,
            ])))
            .await
            {
                Err(conn_err) => return Err!(conn_err),
                Ok(socket) => socket,
            };
            let framed = Framed::new(tcp_socket, Codec::new());
            self.stream.replace(Some(framed));

            // Connect Websocket
            let rng = rand::thread_rng();
            let ws_client = embedded_websocket::WebSocketClient::new_client(rng);
            let mut framer = Framer::new(ws_client);
            if let Err(framer_err) = framer
                .connect(
                    &mut match self.stream.borrow_mut().as_mut() {
                        None => return Err!(WebsocketError::<anyhow::Error>::NotConnected),
                        Some(stream) => stream,
                    },
                    self.buffer,
                    &options,
                )
                .await
            {
                return Err!(WebsocketError::from(framer_err));
            }

            self.framer.replace(Some(framer));

            Ok(())
        }

        pub fn is_open(&self) -> bool {
            self.framer.borrow().is_some() && self.stream.borrow().is_some()
        }
    }

    impl<'a> WebsocketClientIo<'a> for WebsocketClient<'a, TcpTokio, ThreadRng> {
        async fn read(&mut self) -> Option<Result<ReadResult<'_>>> {
            if let Some(framer) = self.framer.borrow_mut().as_mut() {
                let read_result = framer
                    .read(
                        &mut match self.stream.borrow_mut().as_mut() {
                            None => {
                                return Some(Err!(WebsocketError::<anyhow::Error>::NotConnected))
                            }
                            Some(stream) => stream,
                        },
                        self.buffer,
                    )
                    .await;

                return match read_result {
                    Some(Err(err)) => Some(Err!(WebsocketError::<anyhow::Error>::from(err))),
                    Some(Ok(read_res)) => Some(Ok(read_res)),
                    None => None,
                };
            }

            return Some(Err!(WebsocketError::<anyhow::Error>::NotConnected));
        }

        async fn write(
            &mut self,
            message: Cow<'a, str>,
            send_msg_type: Option<WebSocketSendMessageType>,
        ) -> Result<()> {
            if let Some(framer) = self.framer.borrow_mut().as_mut() {
                return match framer
                    .write(
                        match self.stream.borrow_mut().as_mut() {
                            None => {
                                return Err!(WebsocketError::<anyhow::Error>::NotConnected);
                            }
                            Some(stream) => stream,
                        },
                        self.buffer,
                        send_msg_type.unwrap_or(WebSocketSendMessageType::Text),
                        true,
                        message.as_ref().as_bytes(),
                    )
                    .await
                {
                    Ok(()) => Ok(()),
                    Err(err) => Err!(WebsocketError::from(err)),
                };
            }

            return Err!(WebsocketError::<anyhow::Error>::NotConnected);
        }

        async fn close(&mut self) -> Result<()> {
            if let Some(framer) = self.framer.borrow_mut().as_mut() {
                return match framer
                    .close(
                        &mut match self.stream.borrow_mut().as_mut() {
                            None => return Err!(WebsocketError::<anyhow::Error>::NotConnected),
                            Some(stream) => stream,
                        },
                        self.buffer,
                        WebSocketCloseStatusCode::NormalClosure,
                        None,
                    )
                    .await
                {
                    Err(framer_err) => return Err!(WebsocketError::from(framer_err)),
                    Ok(()) => Ok(()),
                };
            }

            return Err!(WebsocketError::<anyhow::Error>::NotConnected);
        }
    }
}

pub trait WebsocketClientIo<'a> {
    async fn read(&mut self) -> Option<Result<ReadResult<'_>>>;
    async fn write(
        &mut self,
        message: Cow<'a, str>,
        send_msg_type: Option<WebsocketSendMessageType>,
    ) -> Result<()>;
    async fn close(&mut self) -> Result<()>;
}
