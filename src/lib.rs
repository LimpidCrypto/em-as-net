#![no_std]
// #![cfg_attr(not(feature = "std"), no_std)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(ip_in_core)]
#![allow(dead_code)] // Remove eventually

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

#[cfg(any(feature = "json-rpc", feature = "websocket"))]
pub mod client;
pub mod constants;
pub mod core;
pub mod utils;

mod _anyhow;
