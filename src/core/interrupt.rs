//! Process-wide cooperative cancel flag (set by delivery on Ctrl+C).

use crate::core::domain::ArchiveError;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static CANCELLED: AtomicBool = AtomicBool::new(false);
/// How many times the user hit Ctrl+C (2+ ⇒ force `_exit`).
static INTERRUPT_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Called from the Ctrl+C handler.
///
/// - 1st press: set cancel flag (cooperative shutdown).
/// - 2nd+ press: immediate `_exit(130)` (async-signal-safe; does not run destructors).
///
/// Returns `true` if this was the first press (caller may print a message).
pub fn on_ctrl_c() -> bool {
    let n = INTERRUPT_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    CANCELLED.store(true, Ordering::SeqCst);
    if n >= 2 {
        // Do not use std::process::exit here: it can re-enter runtime locks from a
        // signal context. libc::_exit is async-signal-safe.
        force_exit_130();
    }
    n == 1
}

/// Mark cancelled without force-exit (tests / internal).
pub fn request_cancel() {
    CANCELLED.store(true, Ordering::SeqCst);
    let _ = INTERRUPT_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[inline]
pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::Relaxed)
}

#[inline]
pub fn interrupt_count() -> usize {
    INTERRUPT_COUNT.load(Ordering::Relaxed)
}

#[inline]
pub fn check() -> Result<(), ArchiveError> {
    if is_cancelled() {
        Err(ArchiveError::Interrupted)
    } else {
        Ok(())
    }
}

fn force_exit_130() -> ! {
    // Prefer raw write + _exit so we never block on buffered I/O locks.
    const MSG: &[u8] = b"\n*** force exit (Ctrl+C x2) ***\n";
    unsafe {
        let _ = libc::write(2, MSG.as_ptr().cast(), MSG.len());
        libc::_exit(130);
    }
}
