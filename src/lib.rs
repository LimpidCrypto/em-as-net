#![cfg_attr(not(test), no_std)]
#![feature(type_alias_impl_trait)]
// #![feature(async_fn_in_trait)]

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
