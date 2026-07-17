//! Multi-line terminal progress (aggregate + optional per-worker speeds).

use crate::core::domain::human_bytes;
use crate::core::ports::{PackProgress, ProgressSnapshot};
use std::fs::OpenOptions;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct RateSample {
    at: Instant,
    logical: u64,
    written: u64,
    in_bps: u64,
    out_bps: u64,
}

#[derive(Clone)]
struct WorkerView {
    phase: &'static str,
    current: String,
    logical: u64,
    rate_at: Instant,
    rate_logical: u64,
    bps: u64,
    active: bool,
}

impl Default for WorkerView {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            phase: "idle",
            current: String::new(),
            logical: 0,
            rate_at: now,
            rate_logical: 0,
            bps: 0,
            active: false,
        }
    }
}

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
            rate: Mutex::new(RateSample {
                at: now,
                logical: 0,
                written: 0,
                in_bps: 0,
                out_bps: 0,
            }),
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

    fn update_rates(&self, snap: &ProgressSnapshot) -> (u64, u64) {
        let now = Instant::now();
        let mut rate = self.rate.lock().unwrap_or_else(|e| e.into_inner());

        if matches!(snap.phase, "hash" | "compress" | "transfer" | "write-map" | "done" | "start")
            && (snap.logical_done > 0 || snap.written_done > 0 || snap.phase == "hash")
        {
            if let Ok(mut ws) = self.work_started.lock() {
                if ws.is_none() && (snap.phase == "hash" || snap.phase == "compress") {
                    *ws = Some(now);
                    rate.at = now;
                    rate.logical = snap.logical_done;
                    rate.written = snap.written_done;
                }
            }
        }

        let dt = now.duration_since(rate.at).as_secs_f64();
        if dt >= 0.25 {
            let d_in = snap.logical_done.saturating_sub(rate.logical);
            let d_out = snap.written_done.saturating_sub(rate.written);
            let inst_in = (d_in as f64 / dt) as u64;
            let inst_out = (d_out as f64 / dt) as u64;
            rate.in_bps = if rate.in_bps == 0 {
                inst_in
            } else {
                (rate.in_bps * 3 + inst_in) / 4
            };
            rate.out_bps = if rate.out_bps == 0 {
                inst_out
            } else {
                (rate.out_bps * 3 + inst_out) / 4
            };
            rate.at = now;
            rate.logical = snap.logical_done;
            rate.written = snap.written_done;
        }

        let in_bps = if rate.in_bps > 0 {
            rate.in_bps
        } else if let Ok(ws) = self.work_started.lock() {
            if let Some(t0) = *ws {
                let e = now.duration_since(t0).as_secs_f64().max(1e-6);
                (snap.logical_done as f64 / e) as u64
            } else {
                0
            }
        } else {
            0
        };
        let out_bps = if rate.out_bps > 0 {
            rate.out_bps
        } else if let Ok(ws) = self.work_started.lock() {
            if let Some(t0) = *ws {
                let e = now.duration_since(t0).as_secs_f64().max(1e-6);
                (snap.written_done as f64 / e) as u64
            } else {
                0
            }
        } else {
            0
        };
        (in_bps, out_bps)
    }

    fn format_block(&self, snap: &ProgressSnapshot) -> Vec<String> {
        let (in_bps, out_bps) = self.update_rates(snap);
        let elapsed = self.start.elapsed().as_secs_f64().max(0.0);
        let time = format_time_fields(elapsed, eta_from_progress(snap, in_bps));

        let in_pct = if snap.logical_total > 0 {
            (100.0 * snap.logical_done as f64 / snap.logical_total as f64).min(100.0)
        } else {
            0.0
        };
        let unit_pct = if snap.units_total > 0 {
            100.0 * snap.units_done as f64 / snap.units_total as f64
        } else {
            0.0
        };
        // Slightly shorter path when -v (more lines) so elapsed/ETA still fit.
        let cur_w = if self.show_workers { 28 } else { 32 };
        let current = truncate_middle(&snap.current, cur_w);

        // Scan / plan: keep compact 2-line layout (no workers).
        if snap.phase == "scan" || snap.phase == "plan" {
            return self.format_scan_plan(snap, elapsed, unit_pct, &current, &time);
        }

        let mut lines = Vec::new();

        // Aggregate line (elapsed + ETA always; -v only adds worker lines)
        let agg = match snap.phase {
            "hash" => format!(
                "total     {done}/{total} units  {pct:5.1}%  {speed}/s  {time}  [hash]  {current}",
                done = snap.units_done,
                total = snap.units_total,
                pct = in_pct,
                speed = human_bytes(in_bps),
                time = time,
                current = current,
            ),
            "transfer" => format!(
                "total     {done}/{total} units  {pct:5.1}%  in {speed_in}/s  {time}  [pipeline]  {current}",
                done = snap.units_done,
                total = snap.units_total,
                pct = in_pct,
                speed_in = human_bytes(in_bps),
                time = time,
                current = current,
            ),
            "done" => format!(
                "total     {done}/{total} units  100.0%  {time}  [done]",
                done = snap.units_done,
                total = snap.units_total,
                time = time,
            ),
            _ => format!(
                "total     {done}/{total} units  {pct:5.1}%  {speed}/s  {time}  [{phase}]  {current}",
                done = snap.units_done,
                total = snap.units_total,
                pct = in_pct,
                speed = human_bytes(in_bps),
                time = time,
                phase = snap.phase,
                current = current,
            ),
        };
        lines.push(agg);

        if self.show_workers {
            if let Ok(workers) = self.workers.lock() {
                for (i, w) in workers.iter().enumerate() {
                    if !w.active && w.phase == "idle" {
                        lines.push(format!("compress{i}  idle"));
                    } else {
                        let cur = truncate_middle(&w.current, 28);
                        lines.push(format!(
                            "compress{i}  {speed}/s  [{phase}]  {cur}",
                            speed = human_bytes(w.bps),
                            phase = w.phase,
                            cur = cur,
                        ));
                    }
                }
            }
        }

        let transfer = format!(
            "transfer  {written} written  {unit_pct:5.1}%  {speed}/s  {time}  {current}",
            written = human_bytes(snap.written_done),
            unit_pct = unit_pct,
            speed = human_bytes(out_bps),
            time = time,
            current = current,
        );
        lines.push(transfer);
        lines
    }

    fn format_scan_plan(
        &self,
        snap: &ProgressSnapshot,
        scan_elapsed: f64,
        unit_pct: f64,
        current: &str,
        time: &str,
    ) -> Vec<String> {
        if snap.phase == "scan" {
            let files_per_sec = (snap.units_done as f64 / scan_elapsed.max(1e-6)) as u64;
            let bps = (snap.logical_done as f64 / scan_elapsed.max(1e-6)) as u64;
            let line1 = format!(
                "scan      {files} files  {bytes}  {fps} files/s  {bps}/s  {time}  {current}",
                files = snap.units_done,
                bytes = human_bytes(snap.logical_done),
                fps = files_per_sec,
                bps = human_bytes(bps),
                time = time,
                current = current,
            );
            let line2 = if snap.current.contains("indexing") || snap.current.contains("sort") {
                "transfer  waiting (post-scan index / sort…)".to_string()
            } else if snap.current.contains("scan complete") {
                "transfer  waiting (planning pack units…)".to_string()
            } else {
                "transfer  waiting (scanning source tree…)".to_string()
            };
            vec![line1, line2]
        } else {
            let line1 = if snap.units_total > 0 {
                format!(
                    "plan      {done}/{total} dirs  {pct:5.1}%  {time}  {current}",
                    done = snap.units_done,
                    total = snap.units_total,
                    pct = unit_pct,
                    time = time,
                    current = current,
                )
            } else {
                format!("plan      {time}  {current}", time = time, current = current)
            };
            vec![
                line1,
                "transfer  waiting (planning pack units…)".to_string(),
            ]
        }
    }

    fn draw_block(&self, lines: &[String], force: bool) {
        if !self.enabled || lines.is_empty() {
            return;
        }
        let now = Instant::now();
        if !force {
            if let Ok(mut last) = self.last_draw.lock() {
                if now.duration_since(*last) < self.min_interval {
                    return;
                }
                *last = now;
            }
        }

        let first = !self.drawn.swap(true, Ordering::Relaxed);
        let mut out = self.out.lock().unwrap_or_else(|e| e.into_inner());
        let mut prev_n = self.lines_drawn.lock().unwrap_or_else(|e| e.into_inner());

        if !first && *prev_n > 0 {
            // Move cursor to the first line of the previous block.
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

        // If new block is shorter, clear leftover lines from previous taller block.
        if !first && *prev_n > lines.len() {
            for _ in 0..(*prev_n - lines.len()) {
                let _ = write!(out, "\n\x1b[2K");
            }
            // Move back up to end of new block.
            let extra = *prev_n - lines.len();
            if extra > 0 {
                let _ = write!(out, "\x1b[{extra}A");
            }
        }

        *prev_n = lines.len();
        let _ = out.flush();
    }
}

impl PackProgress for SingleLineProgress {
    fn report(&self, snap: &ProgressSnapshot) {
        let lines = self.format_block(snap);
        self.draw_block(&lines, false);
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
        let now = Instant::now();
        if let Ok(mut workers) = self.workers.lock() {
            if worker_id >= workers.len() {
                return;
            }
            let w = &mut workers[worker_id];
            w.phase = phase;
            w.current = current.to_string();
            w.active = phase != "idle";
            w.logical = w.logical.saturating_add(logical_delta);

            let dt = now.duration_since(w.rate_at).as_secs_f64();
            if dt >= 0.2 {
                let d = w.logical.saturating_sub(w.rate_logical);
                let inst = (d as f64 / dt) as u64;
                w.bps = if w.bps == 0 {
                    inst
                } else {
                    (w.bps * 3 + inst) / 4
                };
                w.rate_at = now;
                w.rate_logical = w.logical;
            }
        }
        // Trigger redraw with last aggregate if possible — callers also report().
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

fn truncate_middle(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        return s.to_string();
    }
    if max < 5 {
        return chars.into_iter().take(max).collect();
    }
    let head = (max - 1) / 2;
    let tail = max - 1 - head;
    let mut out: String = chars.iter().take(head).collect();
    out.push('…');
    out.extend(chars.iter().skip(chars.len() - tail));
    out
}

/// Compact duration for status lines: `45s`, `12m34s`, `1h05m`.
fn format_duration(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "--".into();
    }
    let total = secs.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}h{m:02}m")
    } else if m > 0 {
        format!("{m}m{s:02}s")
    } else {
        format!("{s}s")
    }
}

/// `elapsed 1m23s  ETA 4m12s` or `elapsed 1m23s  ETA --` when rate unknown.
fn format_time_fields(elapsed_secs: f64, eta_secs: Option<f64>) -> String {
    let elapsed = format_duration(elapsed_secs);
    match eta_secs {
        Some(e) if e.is_finite() && e >= 0.0 => {
            format!("elapsed {elapsed}  ETA {}", format_duration(e))
        }
        _ => format!("elapsed {elapsed}  ETA --"),
    }
}

/// ETA from logical-byte progress and smoothed input rate (bytes/sec).
fn eta_from_progress(snap: &ProgressSnapshot, in_bps: u64) -> Option<f64> {
    if matches!(snap.phase, "done") {
        return Some(0.0);
    }
    if snap.logical_total == 0 || in_bps == 0 {
        // Fall back to unit-rate once we have unit progress and elapsed work.
        if snap.units_total > 0 && snap.units_done > 0 && snap.units_done < snap.units_total {
            // Caller doesn't pass elapsed here; unit-only ETA needs rate from done/total + bps.
            // Without a time base for units, skip unless logical works.
        }
        return None;
    }
    if snap.logical_done >= snap.logical_total {
        return Some(0.0);
    }
    let remaining = snap.logical_total - snap.logical_done;
    Some(remaining as f64 / in_bps as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_names() {
        let s = truncate_middle("docs/very/long/path/part-0001.tar.zst", 20);
        assert!(s.chars().count() <= 20);
        assert!(s.contains('…'));
    }

    #[test]
    fn worker_slots_allocated() {
        let p = SingleLineProgress::new(false, true, 4);
        assert_eq!(p.workers.lock().unwrap().len(), 4);
    }

    #[test]
    fn duration_formats() {
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(45.0), "45s");
        assert_eq!(format_duration(83.0), "1m23s");
        assert_eq!(format_duration(3725.0), "1h02m");
    }

    #[test]
    fn eta_from_half_done() {
        let snap = ProgressSnapshot {
            units_done: 1,
            units_total: 2,
            logical_done: 50,
            logical_total: 100,
            written_done: 0,
            current: String::new(),
            phase: "compress",
        };
        // 50 bytes left at 10 B/s → 5s
        assert_eq!(eta_from_progress(&snap, 10), Some(5.0));
        assert_eq!(eta_from_progress(&snap, 0), None);
    }

    #[test]
    fn time_fields_include_elapsed_and_eta() {
        let s = format_time_fields(83.0, Some(125.0));
        assert!(s.contains("elapsed 1m23s"));
        assert!(s.contains("ETA 2m05s"));
        let unknown = format_time_fields(10.0, None);
        assert!(unknown.contains("ETA --"));
    }
}
