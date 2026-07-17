//! TTY block redraw (cursor up + clear lines).

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Draw a multi-line status block, overwriting the previous block in place.
pub fn draw_block(
    out: &Mutex<Box<dyn Write + Send>>,
    last_draw: &Mutex<Instant>,
    drawn: &AtomicBool,
    lines_drawn: &Mutex<usize>,
    min_interval: Duration,
    lines: &[String],
    force: bool,
) {
    if lines.is_empty() {
        return;
    }
    let now = Instant::now();
    if !force {
        if let Ok(mut last) = last_draw.lock() {
            if now.duration_since(*last) < min_interval {
                return;
            }
            *last = now;
        }
    }

    let first = !drawn.swap(true, Ordering::Relaxed);
    let mut out = out.lock().unwrap_or_else(|e| e.into_inner());
    let mut prev_n = lines_drawn.lock().unwrap_or_else(|e| e.into_inner());

    if !first && *prev_n > 0 {
        let up = prev_n.saturating_sub(1);
        if up > 0 {
            let _ = write!(out, "\x1b[{up}A");
        }
    }

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            let _ = write!(out, "\r\x1b[2K{line}");
        } else {
            let _ = write!(out, "\n\x1b[2K{line}");
        }
    }

    if !first && *prev_n > lines.len() {
        for _ in 0..(*prev_n - lines.len()) {
            let _ = write!(out, "\n\x1b[2K");
        }
        let extra = *prev_n - lines.len();
        if extra > 0 {
            let _ = write!(out, "\x1b[{extra}A");
        }
    }

    *prev_n = lines.len();
    let _ = out.flush();
}
