use crate::{client::websocket::errors::WebSocketError, Err};

use alloc::string::ToString;
use anyhow::Result;
use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    task::Poll,
};
use embedded_websocket::{
    framer_async::Framer as EmbeddedWebSocketFramer, Client as EmbeddedWebSocketClient,
    WebSocket as EmbeddedWebSocket,
};
use futures::{Sink, Stream};
use rand_core::RngCore;
use url::Url;

#[cfg(feature = "std")]
use tokio::net::TcpStream;
#[cfg(feature = "std")]
use tokio_tungstenite::{
    connect_async as tungstenite_connect_async, MaybeTlsStream as TungsteniteMaybeTlsStream,
    WebSocketStream as TungsteniteWebSocketStream,
};

// Exports
pub use embedded_websocket::{
    framer_async::{
        FramerError as EmbeddedWebSocketFramerError, ReadResult as EmbeddedWebSocketReadMessageType,
    },
    Error as EmbeddedWebSocketError, WebSocketCloseStatusCode as EmbeddedWebSocketCloseStatusCode,
    WebSocketOptions as EmbeddedWebSocketOptions,
    WebSocketSendMessageType as EmbeddedWebSocketSendMessageType,
    WebSocketState as EmbeddedWebSocketState,
};

#[cfg(feature = "std")]
pub type AsyncWebSocketClientTungstenite<Status> =
    AsyncWebSocketClient<TungsteniteWebSocketStream<TungsteniteMaybeTlsStream<TcpStream>>, Status>;
pub type AsyncWebSocketClientEmbeddedWebSocketTokio<Rng, Status> =
    AsyncWebSocketClient<EmbeddedWebSocketFramer<Rng, EmbeddedWebSocketClient>, Status>;
#[cfg(feature = "std")]
pub use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

pub struct WebSocketOpen;
pub struct WebSocketClosed;

pub struct AsyncWebSocketClient<T, Status = WebSocketClosed> {
    inner: T,
    status: PhantomData<Status>,
}

impl<T, Status> AsyncWebSocketClient<T, Status> {
    pub fn is_open(&self) -> bool {
        core::any::type_name::<Status>() == core::any::type_name::<WebSocketOpen>()
    }
}

impl<T, I> Sink<I> for AsyncWebSocketClient<T, WebSocketOpen>
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

impl<T> Stream for AsyncWebSocketClient<T, WebSocketOpen>
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
impl
    AsyncWebSocketClient<
        TungsteniteWebSocketStream<TungsteniteMaybeTlsStream<TcpStream>>,
        WebSocketClosed,
    >
{
    pub async fn open(
        uri: Url,
    ) -> Result<
        AsyncWebSocketClient<
            TungsteniteWebSocketStream<TungsteniteMaybeTlsStream<TcpStream>>,
            WebSocketOpen,
        >,
    > {
        let (websocket_stream, _) = tungstenite_connect_async(uri.to_string()).await.unwrap();

        Ok(AsyncWebSocketClient {
            inner: websocket_stream,
            status: PhantomData::<WebSocketOpen>,
        })
    }
}

impl<Rng>
    AsyncWebSocketClient<EmbeddedWebSocketFramer<Rng, EmbeddedWebSocketClient>, WebSocketClosed>
where
    Rng: RngCore,
{
    pub async fn open<B, E>(
        stream: &mut (impl Stream<Item = Result<B, E>> + for<'a> Sink<&'a [u8], Error = E> + Unpin),
        buffer: &mut [u8],
        rng: Rng,
        websocket_options: &EmbeddedWebSocketOptions<'_>,
    ) -> Result<
        AsyncWebSocketClient<EmbeddedWebSocketFramer<Rng, EmbeddedWebSocketClient>, WebSocketOpen>,
    >
    where
        B: AsRef<[u8]>,
        E: Debug,
    {
        let websocket = EmbeddedWebSocket::<Rng, EmbeddedWebSocketClient>::new_client(rng);
        let mut framer = EmbeddedWebSocketFramer::new(websocket);
        framer
            .connect(stream, buffer, websocket_options)
            .await
            .unwrap();

        Ok(AsyncWebSocketClient {
            inner: framer,
            status: PhantomData::<WebSocketOpen>,
        })
    }
}

impl<Rng> AsyncWebSocketClient<EmbeddedWebSocketFramer<Rng, EmbeddedWebSocketClient>, WebSocketOpen>
where
    Rng: RngCore,
{
    pub fn encode<E>(
        &mut self,
        message_type: EmbeddedWebSocketSendMessageType,
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
        message_type: EmbeddedWebSocketSendMessageType,
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
        close_status: EmbeddedWebSocketCloseStatusCode,
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
    ) -> Option<Result<EmbeddedWebSocketReadMessageType<'_>>>
    where
        E: Debug,
    {
        match self.inner.read(stream, buffer).await {
            Some(Ok(read_result)) => Some(Ok(read_result)),
            Some(Err(error)) => Some(Err!(WebSocketError::from(error))),
            None => None,
        }
    }

    pub async fn try_next<'a, B: Deref<Target = [u8]>, E>(
        &'a mut self,
        stream: &mut (impl Stream<Item = Result<B, E>> + Sink<&'a [u8], Error = E> + Unpin),
        buffer: &'a mut [u8],
    ) -> Result<Option<EmbeddedWebSocketReadMessageType<'_>>>
    where
        E: Debug,
    {
        match self.inner.read(stream, buffer).await {
            Some(Ok(read_result)) => Ok(Some(read_result)),
            Some(Err(error)) => Err!(WebSocketError::from(error)),
            None => Ok(None),
        }
    }
}
