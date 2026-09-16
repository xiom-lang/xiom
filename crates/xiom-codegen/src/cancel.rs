// XIOM -- Cooperative cancellation token (audit #12)
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Process-wide cooperative cancellation for long compilations.
//!
//! The CLI watchdog threads (timeout / memory budget) used to call
//! `std::process::exit(1)` from a worker thread: no unwinding, atexit
//! handlers racing the compiler's own cleanup, and embedders of the library
//! (LSP/MCP) lost control. The watchdog now sets this token instead; the
//! codegen pipeline observes it between functions and the driver's clang
//! child is killed, after which the MAIN thread reports the error and exits.
//!
//! Reset once at process start (`reset()`). Embedders that do not use the
//! token are unaffected (it stays false).

use std::sync::atomic::{AtomicBool, Ordering};

static CANCELLED: AtomicBool = AtomicBool::new(false);

/// Request cancellation; safe from any thread. Idempotent.
pub fn request_cancel() {
    CANCELLED.store(true, Ordering::SeqCst);
}

/// True when a watchdog has requested cancellation.
pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::Relaxed)
}

/// Clear the token (process start / between independent compilations in a
/// single embedder process).
pub fn reset() {
    CANCELLED.store(false, Ordering::SeqCst);
}
