use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::net::{SocketAddr, SocketAddrV6};
use super::errors::DnsError;

#[derive(Debug)]
pub struct Aaaa {
    pub(crate) addrs: SocketAddrV6,
}

#[cfg(feature = "std")]
mod if_std {
    use alloc::boxed::Box;
    use core::net::{IpAddr, Ipv6Addr};
    use super::*;
    use tokio::net::lookup_host;
    // use crate::core::dns::queries::query_lookup;

    impl<'a> Aaaa {
        pub async fn new(host: Cow<'a, str>) -> Result<Self, DnsError> {
            let mut addresses = match lookup_host(&*host.clone()).await{
                Err(err) => { return Err(DnsError::LookupError(&*host.clone())) },
                Ok(socket_addrs_iter) => { socket_addrs_iter }
            };
            let ipv6_addrs = match addresses
                .filter(|x| x.is_ipv6())
                .collect::<Vec<SocketAddr>>()
                .first()
            {
                Some(addrs) => addrs,
                None => return Err(DnsError::LookupIpv4Error(""))
            };

            return match ipv6_addrs {
                SocketAddr::V4(_) => {
                    Err(DnsError::LookupIpv6Error(""))
                }
                SocketAddr::V6(addrs) => {
                    Ok(Self { addrs: *addrs })
                }
            }
        }
    }
}
