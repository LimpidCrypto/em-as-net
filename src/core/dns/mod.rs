use crate::core::dns::queries::errors::DnsError;
use crate::Err;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use anyhow::Result;
use core::net::SocketAddr;

pub mod queries;

pub async fn lookup(uri_str: Cow<str>) -> Result<SocketAddr> {
    #[cfg(all(feature = "ipv4", not(feature = "ipv6")))]
    use crate::core::dns::queries::a::A as Lookup;
    #[cfg(all(feature = "ipv6", not(feature = "ipv4")))]
    use crate::core::dns::queries::aaaa::Aaaa as Lookup;

    let addrs_res: Result<Lookup, DnsError> = Lookup::new(uri_str).await;
    let addrs = match addrs_res {
        Ok(addrs) => addrs.addrs,
        Err(err) => {
            return Err!(err);
        }
    };

    Ok(SocketAddr::from(addrs))
}
