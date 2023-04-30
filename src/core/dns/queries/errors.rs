use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DnsError<'a> {
    #[error("Unable to look up hostname: {0:?}")]
    LookupError(&'a str),
    #[error("Unable to look up IPv4 address for hostname: {0:?}")]
    LookupIpv4Error(&'a str),
    #[error("Unable to look up IPv6 address for hostname: {0:?}")]
    LookupIpv6Error(&'a str),
}

#[cfg(feature = "std")]
impl alloc::error::Error for DnsError<'_> {}
