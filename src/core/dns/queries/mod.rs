mod a;

pub use a::A;
mod aaaa;
pub use aaaa::Aaaa;
mod exceptions;
pub use exceptions::DnsException;

use anyhow::Result;
use url::Url;

pub trait Lookup<T> {
    async fn lookup(url: &Url) -> Result<T>;
}
