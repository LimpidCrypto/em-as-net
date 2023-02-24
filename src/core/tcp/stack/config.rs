use embassy_net::{DhcpConfig, Ipv4Address, Ipv4Cidr, StaticConfig};
use heapless::Vec;

pub struct TcpConfig {
    pub inner: embassy_net::Config,
}

impl TcpConfig {
    pub fn new_dhcp() -> Self {
        let config = embassy_net::Config::Dhcp(DhcpConfig {
            ..Default::default()
        });

        Self { inner: config }
    }

    pub fn new_static(
        address: Ipv4Cidr,
        gateway: Option<Ipv4Address>,
        dns_servers: Vec<Ipv4Address, 3>,
    ) -> Self {
        let config = embassy_net::Config::Static(StaticConfig {
            address,
            gateway,
            dns_servers,
        });

        Self { inner: config }
    }
}
