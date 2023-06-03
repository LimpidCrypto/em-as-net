mod a;

pub use a::A;
use alloc::borrow::Cow;
mod aaaa;
pub use aaaa::Aaaa;
mod errors;
pub use errors::DnsError;

use anyhow::Result;

pub trait Lookup<'a, T> {
    async fn lookup(url: Cow<'a, str>) -> Result<T>;
}
