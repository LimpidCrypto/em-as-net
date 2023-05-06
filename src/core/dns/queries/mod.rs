use crate::core::dns::queries::errors::DnsError;
use crate::Err;
use alloc::borrow::Cow;
use anyhow::Result;
use core::net::{SocketAddr, SocketAddrV4};
use tokio::net::lookup_host;

pub mod a;
pub mod aaaa;
pub mod errors;
