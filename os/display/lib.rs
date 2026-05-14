#![cfg_attr(not(feature = "host"), no_std)]

extern crate alloc;

#[cfg(feature = "host")]
extern crate std;

pub mod apps;
pub mod client;
pub mod protocol;
pub mod server;