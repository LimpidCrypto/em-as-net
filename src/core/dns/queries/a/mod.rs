use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::net::{SocketAddr, SocketAddrV4};
use super::errors::DnsError;

#[derive(Debug)]
pub struct A {
    pub(crate) addrs: SocketAddrV4,
}

#[cfg(feature = "std")]
mod if_std {
    use alloc::boxed::Box;
    use tokio::net::lookup_host;
    use super::*;
    // use crate::core::dns::queries::query_lookup;

    impl<'a> A {
        pub async fn new(host: Cow<'a, str>) -> Result<Self, DnsError> {
            let mut addresses = match lookup_host(&*host.clone()).await{
                Err(err) => { return Err(DnsError::LookupError(&*host.clone())) },
                Ok(socket_addrs_iter) => { socket_addrs_iter }
            };
            let ipv4_addrs = match addresses
                .filter(|x| x.is_ipv4())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(addrs) => addrs,
                None => return Err(DnsError::LookupIpv4Error(""))
            };

            return match ipv6_addrs {
                SocketAddr::V4(addrs) => {
                    Ok(Self { addrs: *addrs })
                }
                SocketAddr::V6(_) => {
                    Err(DnsError::LookupIpv4Error(""))
                }
            }
        }
    }
}

