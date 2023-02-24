pub mod config;
pub mod device;

use self::config::TcpConfig;
use self::device::Nic;
use crate::constants::{ETHERNET_HEADER_LEN, ETHERNET_MTU, ETHERNET_MTU_COUNT};
use crate::singleton;

use embassy_net;
use rand::RngCore;
use static_cell::StaticCell;

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
        let config = TcpConfig::new_dhcp();
        let resources: &mut embassy_net::StackResources<2> =
            singleton!(embassy_net::StackResources::new());

        let mut seed = [0; 8];
        rand::thread_rng().fill_bytes(&mut seed);
        let seed = u64::from_le_bytes(seed);

        let stack = embassy_net::Stack::new(device.inner, config.inner, resources, seed);

        Self { inner: stack }
    }

    pub async fn _run(&self) -> ! {
        self.inner.run().await
    }
}
