use alloc::borrow::Cow;
use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DnsError<'a> {
    #[error("Invalid socket address (found: {0:?})")]
    LookupError(Cow<'a, str>),
    #[error("Unable to look up IPv4 address for hostname (found: {0:?})")]
    LookupIpv4Error(Cow<'a, str>),
    #[error("Unable to look up IPv6 address for hostname (found: {0:?})")]
    LookupIpv6Error(Cow<'a, str>),
}

#[cfg(feature = "std")]
impl alloc::error::Error for DnsError<'_> {}
