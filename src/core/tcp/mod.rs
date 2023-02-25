pub mod error;
pub mod stack;

use core::cell::RefCell;

use anyhow::Result;
use embassy_net::{tcp::ConnectError, Ipv4Address};
use embedded_io::asynch::Write;

use crate::Err;
use crate::{constants::TCP_BUF, core::tcp::stack::TcpStack, singleton};
use error::tcp_socket_error::TcpSocketError;
use static_cell::StaticCell;

pub struct TcpSocket {
    pub remote: (Ipv4Address, u16),
    pub inner: RefCell<embassy_net::tcp::TcpSocket<'static>>,
    pub stack: &'static TcpStack,
}

impl TcpSocket {
    pub fn new(mac_address: [u8; 6], remote: (Ipv4Address, u16)) -> Self {
        let stack: &'static TcpStack = &*singleton!(TcpStack::new(mac_address));

        let rx_buffer: &'static mut [u8; TCP_BUF] = singleton!([0; TCP_BUF]);
        let tx_buffer: &'static mut [u8; TCP_BUF] = singleton!([0; TCP_BUF]);

        let socket = embassy_net::tcp::TcpSocket::new(&stack.inner, rx_buffer, tx_buffer);

        Self {
            remote,
            inner: RefCell::new(socket),
            stack,
        }
    }

    #[allow(unused_must_use)]
    pub async fn connect(&self) -> Result<(), ConnectError> {
        self.stack.run();
        embassy_futures::yield_now().await;

        self._do_connect().await
    }

    pub async fn write_all(&self, buf: &[u8]) -> Result<()> {
        match self.inner.borrow_mut().write_all(buf).await {
            Ok(_) => Ok(()),
            Err(_) => Err!(TcpSocketError::WriteError),
        }
    }

    async fn _do_connect(&self) -> Result<(), ConnectError> {
        self.inner
            .borrow_mut()
            .set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));
        self.inner.borrow_mut().connect(self.remote).await
    }
}

#[cfg(test)]
#[cfg(feature = "test")]
mod test {
    use super::*;
    use embassy_net::Ipv4Address;

    #[tokio::test]
    async fn test_connect() {
        let mac_address = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0];
        let ip = Ipv4Address::new(4, 2, 2, 2);
        let remote = (ip, 53);
        let socket = TcpSocket::new(mac_address, remote);

        // TODO: FIX `.connect` takes forever
        let r = socket.connect().await;
        if let Err(e) = r {
            println!("connect error: {:?}", e);
            return;
        }
        println!("connected!");
        loop {
            let r = socket.write_all(b"Hello!\n").await;
            if let Err(e) = r {
                println!("write error: {:?}", e);
                return;
            }
        }
    }
}
