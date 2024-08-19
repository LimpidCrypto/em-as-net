pub mod exceptions;

use core::marker::PhantomData;

use alloc::string::ToString;
use anyhow::Result;
use embedded_io_async::{Read, Write};
use embedded_websocket::{framer_async::Framer, Client, WebSocketClient, WebSocketOptions};
use exceptions::WebSocketException;
use rand_core::RngCore;
use url::Url;

pub use embedded_websocket::{framer_async::ReadResult, WebSocketSendMessageType};

use crate::Err;

pub struct WebSocketClosed;
pub struct WebSocketOpen;

#[allow(async_fn_in_trait)]
pub trait WebSocketRead {
    async fn read<'a, S: Read + Write + Unpin>(
        &'a mut self,
        stream: &mut S,
        buf: &'a mut [u8],
    ) -> Option<Result<ReadResult<'_>>>;

    async fn try_read<'a, S: Read + Write + Unpin>(
        &'a mut self,
        stream: &mut S,
        buf: &'a mut [u8],
    ) -> Result<Option<ReadResult<'_>>> {
        match self.read(stream, buf).await {
            Some(Ok(result)) => Ok(Some(result)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait WebSocketWrite {
    async fn write<S: Write + Read + Unpin>(
        &mut self,
        tx: &mut S,
        tx_buf: &mut [u8],
        message_type: WebSocketSendMessageType,
        end_of_message: bool,
        frame_buf: &[u8],
    ) -> Result<()>;
}

pub struct AsyncWebSocketClient<T: RngCore, Status = WebSocketClosed> {
    inner: Framer<T, Client>,
    status: PhantomData<Status>,
}

impl<T: RngCore, Status> AsyncWebSocketClient<T, Status> {
    pub fn is_open(&self) -> bool {
        core::any::type_name::<Status>() == core::any::type_name::<WebSocketOpen>()
    }
}

impl<T: RngCore> AsyncWebSocketClient<T, WebSocketClosed> {
    pub async fn open<S>(
        stream: &mut S,
        buf: &mut [u8],
        uri: &Url,
        rng: T,
        sub_protocols: Option<&[&str]>,
        additional_headers: Option<&[&str]>,
    ) -> Result<AsyncWebSocketClient<T, WebSocketOpen>>
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
            None => return Err(WebSocketException::<anyhow::Error>::InvalidDomain.into()),
        };
        let origin = scheme.to_string() + "://" + host + ":" + &port + path;
        let websocket_options = WebSocketOptions {
            path,
            host,
            origin: origin.as_str(),
            sub_protocols,
            additional_headers,
        };
        let mut websocket = Framer::new(WebSocketClient::new_client(rng));
        match websocket.connect(stream, buf, &websocket_options).await {
            Ok(_) => Ok(AsyncWebSocketClient {
                inner: websocket,
                status: PhantomData::<WebSocketOpen>,
            }),
            Err(e) => Err(WebSocketException::from(e).into()),
        }
    }
}

impl<T: RngCore> WebSocketRead for AsyncWebSocketClient<T, WebSocketOpen> {
    async fn read<'a, S: Read + Write + Unpin>(
        &'a mut self,
        stream: &mut S,
        buf: &'a mut [u8],
    ) -> Option<Result<ReadResult<'_>>> {
        match self.inner.read(stream, buf).await {
            Some(Ok(result)) => Some(Ok(result)),
            Some(Err(e)) => Some(Err!(WebSocketException::from(e))),
            None => None,
        }
    }
}

impl<T: RngCore> WebSocketWrite for AsyncWebSocketClient<T, WebSocketOpen> {
    async fn write<S: Write + Read + Unpin>(
        &mut self,
        tx: &mut S,
        tx_buf: &mut [u8],
        message_type: WebSocketSendMessageType,
        end_of_message: bool,
        frame_buf: &[u8],
    ) -> Result<()> {
        match self
            .inner
            .write(tx, tx_buf, message_type, end_of_message, frame_buf)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err!(WebSocketException::from(e)),
        }
    }
}
