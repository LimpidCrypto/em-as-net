mod queries;

use crate::Err;
use alloc::borrow::Cow;
use anyhow::Result;
use core::net::SocketAddr;
pub use queries::DnsError;

#[cfg(all(feature = "ipv4", not(feature = "ipv6")))]
pub async fn lookup(uri_str: Cow<'_, str>) -> Result<SocketAddr> {
    use self::queries::A;

    let addrs_res: Result<A, DnsError> = A::new(uri_str).await;
    let addrs = match addrs_res {
        Ok(addrs) => addrs.addrs,
        Err(err) => {
            return Err!(err);
        }
    };

    Ok(SocketAddr::from(addrs))
}

#[cfg(all(feature = "ipv6", not(feature = "ipv4")))]
pub async fn lookup(uri_str: Cow<'_, str>) -> Result<SocketAddr> {
    use self::queries::Aaaa;

    let addrs_res: Result<Aaaa, DnsError> = Aaaa::new(uri_str).await;
    let addrs = match addrs_res {
        Ok(addrs) => addrs.addrs,
        Err(err) => {
            return Err!(err);
        }
    };

    Ok(SocketAddr::from(addrs))
}
