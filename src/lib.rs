#![cfg_attr(not(test), no_std)]
#![feature(type_alias_impl_trait)]
//! Unstable
#![feature(associated_type_defaults)]
#![feature(never_type)]

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
