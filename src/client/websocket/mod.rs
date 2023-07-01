pub mod errors;

use crate::client::websocket::errors::{AddrsError, WebsocketError};
use crate::core::framed::{Codec, Framed};
use crate::core::io::{AsyncRead, AsyncWrite};
use crate::core::tcp::TcpConnect;
use crate::Err;
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use anyhow::Result;
use embedded_nal_async::IpAddr;
use embedded_websocket::framer_async::Framer;
use embedded_websocket::{Client, WebSocketCloseStatusCode};
use rand_core::RngCore;
use url::Url;

// exports
pub use embedded_websocket::{
    framer_async::ReadResult, WebSocketOptions as WebsocketOptions,
    WebSocketSendMessageType as WebsocketSendMessageType, WebSocketState as WebsocketState,
};

pub struct WebsocketClient<'a, T, R: RngCore> {
    socket: Option<Framed<T, Codec>>,
    framer: Option<Framer<R, Client>>,
    uri: Cow<'a, str>,
    buffer: &'a mut [u8],
}

impl<'a, T, R: RngCore> WebsocketClient<'a, T, R> {
    pub fn new(uri: Cow<'a, str>, buffer: &'a mut [u8]) -> Self {
        Self {
            socket: None,
            framer: None,
            uri,
            buffer,
        }
    }

    fn get_ws_options_path(&self, url: &'a Url) -> Result<&'a str> {
        let path = self.get_path(url);

        Ok(path)
    }

    fn get_ws_options_origin(&self) -> Result<Cow<'a, str>> {
        let url = self.get_url()?;
        let scheme = self.get_scheme(&url)?;
        let domain = self.get_domain(&url)?;
        let port = self.get_port(&url)?;
        let origin = String::from_iter([scheme, "://", domain, ":", port.to_string().as_str()]);

        Ok(origin.into())
    }

    async fn lookup_ip(&self, domain: &str) -> Result<IpAddr> {
        crate::core::dns::lookup(domain.into()).await
    }
}

impl<'a, T, R: RngCore> UriParser<'a> for WebsocketClient<'a, T, R> {
    fn get_url(&self) -> Result<Url> {
        match Url::parse(&self.uri) {
            Err(_) => Err!(AddrsError::ParseUrlError(&self.uri)),
            Ok(url) => Ok(url),
        }
    }
}

impl<'a, T, R> WebsocketClientConnect<'a, T, R> for WebsocketClient<'a, T, R>
where
    T: TcpConnect<'a> + AsyncRead + AsyncWrite + Unpin,
    R: RngCore,
{
    async fn connect(
        &mut self,
        socket: T,
        options: Option<WebsocketOptions<'a>>,
        rng: R,
    ) -> Result<()> {
        // parse uri
        let url = self.get_url()?;
        let domain = self.get_domain(&url)?;
        let port = self.get_port(&url)?.to_string();
        let opt_path = self.get_ws_options_path(&url)?;

        // get websocket options
        let ws_options = match options {
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
        let ip = self
            .lookup_ip(String::from_iter([domain, ":", &*port]).as_str())
            .await?;

        let mut socket = socket;
        socket
            .connect(Cow::from(String::from_iter([
                ip.to_string().as_str(),
                ":",
                &*port,
            ])))
            .await?;

        // define socket
        let framed = Framed::new(socket, Codec::new());
        self.socket = Some(framed);

        // Connect Websocket
        let ws_client = embedded_websocket::WebSocketClient::new_client(rng);
        let mut framer = Framer::new(ws_client);
        if let Err(framer_err) = framer
            .connect(
                match &mut self.socket {
                    None => return Err!(WebsocketError::<anyhow::Error>::NotConnected),
                    Some(s) => s,
                },
                self.buffer,
                &ws_options,
            )
            .await
        {
            return Err!(WebsocketError::from(framer_err));
        }

        self.framer = Some(framer);

        Ok(())
    }
}

impl<'a, T, R> WebsocketClientIo<'a> for WebsocketClient<'a, T, R>
where
    T: TcpConnect<'a> + AsyncRead + AsyncWrite + Unpin,
    R: RngCore,
{
    async fn read(&mut self) -> Option<Result<ReadResult<'_>>> {
        if let Some(framer) = &mut self.framer {
            let read_result = framer
                .read(
                    match &mut self.socket {
                        None => return Some(Err!(WebsocketError::<anyhow::Error>::NotConnected)),
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

        Some(Err!(WebsocketError::<anyhow::Error>::NotConnected))
    }

    async fn write(
        &mut self,
        message: Cow<'a, str>,
        send_msg_type: Option<WebsocketSendMessageType>,
    ) -> Result<()> {
        if let Some(framer) = &mut self.framer {
            return match framer
                .write(
                    match &mut self.socket {
                        None => {
                            return Err!(WebsocketError::<anyhow::Error>::NotConnected);
                        }
                        Some(stream) => stream,
                    },
                    self.buffer,
                    send_msg_type.unwrap_or(WebsocketSendMessageType::Text),
                    true,
                    message.as_ref().as_bytes(),
                )
                .await
            {
                Ok(()) => Ok(()),
                Err(err) => Err!(WebsocketError::from(err)),
            };
        }

        Err!(WebsocketError::<anyhow::Error>::NotConnected)
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(framer) = &mut self.framer {
            return match framer
                .close(
                    match &mut self.socket {
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

        Err!(WebsocketError::<anyhow::Error>::NotConnected)
    }
}

pub trait WebsocketClientConnect<'a, T, R: RngCore> {
    async fn connect(
        &mut self,
        socket: T,
        options: Option<WebsocketOptions<'a>>,
        rng: R,
    ) -> Result<()>;
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

trait UriParser<'a> {
    fn get_scheme(&self, url: &Url) -> Result<&'a str> {
        match url.scheme() {
            "ws" => Ok("http"),
            "wss" => Ok("https"),
            invalid => Err!(AddrsError::InvalidScheme(invalid)),
        }
    }

    fn get_domain(&self, url: &'a Url) -> Result<&'a str> {
        match url.domain() {
            None => Err!(AddrsError::ParseDomainError),
            Some(domain) => Ok(domain),
        }
    }

    fn get_port(&self, url: &Url) -> Result<u16> {
        match url.port() {
            None => match self.get_scheme(url) {
                Err(invalid_scheme) => Err(invalid_scheme),
                Ok("http") => Ok(80),
                Ok("https") => Ok(443),
                _ => Err!(AddrsError::InvalidScheme("")),
            },
            Some(port) => Ok(port),
        }
    }

    fn get_path(&self, url: &'a Url) -> &'a str {
        url.path()
    }

    fn get_url(&self) -> Result<Url>;
}
