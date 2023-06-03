use super::errors::DnsError;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::Result;
use core::net::SocketAddr;
use embedded_nal_async::Ipv6Addr;

#[derive(Debug)]
pub struct Aaaa;

#[cfg(feature = "std")]
mod if_std {
    use super::*;
    use crate::core::dns::queries::Lookup;
    use crate::Err;
    use tokio::net::lookup_host;

    impl<'a> Lookup<'a, Ipv6Addr> for Aaaa {
        async fn lookup(url: Cow<'a, str>) -> Result<Ipv6Addr> {
            let addresses = match lookup_host(&*url).await {
                Err(_) => return Err!(DnsError::LookupError(url.clone())),
                Ok(socket_addrs_iter) => socket_addrs_iter,
            };
            return match addresses
                .filter(|x| x.is_ipv6())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(SocketAddr::V6(addrs)) => Ok(Ipv6Addr::from(addrs.ip().octets())),
                None => Err!(DnsError::LookupIpv6Error(url.clone())),
                _ => Err!(DnsError::LookupIpv6Error(url.clone())),
            };
        }
    }
}
