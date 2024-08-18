use crate::core::dns::queries::Lookup;
use anyhow::Result;
use embedded_nal_async::Ipv4Addr;

#[derive(Debug)]
pub struct A;

#[cfg(feature = "std")]
mod impl_lookup {
    use core::net::SocketAddr;

    use super::*;
    use crate::{core::dns::DnsError, Err};
    use alloc::{string::ToString, vec::Vec};
    use tokio::net::lookup_host;
    use url::Url;

    impl Lookup<Ipv4Addr> for A {
        async fn lookup(url: &Url) -> Result<Ipv4Addr> {
            let url = url.to_string();
            let addresses = match lookup_host(&*url).await {
                Err(_) => return Err!(DnsError::LookupError(url.into())),
                Ok(socket_addrs_iter) => socket_addrs_iter,
            };
            return match addresses
                .filter(|x| x.is_ipv4())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(SocketAddr::V4(addrs)) => Ok(Ipv4Addr::from(addrs.ip().octets())),
                None => Err!(DnsError::LookupIpv4Error(url.into())),
                _ => Err!(DnsError::LookupIpv4Error(url.into())),
            };
        }
    }
}

#[cfg(not(feature = "std"))]
mod impl_lookup {
    use url::Url;

    use super::*;

    impl Lookup<Ipv4Addr> for A {
        async fn lookup(_url: &Url) -> Result<Ipv4Addr> {
            todo!("Implement lookup for A record type without std")
        }
    }
}
