pub mod config;
pub mod device;

use self::config::TcpConfig;
use self::device::Nic;
use crate::constants::{ETHERNET_HEADER_LEN, ETHERNET_MTU, ETHERNET_MTU_COUNT};
use crate::{singleton, Err};

use anyhow::Result;
use embassy_net::{self, Ipv4Address, Ipv4Cidr};
use heapless::Vec;
use rand::RngCore;
use static_cell::StaticCell;

use super::error::tcp_stack_error::TcpStackError;

trait Sizes {
    const MTU: usize;
    const MTU_COUNT: usize;
}

pub struct TcpStack {
    pub inner: embassy_net::Stack<
        embassy_net_driver_channel::Device<'static, { <TcpStack as Sizes>::MTU }>,
    >,
}

impl Sizes for TcpStack {
    const MTU: usize = ETHERNET_MTU + ETHERNET_HEADER_LEN; // 1514
    const MTU_COUNT: usize = ETHERNET_MTU_COUNT; // 4
}

impl TcpStack {
    pub fn new(nic_mac_address: [u8; 6]) -> Self {
        let state = singleton!(embassy_net_driver_channel::State::<
            { <TcpStack as Sizes>::MTU },
            { <TcpStack as Sizes>::MTU_COUNT },
            { <TcpStack as Sizes>::MTU_COUNT },
        >::new());
        let device = Nic::new(nic_mac_address, state);
        let config = TcpConfig::new_static(
            Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
            Some(Ipv4Address::new(192, 168, 69, 1)),
            Vec::from_iter([
                Ipv4Address::new(8, 8, 8, 8),
                Ipv4Address::new(9, 9, 9, 9),
                Ipv4Address::new(1, 1, 1, 1),
            ]),
        );
        let resources: &mut embassy_net::StackResources<2> =
            singleton!(embassy_net::StackResources::new());

        let mut seed = [0; 8];
        rand::thread_rng().fill_bytes(&mut seed);
        let seed = u64::from_le_bytes(seed);

        let stack = embassy_net::Stack::new(device.inner, config.inner, resources, seed);

        Self { inner: stack }
    }

    pub async fn run(&self) -> Result<()> {
        self.inner.run().await;

        Err!(TcpStackError::StackStoppedError)
    }
}
