use super::errors::DnsError;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::net::{SocketAddr, SocketAddrV4};

#[derive(Debug)]
pub struct A {
    pub(crate) addrs: SocketAddrV4,
}

#[cfg(feature = "std")]
mod if_std {
    use super::*;
    use tokio::net::lookup_host;

    impl<'a> A {
        pub async fn new(host: Cow<'a, str>) -> Result<Self, DnsError> {
            let addresses = match lookup_host(&*host).await {
                Err(_) => return Err(DnsError::LookupError(host.clone())),
                Ok(socket_addrs_iter) => socket_addrs_iter,
            };
            return match addresses
                .filter(|x| x.is_ipv4())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(SocketAddr::V4(addrs)) => Ok(Self { addrs: *addrs }),
                None => Err(DnsError::LookupIpv4Error(host.clone())),
                _ => Err(DnsError::LookupIpv4Error(host.clone())),
            };
        }
    }
}
