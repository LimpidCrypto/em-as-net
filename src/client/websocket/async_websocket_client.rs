use anyhow::Result;
use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    task::Poll,
};
use embedded_websocket::{framer_async::Framer, Client, WebSocketClient};
use futures::{Sink, Stream};
use rand_core::RngCore;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

// Exports
pub use embedded_websocket::{
    framer_async::ReadResult, WebSocketCloseStatusCode as WebsocketCloseStatusCode,
    WebSocketOptions as WebsocketOptions, WebSocketSendMessageType as WebsocketSendMessageType,
    WebSocketState as WebsocketState,
};

use crate::{client::websocket::errors::WebsocketError, Err};

#[cfg(feature = "std")]
pub type AsyncWebsocketClientTungstenite<Status> =
    AsyncWebsocketClient<WebSocketStream<MaybeTlsStream<TcpStream>>, Status>;
pub type AsyncWebsocketClientEmbeddedWebsocketTokio<Rng, Status> =
    AsyncWebsocketClient<Framer<Rng, Client>, Status>;
#[cfg(feature = "std")]
pub use tokio_tungstenite::tungstenite::Message;

pub struct WebsocketOpen;
pub struct WebsocketClosed;

pub struct AsyncWebsocketClient<T, Status = WebsocketClosed> {
    inner: T,
    status: PhantomData<Status>,
}

impl<T, Status> AsyncWebsocketClient<T, Status> {
    pub fn is_open(&self) -> bool {
        core::any::type_name::<Status>() == core::any::type_name::<WebsocketOpen>()
    }
}

impl<T, I> Sink<I> for AsyncWebsocketClient<T, WebsocketOpen>
where
    T: Sink<I> + Unpin,
    <T as Sink<I>>::Error: Display,
{
    type Error = anyhow::Error;

    fn poll_ready(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<core::result::Result<(), Self::Error>> {
        match Pin::new(&mut self.inner).poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn start_send(
        mut self: core::pin::Pin<&mut Self>,
        item: I,
    ) -> core::result::Result<(), Self::Error> {
        match Pin::new(&mut self.inner).start_send(item) {
            Ok(()) => Ok(()),
            Err(error) => Err!(error),
        }
    }

    fn poll_flush(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<core::result::Result<(), Self::Error>> {
        match Pin::new(&mut self.inner).poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<core::result::Result<(), Self::Error>> {
        match Pin::new(&mut self.inner).poll_close(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(error)) => Poll::Ready(Err!(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> Stream for AsyncWebsocketClient<T, WebsocketOpen>
where
    T: Stream + Unpin,
{
    type Item = <T as Stream>::Item;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(feature = "std")]
impl AsyncWebsocketClient<WebSocketStream<MaybeTlsStream<TcpStream>>, WebsocketClosed> {
    pub async fn open(
        uri: Url,
    ) -> Result<AsyncWebsocketClient<WebSocketStream<MaybeTlsStream<TcpStream>>, WebsocketOpen>>
    {
        let (websocket_stream, _) = connect_async(uri).await.unwrap();

        Ok(AsyncWebsocketClient {
            inner: websocket_stream,
            status: PhantomData::<WebsocketOpen>,
        })
    }
}

impl<Rng> AsyncWebsocketClient<Framer<Rng, Client>, WebsocketClosed>
where
    Rng: RngCore,
{
    pub async fn open<B, E>(
        stream: &mut (impl Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin),
        buffer: &mut [u8],
        rng: Rng,
        websocket_options: &WebsocketOptions<'_>,
    ) -> Result<AsyncWebsocketClient<Framer<Rng, Client>, WebsocketOpen>>
    where
        B: AsRef<[u8]>,
        E: Debug,
    {
        let websocket = WebSocketClient::new_client(rng);
        let mut framer = Framer::new(websocket);
        framer
            .connect(stream, buffer, websocket_options)
            .await
            .unwrap();

        Ok(AsyncWebsocketClient {
            inner: framer,
            status: PhantomData::<WebsocketOpen>,
        })
    }
}

impl<Rng> AsyncWebsocketClient<Framer<Rng, Client>, WebsocketOpen>
where
    Rng: RngCore,
{
    pub fn encode<E>(
        &mut self,
        message_type: WebsocketSendMessageType,
        end_of_message: bool,
        from: &[u8],
        to: &mut [u8],
    ) -> Result<usize>
    where
        E: Debug,
    {
        let len = self
            .inner
            .encode::<E>(message_type, end_of_message, from, to)
            .unwrap();

        Ok(len)
    }

    pub async fn send<'b, E>(
        &mut self,
        stream: &mut (impl Sink<&'b [u8], Error = E> + Unpin),
        stream_buf: &'b mut [u8],
        message_type: WebsocketSendMessageType,
        end_of_message: bool,
        frame_buf: &'b [u8],
    ) -> Result<()>
    where
        E: Debug,
    {
        self.inner
            .write(stream, stream_buf, message_type, end_of_message, frame_buf)
            .await
            .unwrap();

        Ok(())
    }

    pub async fn close<'b, E>(
        &mut self,
        stream: &mut (impl Sink<&'b [u8], Error = E> + Unpin),
        stream_buf: &'b mut [u8],
        close_status: WebsocketCloseStatusCode,
        status_description: Option<&str>,
    ) -> Result<()>
    where
        E: Debug,
    {
        self.inner
            .close(stream, stream_buf, close_status, status_description)
            .await
            .unwrap();

        Ok(())
    }

    pub async fn next<'a, B: Deref<Target = [u8]>, E>(
        &'a mut self,
        stream: &mut (impl Stream<Item = Result<B, E>> + Sink<&'a [u8], Error = E> + Unpin),
        buffer: &'a mut [u8],
    ) -> Option<Result<ReadResult<'_>>>
    where
        E: Debug,
    {
        match self.inner.read(stream, buffer).await {
            Some(Ok(read_result)) => Some(Ok(read_result)),
            Some(Err(error)) => Some(Err!(WebsocketError::from(error))),
            None => None,
        }
    }

    pub async fn try_next<'a, B: Deref<Target = [u8]>, E>(
        &'a mut self,
        stream: &mut (impl Stream<Item = Result<B, E>> + Sink<&'a [u8], Error = E> + Unpin),
        buffer: &'a mut [u8],
    ) -> Result<Option<ReadResult<'_>>>
    where
        E: Debug,
    {
        match self.inner.read(stream, buffer).await {
            Some(Ok(read_result)) => Ok(Some(read_result)),
            Some(Err(error)) => Err!(WebsocketError::from(error)),
            None => Ok(None),
        }
    }
}
