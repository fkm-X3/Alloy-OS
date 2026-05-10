#![cfg_attr(not(feature = "host"), no_std)]

#[cfg(feature = "host")]
extern crate std;

extern crate alloc;

pub mod apps;
pub mod client;
pub mod protocol;
pub mod server;
