//! Multi-line terminal progress (aggregate + optional per-worker speeds).

mod draw;
mod format;
mod rate;

use crate::core::ports::{PackProgress, ProgressSnapshot};
use draw::draw_block;
use format::format_block;
use rate::{update_rates, update_worker, RateSample, WorkerView};
use std::fs::OpenOptions;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Overwrites a fixed block of terminal lines without scrolling a log.
pub struct SingleLineProgress {
    start: Instant,
    work_started: Mutex<Option<Instant>>,
    last_draw: Mutex<Instant>,
    rate: Mutex<RateSample>,
    workers: Mutex<Vec<WorkerView>>,
    show_workers: bool,
    enabled: bool,
    min_interval: Duration,
    drawn: AtomicBool,
    lines_drawn: Mutex<usize>,
    out: Mutex<Box<dyn Write + Send>>,
}

impl SingleLineProgress {
    /// `want`: enable UI when a TTY is available.
    /// `show_workers`: with `-v`, show one status line per compress job.
    /// `jobs`: number of compress worker slots (for `-v` layout).
    pub fn new(want: bool, show_workers: bool, jobs: usize) -> Self {
        let (enabled, out): (bool, Box<dyn Write + Send>) = if !want {
            (false, Box::new(io::sink()))
        } else if let Ok(tty) = OpenOptions::new().write(true).open("/dev/tty") {
            (true, Box::new(tty))
        } else if io::stderr().is_terminal() {
            (true, Box::new(io::stderr()))
        } else {
            (false, Box::new(io::sink()))
        };

        let jobs = jobs.max(1);
        let now = Instant::now();
        Self {
            start: now,
            work_started: Mutex::new(None),
            last_draw: Mutex::new(now - Duration::from_secs(1)),
            rate: Mutex::new(RateSample::new(now)),
            workers: Mutex::new(vec![WorkerView::default(); jobs]),
            show_workers: show_workers && enabled,
            enabled,
            min_interval: Duration::from_millis(100),
            drawn: AtomicBool::new(false),
            lines_drawn: Mutex::new(0),
            out: Mutex::new(out),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn status_message(&self, msg: &str) {
        if !self.enabled {
            eprintln!("{msg}");
            let _ = io::stderr().flush();
            return;
        }
        if self.drawn.load(Ordering::Relaxed) {
            return;
        }
        if let Ok(mut out) = self.out.lock() {
            let _ = writeln!(out, "{msg}");
            let _ = out.flush();
        }
    }
}

impl PackProgress for SingleLineProgress {
    fn report(&self, snap: &ProgressSnapshot) {
        if !self.enabled {
            return;
        }
        let (in_bps, out_bps) = update_rates(&self.rate, &self.work_started, snap);
        let elapsed = self.start.elapsed().as_secs_f64().max(0.0);
        let workers = self.workers.lock().unwrap_or_else(|e| e.into_inner());
        let lines = format_block(
            snap,
            elapsed,
            in_bps,
            out_bps,
            self.show_workers,
            &workers,
        );
        drop(workers);
        draw_block(
            &self.out,
            &self.last_draw,
            &self.drawn,
            &self.lines_drawn,
            self.min_interval,
            &lines,
            false,
        );
    }

    fn report_worker(
        &self,
        worker_id: usize,
        phase: &'static str,
        current: &str,
        logical_delta: u64,
    ) {
        if !self.show_workers {
            return;
        }
        if let Ok(mut workers) = self.workers.lock() {
            if worker_id >= workers.len() {
                return;
            }
            update_worker(&mut workers[worker_id], phase, current, logical_delta);
        }
    }

    fn finish(&self) {
        if !self.enabled {
            return;
        }
        if self.drawn.load(Ordering::Relaxed) {
            if let Ok(mut out) = self.out.lock() {
                let _ = writeln!(out);
                let _ = out.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_slots_allocated() {
        let p = SingleLineProgress::new(false, true, 4);
        assert_eq!(p.workers.lock().unwrap().len(), 4);
    }
}
