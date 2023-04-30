pub mod errors;

use alloc::borrow::Cow;
use core::cell::RefCell;
use futures::{Sink, Stream};
use pin_project_lite::pin_project;
use crate::core::tcp::{Connect, TcpStream};

pin_project! {
    pub struct WebsocketClient<'a, T> {
        uri: Cow<'a, str>,
        stream: RefCell<T>,
    }
}

#[cfg(feature = "std")]
mod if_std {
    use anyhow::Result;
    use alloc::borrow::{Cow, ToOwned};
    use core::cell::RefCell;
    use core::str::FromStr;
    use embedded_websocket::framer_async::Framer;
    use futures::{Sink, Stream};
    use tokio::net;
    use crate::client::websocket::WebsocketClient;
    use crate::core::framed::{Codec, Framed};
    use crate::core::tcp::{Connect, TcpStream};

    pub use embedded_websocket::WebSocketOptions;
    use url::{Host, Url};
    use crate::core::io::{AsyncRead, AsyncWrite};
    use crate::Err;

    impl<'a, T: Connect<'a> + AsyncWrite + AsyncRead> WebsocketClient<'a, T> {
        pub fn new(uri: Cow<'a, str>, tcp_stream: T) -> Self {
            Self {
                uri,
                stream: RefCell::new(tcp_stream),
            }
        }

        pub async fn connect(&self, options: WebSocketOptions<'a>, to_buffer: &mut [u8]) {
            let project = self.project();
            let mut tcp_stream: TcpStream<net::TcpStream> = TcpStream::new();
            tcp_stream.connect(project.uri.to_owned()).await.unwrap(); // TODO: handle error
            let mut framed = Framed::new(tcp_stream, Codec::new());

            let rng = rand::thread_rng();
            let ws_client = embedded_websocket::WebSocketClient::new_client(rng);

            let mut framer = Framer::new(ws_client);
            framer.connect(&mut framed, to_buffer, &options).await.unwrap();

            project.stream.replace(Some(framer));
        }

        async fn connect_tcp(&self) -> Result<()> {
            let project = self.project();
            match project.stream.borrow_mut().connect(project.uri.to_owned()).await {
                Ok(_) => { Ok(()) }
                Err(err) => { Err!(err) }
            }

        }

        fn get_domain(&self) -> &'a str {
            let project = self.project();
            let url = Url::parse(project.uri.into()).unwrap(); // TODO: handle error

            url.domain().unwrap() // TODO: handle error
        }
    }

    impl<'a, T: Sink<&'a [u8]> + Stream> WebsocketClient<'a, T> {
        pub async fn read(&mut self) {
            let project = self.project();
            match self.stream.borrow_mut().as_mut() {
                None => {}
                Some(ws) => {

                }
            }
        }
        pub async fn write(&mut self) {}
        pub async fn close(&mut self) {}
    }
}
