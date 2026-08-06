//! The safe public boundary of `alloy-kernel-unsafe-core`.
//!
//! This is the only surface the safe kernel and the HAL are allowed to use.
//! Every item here must be callable from safe code with no UB possible; raw
//! pointers never cross this boundary.
//!
//! Planned submodules:
//! io, mem, interrupt, serial, time, drivers, alloc, sync, arch, callback.
