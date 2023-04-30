use anyhow::Result;
use alloc::borrow::Cow;
use core::net::{SocketAddr, SocketAddrV4};
use tokio::net::lookup_host;
use crate::core::dns::queries::errors::DnsError;
use crate::Err;

pub mod a;
pub mod aaaa;
pub mod errors;
