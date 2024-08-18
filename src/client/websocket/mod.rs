pub mod exceptions;

use core::marker::PhantomData;

use alloc::string::ToString;
use anyhow::Result;
use embedded_io_async::{Read, Write};
use embedded_websocket::{framer_async::Framer, Client, WebSocketClient, WebSocketOptions};
use exceptions::WebsocketError;
use rand_core::RngCore;
use url::Url;

use crate::Err;

pub struct WebsocketClosed;
pub struct WebsocketOpen;

pub struct AsyncWebsocketClient<T: RngCore, Status = WebsocketClosed> {
    inner: Framer<T, Client>,
    status: PhantomData<Status>,
}

impl<T: RngCore, Status> AsyncWebsocketClient<T, Status> {
    pub fn is_open(&self) -> bool {
        core::any::type_name::<Status>() == core::any::type_name::<WebsocketOpen>()
    }
}

impl<T: RngCore> AsyncWebsocketClient<T, WebsocketClosed> {
    pub async fn open<S>(
        buf: &mut [u8],
        stream: &mut S,
        uri: Url,
        rng: T,
    ) -> Result<AsyncWebsocketClient<T, WebsocketOpen>>
    where
        S: Read + Write + Unpin,
    {
        // replace the scheme with http or https
        let scheme = match uri.scheme() {
            "wss" => "https",
            "ws" => "http",
            _ => uri.scheme(),
        };
        let port = match uri.port() {
            Some(port) => port,
            None => match uri.scheme() {
                "wss" => 443,
                "ws" => 80,
                _ => 80,
            },
        }
        .to_string();
        let path = uri.path();
        let host = match uri.host_str() {
            Some(host) => host,
            None => return Err(WebsocketError::Disconnected.into()),
        };
        let origin = scheme.to_string() + "://" + host + ":" + &port + path;
        let websocket_options = WebSocketOptions {
            path,
            host,
            origin: &origin,
            sub_protocols: None,
            additional_headers: None,
        };
        let websocket = Framer::new(WebSocketClient::new_client(rng));
        match websocket.connect(stream, buf, &websocket_options).await {
            Ok(_) => Ok(AsyncWebsocketClient {
                inner: websocket,
                status: PhantomData::<WebsocketOpen>,
            }),
            Err(e) => Err!(e),
        }
    }
}
