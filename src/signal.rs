//! SIGTERM/SIGINT → clean-exit flag.
//!
//! systemd stops the kiosk session with SIGTERM; without a handler the
//! process dies mid-frame with no `shutdown` call. The handler here only
//! sets an `AtomicBool` (the only async-signal-safe thing worth doing);
//! the frame loop polls [`should_exit`] once per frame.
//!
//! Implemented against `signal(2)` from the libc that `std` already links,
//! declared by hand: the `libc` crate was weighed and rejected to keep this
//! crate at zero external dependencies (`docs/DESIGN.md` §9). `sigaction`
//! semantics (SA_RESTART control) are deliberately deferred until a
//! concrete need appears.

// Everything here is consumed by the frame loop (`Kiosk::run`), which is
// still a stub; drop this allow when the loop lands.
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

/// True once SIGTERM or SIGINT has been received.
pub(crate) fn should_exit() -> bool {
    SHOULD_EXIT.load(Ordering::Relaxed)
}

#[cfg(unix)]
mod imp {
    use super::SHOULD_EXIT;
    use std::sync::atomic::Ordering;

    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;

    extern "C" fn handle(_signum: i32) {
        // Async-signal-safe: a relaxed atomic store, nothing else.
        SHOULD_EXIT.store(true, Ordering::Relaxed);
    }

    extern "C" {
        fn signal(signum: i32, handler: extern "C" fn(i32)) -> isize;
    }

    pub(crate) fn install() {
        unsafe {
            signal(SIGTERM, handle);
            signal(SIGINT, handle);
        }
    }
}

#[cfg(not(unix))]
mod imp {
    /// No-op on non-unix development hosts.
    pub(crate) fn install() {}
}

/// Install the handlers. Called once by `Kiosk::run`.
#[allow(unused_imports)]
pub(crate) use imp::install;
