//! Process-wide cooperative cancel flag (set by delivery on Ctrl+C).

use crate::core::domain::ArchiveError;
use std::sync::atomic::{AtomicBool, Ordering};

static CANCELLED: AtomicBool = AtomicBool::new(false);

/// Mark the process as cancelled (called from signal handler).
pub fn request_cancel() {
    CANCELLED.store(true, Ordering::SeqCst);
}

#[inline]
pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::Relaxed)
}

#[inline]
pub fn check() -> Result<(), ArchiveError> {
    if is_cancelled() {
        Err(ArchiveError::Interrupted)
    } else {
        Ok(())
    }
}
