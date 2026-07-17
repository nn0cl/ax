//! Pack progress UI.
//!
//! # Design (easy to debug)
//!
//! ```text
//!  compress / transfer threads          UI thread ("ax-progress")
//!  ─────────────────────────          ────────────────────────
//!  report()       ──┐
//!  report_worker() ─┼──► ProgressState ──► render_dashboard() ──► TtyBlock.paint()
//!                   │         (Mutex)           (pure)              (ANSI only)
//! ```
//!
//! - **Workers never touch the TTY** — they only update shared state.
//! - **UI thread** wakes every ~200ms, reads state, paints a **fixed-height**
//!   block (total + compress0..N + transfer) in place.
//! - **Width**: rows capped at **80 display columns** (Linux console baseline,
//!   ADR 0024); CJK ≈ 2 columns so lines do not wrap.
//!
//! Modules:
//! - [`rate`] — throughput EMA / worker slots
//! - [`width`] — column-aware truncate
//! - [`render`] — pure strings
//! - [`tty`] — ANSI cursor paint

mod rate;
mod render;
mod tty;
mod width;

use crate::core::ports::{PackProgress, ProgressSnapshot};
use rate::{RateSample, WorkerView};
use render::{render_dashboard, Layout};
use std::fs::OpenOptions;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tty::{show_cursor, TtyBlock};
use width::probe_terminal_columns;

/// Shared state between pack workers and the UI thread.
struct ProgressState {
    start: Instant,
    work_started: Option<Instant>,
    rate: RateSample,
    workers: Vec<WorkerView>,
    snap: Option<ProgressSnapshot>,
    jobs: usize,
    /// ADR 0024: effective paint width (≤ 80).
    layout: Layout,
}

impl ProgressState {
    fn new(jobs: usize, layout: Layout) -> Self {
        let now = Instant::now();
        Self {
            start: now,
            work_started: None,
            rate: RateSample::new(now),
            workers: vec![WorkerView::default(); jobs.max(1)],
            snap: None,
            jobs: jobs.max(1),
            layout,
        }
    }

    /// Build dashboard lines for the UI thread (needs &mut for rate update).
    fn dashboard_lines(&mut self) -> Option<Vec<String>> {
        let snap = self.snap.as_ref()?.clone();
        let now = Instant::now();
        let (in_bps, out_bps) =
            self.rate
                .observe(now, &mut self.work_started, &snap);
        let elapsed = self.start.elapsed().as_secs_f64().max(0.0);
        let mut lines =
            render_dashboard(&snap, elapsed, in_bps, out_bps, &self.workers, self.layout);
        let expect = self.jobs + 2;
        lines.resize(expect, String::new());
        Some(lines)
    }
}

struct Shared {
    state: Mutex<ProgressState>,
    out: Mutex<Box<dyn Write + Send>>,
    block: TtyBlock,
    stop: AtomicBool,
    /// Sticky log line being printed — UI skips paint.
    pause: AtomicBool,
}

/// Live progress dashboard (name kept for call-site stability).
pub struct SingleLineProgress {
    enabled: bool,
    shared: Option<Arc<Shared>>,
    ui: Mutex<Option<JoinHandle<()>>>,
}

impl SingleLineProgress {
    /// `want`: enable when a controlling TTY exists.
    /// `jobs`: compress worker rows (always shown).
    /// `_detail`: unused here (`-vv` text logs are handled in CLI).
    pub fn new(want: bool, _detail: bool, jobs: usize) -> Self {
        let out = open_tty_writer(want);
        let Some(out) = out else {
            return Self {
                enabled: false,
                shared: None,
                ui: Mutex::new(None),
            };
        };

        // ADR 0024: design at 80 cols; probe TTY and cap at 80.
        let layout = Layout::from_terminal_cols(probe_terminal_columns());
        let shared = Arc::new(Shared {
            state: Mutex::new(ProgressState::new(jobs, layout)),
            out: Mutex::new(out),
            block: TtyBlock::new(Duration::from_millis(150)),
            stop: AtomicBool::new(false),
            pause: AtomicBool::new(false),
        });

        let ui_shared = Arc::clone(&shared);
        let handle = thread::Builder::new()
            .name("ax-progress".into())
            .spawn(move || ui_loop(ui_shared))
            .ok();

        Self {
            enabled: true,
            shared: Some(shared),
            ui: Mutex::new(handle),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Print a sticky status line above/outside the dashboard.
    pub fn status_message(&self, msg: &str) {
        if !self.enabled {
            eprintln!("{msg}");
            let _ = io::stderr().flush();
            return;
        }
        let Some(shared) = self.shared.as_ref() else {
            eprintln!("{msg}");
            return;
        };

        shared.pause.store(true, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(25));

        if shared.block.is_painted() {
            show_cursor(&shared.out);
            if let Ok(mut out) = shared.out.lock() {
                let _ = writeln!(out);
                let _ = out.flush();
            }
            shared.block.reset_after_log();
        }
        if let Ok(mut out) = shared.out.lock() {
            let _ = writeln!(out, "{msg}");
            let _ = out.flush();
        }
        shared.pause.store(false, Ordering::SeqCst);
    }

    fn stop_ui(&self) {
        let Some(shared) = self.shared.as_ref() else {
            return;
        };
        shared.stop.store(true, Ordering::SeqCst);
        if let Ok(mut h) = self.ui.lock() {
            if let Some(t) = h.take() {
                let _ = t.join();
            }
        }
    }
}

fn open_tty_writer(want: bool) -> Option<Box<dyn Write + Send>> {
    if !want {
        return None;
    }
    if let Ok(tty) = OpenOptions::new().write(true).open("/dev/tty") {
        return Some(Box::new(tty));
    }
    if io::stderr().is_terminal() {
        return Some(Box::new(io::stderr()));
    }
    None
}

fn ui_loop(shared: Arc<Shared>) {
    let tick = Duration::from_millis(200);
    while !shared.stop.load(Ordering::Relaxed) {
        thread::sleep(tick);
        if shared.stop.load(Ordering::Relaxed) || shared.pause.load(Ordering::Relaxed) {
            continue;
        }
        paint_frame(&shared, false);
    }
    if !shared.pause.load(Ordering::Relaxed) {
        paint_frame(&shared, true);
    }
}

fn paint_frame(shared: &Shared, force: bool) {
    let lines = {
        let Ok(mut st) = shared.state.lock() else {
            return;
        };
        st.dashboard_lines()
    };
    let Some(lines) = lines else {
        return;
    };
    shared.block.paint(&shared.out, &lines, force);
}

impl PackProgress for SingleLineProgress {
    fn report(&self, snap: &ProgressSnapshot) {
        let Some(shared) = self.shared.as_ref() else {
            return;
        };
        if let Ok(mut st) = shared.state.lock() {
            st.snap = Some(snap.clone());
        }
    }

    fn report_worker(
        &self,
        worker_id: usize,
        phase: &'static str,
        primary: &str,
        source: &str,
        logical_delta: u64,
    ) {
        let Some(shared) = self.shared.as_ref() else {
            return;
        };
        if let Ok(mut st) = shared.state.lock() {
            if let Some(w) = st.workers.get_mut(worker_id) {
                w.tick(phase, primary, source, logical_delta);
            }
        }
    }

    fn finish(&self) {
        self.stop_ui();
        let Some(shared) = self.shared.as_ref() else {
            return;
        };
        shared.block.finish(&shared.out);
    }

    fn set_jobs(&self, jobs: usize) {
        let Some(shared) = self.shared.as_ref() else {
            return;
        };
        let n = jobs.max(1);
        if let Ok(mut st) = shared.state.lock() {
            if st.jobs == n {
                return;
            }
            st.jobs = n;
            st.workers.resize_with(n, WorkerView::default);
            // Force next paint to use new height (TtyBlock tracks prev rows).
            shared.block.reset_after_log();
        }
    }
}

impl Drop for SingleLineProgress {
    fn drop(&mut self) {
        self.stop_ui();
        if let Some(shared) = self.shared.as_ref() {
            show_cursor(&shared.out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_without_tty_request() {
        let p = SingleLineProgress::new(false, false, 4);
        assert!(!p.is_enabled());
    }
}
