use super::errors::DnsError;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::net::{SocketAddr, SocketAddrV6};

#[derive(Debug)]
pub struct Aaaa {
    pub(crate) addrs: SocketAddrV6,
}

#[cfg(feature = "std")]
mod if_std {
    use super::*;
    use tokio::net::lookup_host;

    impl<'a> Aaaa {
        pub async fn new(host: Cow<'a, str>) -> Result<Self, DnsError> {
            let addresses = match lookup_host(&*host).await {
                Err(_) => return Err(DnsError::LookupError(host.clone())),
                Ok(socket_addrs_iter) => socket_addrs_iter,
            };
            return match addresses
                .filter(|x| x.is_ipv6())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(SocketAddr::V6(addrs)) => Ok(Self { addrs: *addrs }),
                None => Err(DnsError::LookupIpv6Error(host.clone())),
                _ => Err(DnsError::LookupIpv4Error(host.clone())),
            };
        }
    }
}
