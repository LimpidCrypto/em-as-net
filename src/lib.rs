#![no_std]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![feature(ip_in_core)]
#![allow(dead_code)] // Remove eventually

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod constants;
// #[cfg(feature = "core")]
pub mod core;
// #[cfg(feature = "client")]
pub mod client;
pub mod utils;

mod _anyhow;

#[cfg(all(feature = "ipv4", feature = "ipv6"))]
compile_error!("The `ipv4` and `ipv6` features are mutually exclusive and cannot be enabled at the same time");
