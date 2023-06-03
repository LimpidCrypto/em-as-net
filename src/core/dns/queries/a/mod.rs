use super::errors::DnsError;
use anyhow::Result;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::net::SocketAddr;
use embedded_nal_async::Ipv4Addr;
use crate::core::dns::queries::Lookup;

#[derive(Debug)]
pub struct A;

#[cfg(feature = "std")]
mod if_std {
    use super::*;
    use tokio::net::lookup_host;
    use crate::Err;

    impl<'a> Lookup<'a, Ipv4Addr> for A {
        async fn lookup(url: Cow<'a, str>) -> Result<Ipv4Addr> {
            let addresses = match lookup_host(&*url).await {
                Err(_) => return Err!(DnsError::LookupError(url.clone())),
                Ok(socket_addrs_iter) => socket_addrs_iter,
            };
            return match addresses
                .filter(|x| x.is_ipv4())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(SocketAddr::V4(addrs)) => Ok( Ipv4Addr::from(addrs.ip().octets()) ),
                None => Err!(DnsError::LookupIpv4Error(url.clone())),
                _ => Err!(DnsError::LookupIpv4Error(url.clone())),
            };
        }
    }
}
