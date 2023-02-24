use core::cell::RefCell;

use anyhow::Result;
use embassy_net::{tcp::ConnectError, Ipv4Address};

use crate::{constants::TCP_BUF, core::tcp::stack::TcpStack, singleton};
use static_cell::StaticCell;

pub mod stack;

pub struct TcpSocket {
    pub remote: (Ipv4Address, u16),
    pub socket: RefCell<embassy_net::tcp::TcpSocket<'static>>,
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
            socket: RefCell::new(socket),
            stack,
        }
    }

    #[allow(unused_must_use)]
    pub async fn connect(&self) -> Result<(), ConnectError> {
        self.stack._run();
        embassy_futures::yield_now().await;

        self._do_connect().await
    }

    async fn _do_connect(&self) -> Result<(), ConnectError> {
        self.socket
            .borrow_mut()
            .set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));
        self.socket.borrow_mut().connect(self.remote).await
    }
}

#[cfg(test)]
#[cfg(feature = "test")]
mod test {
    use super::*;
    use embassy_net::Ipv4Address;

    #[tokio::test]
    async fn test_connect() {
        let mac_address = [0xa4, 0x83, 0xe7, 0x48, 0x31, 0x21];
        let ip = Ipv4Address::new(4, 2, 2, 2);
        let remote = (ip, 53);
        let stream = TcpSocket::new(mac_address, remote);

        // TODO: FIX NoRoute ERROR
        stream.connect().await.unwrap();
    }
}
