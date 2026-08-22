//! Serial render-pipeline tracing
//!
//! Lightweight instrumentation along the attach → commit → composite path.
//! Every line is prefixed `[RenderTrace]` and stamped with a monotonic
//! sequence number plus uptime in milliseconds, so boot logs can be
//! correlated with the deterministic `test_shm_client` gradient frames.
//!
//! Trace points live at:
//! - `rust_sys_alloc_shm` / `rust_sys_shm_user_vaddr` (syscall entry)
//! - `shm_alloc::shm_user_vaddr` (mapping side effects)
//! - `buffer_handler` (wl_shm.create_pool / wl_shm_pool.create_buffer)
//! - `compositor_handler` (surface attach / damage / commit)
//! - `surface::commit` (pending → current promotion)
//! - `compositor_integration::composite_frame` (per-surface composite decisions)
//! - `display_server::run` (main loop liveness / starvation evidence)

use core::sync::atomic::{AtomicU32, Ordering};

static TRACE_SEQ: AtomicU32 = AtomicU32::new(0);

/// Emit one `[RenderTrace]` line with sequence number and uptime stamp.
pub fn emit(args: core::fmt::Arguments) {
    let seq = TRACE_SEQ.fetch_add(1, Ordering::Relaxed);
    let ts = crate::SystemTimer::uptime_ms();
    crate::println!("[RenderTrace #{:06} {:>10}ms] {}", seq, ts, args);
}

/// Returns true on every `n`-th call (`n > 0`), for rate-limiting hot paths.
pub fn every_nth(counter: &AtomicU32, n: u32) -> bool {
    n > 0 && counter.fetch_add(1, Ordering::Relaxed) % n == 0
}
