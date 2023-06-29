mod queries;

use crate::core::dns::queries::{Aaaa, Lookup, A};
use alloc::borrow::Cow;
use anyhow::Result;
use core::marker::PhantomData;
use embedded_nal_async::{IpAddr, Ipv4Addr, Ipv6Addr};
pub use queries::DnsError;

/// Tries to look up IPv6 addresses first. If it fails it then tries to look up IPv4 addresses.
pub async fn lookup(url: Cow<'_, str>) -> Result<IpAddr> {
    let dns_a = Dns::<A>::new(url.clone());
    let dns_aaaa = Dns::<Aaaa>::new(url);

    return match dns_aaaa.lookup().await {
        Ok(addrs) => Ok(IpAddr::V6(addrs)),
        Err(_) => Ok(IpAddr::V4(dns_a.lookup().await?)),
    };
}

pub struct Dns<'a, T = Aaaa> {
    url: Cow<'a, str>,
    record_type: PhantomData<T>,
}

impl<'a, T> Dns<'a, T> {
    pub fn new(url: Cow<'a, str>) -> Self {
        Self {
            url,
            record_type: PhantomData,
        }
    }
}

impl<'a> Dns<'a, A> {
    pub async fn lookup(&self) -> Result<Ipv4Addr> {
        A::lookup(self.url.clone()).await
    }
}

impl<'a> Dns<'a, Aaaa> {
    pub async fn lookup(&self) -> Result<Ipv6Addr> {
        Aaaa::lookup(self.url.clone()).await
    }
}
