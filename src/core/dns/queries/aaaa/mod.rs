use anyhow::Result;
use embedded_nal_async::Ipv6Addr;

#[derive(Debug)]
pub struct Aaaa;

#[cfg(feature = "std")]
mod impl_lookup {
    use core::net::SocketAddr;

    use super::*;
    use crate::core::dns::queries::Lookup;
    use crate::core::dns::DnsError;
    use crate::Err;
    use alloc::string::ToString;
    use alloc::vec::Vec;
    use tokio::net::lookup_host;
    use url::Url;

    impl Lookup<Ipv6Addr> for Aaaa {
        async fn lookup(url: &Url) -> Result<Ipv6Addr> {
            let url = url.to_string();
            let addresses = match lookup_host(&*url).await {
                Err(_) => return Err!(DnsError::LookupError(url.into())),
                Ok(socket_addrs_iter) => socket_addrs_iter,
            };
            return match addresses
                .filter(|x| x.is_ipv6())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(SocketAddr::V6(addrs)) => Ok(Ipv6Addr::from(addrs.ip().octets())),
                None => Err!(DnsError::LookupIpv6Error(url.into())),
                _ => Err!(DnsError::LookupIpv6Error(url.into())),
            };
        }
    }
}

#[cfg(not(feature = "std"))]
mod impl_lookup {
    use super::*;
    use crate::core::dns::queries::Lookup;
    use url::Url;

    impl Lookup<Ipv6Addr> for Aaaa {
        async fn lookup(_url: &Url) -> Result<Ipv6Addr> {
            todo!("Implement lookup for Aaaa record type without std")
        }
    }
}
