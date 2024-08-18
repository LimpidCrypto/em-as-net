mod queries;

pub use queries::DnsError;
use queries::{Aaaa, Lookup, A};

use anyhow::Result;
use core::marker::PhantomData;
use embedded_nal_async::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

/// Tries to look up IPv6 addresses first. If it fails it then tries to look up IPv4 addresses.
pub async fn lookup(url: Url) -> Result<IpAddr> {
    let dns_a = Dns::<A>::new(url.clone());
    let dns_aaaa = Dns::<Aaaa>::new(url);

    match dns_aaaa.lookup().await {
        Ok(addrs) => Ok(IpAddr::V6(addrs)),
        Err(_) => Ok(IpAddr::V4(dns_a.lookup().await?)),
    }
}

pub struct Dns<T = Aaaa> {
    url: Url,
    record_type: PhantomData<T>,
}

impl<T> Dns<T> {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            record_type: PhantomData,
        }
    }
}

impl Dns<A> {
    pub async fn lookup(&self) -> Result<Ipv4Addr> {
        A::lookup(&self.url).await
    }
}

impl Dns<Aaaa> {
    pub async fn lookup(&self) -> Result<Ipv6Addr> {
        Aaaa::lookup(&self.url).await
    }
}
