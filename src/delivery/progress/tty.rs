//! Terminal paint for a fixed-height block (ANSI cursor up + clear line).
//!
//! Contract after each paint: cursor sits on the blank line **below** the block.
//! Next paint moves up `n` rows and rewrites all `n` lines.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Paints a constant-height status block onto a locked writer.
pub struct TtyBlock {
    last_paint: Mutex<Instant>,
    painted: AtomicBool,
    rows: Mutex<usize>,
    min_interval: Duration,
}

impl TtyBlock {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_paint: Mutex::new(Instant::now() - Duration::from_secs(1)),
            painted: AtomicBool::new(false),
            rows: Mutex::new(0),
            min_interval,
        }
    }

    pub fn is_painted(&self) -> bool {
        self.painted.load(Ordering::Relaxed)
    }

    /// Reset paint state after a sticky log line was printed below the block.
    pub fn reset_after_log(&self) {
        self.painted.store(false, Ordering::Relaxed);
        if let Ok(mut n) = self.rows.lock() {
            *n = 0;
        }
    }

    pub fn paint(
        &self,
        out: &Mutex<Box<dyn Write + Send>>,
        lines: &[String],
        force: bool,
    ) {
        if lines.is_empty() {
            return;
        }
        if !force && !self.throttle_ok() {
            return;
        }

        let first = !self.painted.swap(true, Ordering::Relaxed);
        let mut out = out.lock().unwrap_or_else(|e| e.into_inner());
        let mut prev = self.rows.lock().unwrap_or_else(|e| e.into_inner());
        let n = lines.len();

        let _ = write!(out, "\x1b[?25l"); // hide cursor

        if !first {
            // Cursor is on the line below the previous block.
            let up = (*prev).max(1);
            let _ = write!(out, "\x1b[{up}A");
        }

        for line in lines {
            let _ = write!(out, "\r\x1b[2K{line}");
            let _ = writeln!(out);
        }

        if !first && *prev > n {
            let extra = *prev - n;
            for _ in 0..extra {
                let _ = write!(out, "\r\x1b[2K");
                let _ = writeln!(out);
            }
            if extra > 0 {
                let _ = write!(out, "\x1b[{extra}A");
            }
        }

        *prev = n;
        let _ = out.flush();
    }

    pub fn finish(&self, out: &Mutex<Box<dyn Write + Send>>) {
        show_cursor(out);
        if self.painted.load(Ordering::Relaxed) {
            if let Ok(mut out) = out.lock() {
                let _ = writeln!(out);
                let _ = out.flush();
            }
        }
    }

    fn throttle_ok(&self) -> bool {
        let now = Instant::now();
        let Ok(mut last) = self.last_paint.lock() else {
            return true;
        };
        if now.duration_since(*last) < self.min_interval {
            return false;
        }
        *last = now;
        true
    }
}

pub fn show_cursor(out: &Mutex<Box<dyn Write + Send>>) {
    if let Ok(mut out) = out.lock() {
        let _ = write!(out, "\x1b[?25h");
        let _ = out.flush();
    }
}
