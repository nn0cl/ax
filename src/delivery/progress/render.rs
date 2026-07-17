//! Pure string rendering for the progress dashboard (no I/O, no locks).
//!
//! Layout (fixed height = `workers.len() + 2`):
//! ```text
//! total     …
//! compress0 …
//! compress1 …
//! …
//! transfer  …
//! ```

use super::rate::WorkerView;
use super::width::{fit_row, truncate_middle, PATH_COLS};
use crate::core::domain::{eta_wall_clock, human_bytes, write_progress_pct, EtaBreakdown};
use crate::core::ports::ProgressSnapshot;

const MAX_ETA_SECS: f64 = 48.0 * 3600.0;

/// Build the fixed-height dashboard lines from a snapshot + worker rows.
pub fn render_dashboard(
    snap: &ProgressSnapshot,
    elapsed: f64,
    in_bps: u64,
    out_bps: u64,
    workers: &[WorkerView],
) -> Vec<String> {
    let time = format_clock(elapsed, snap, in_bps, out_bps);
    let path = truncate_middle(&snap.current, PATH_COLS);

    if snap.phase == "scan" || snap.phase == "plan" {
        return render_scan_or_plan(snap, elapsed, &path, &time, workers.len());
    }

    let in_pct = pct(snap.logical_done, snap.logical_total);
    let xfer_pct = write_progress_pct(
        snap.logical_total,
        snap.written_done,
        snap.logical_transferred,
        prior(snap),
    );

    let mut lines = Vec::with_capacity(workers.len() + 2);
    lines.push(fit_row(format!(
        "total     {done}/{total} units  {pct:5.1}%  in {speed}/s  {time}  [{phase}]  {path}",
        done = snap.units_done,
        total = snap.units_total,
        pct = in_pct,
        speed = human_bytes(in_bps),
        time = time,
        phase = snap.phase,
        path = path,
    )));

    for (i, w) in workers.iter().enumerate() {
        lines.push(render_worker_line(i, w));
    }

    let xfer_path = if snap.phase == "transfer" {
        path.as_str()
    } else {
        "—"
    };
    lines.push(fit_row(format!(
        "transfer  {written} written  {xp:5.1}%  {os}/s  {time}  {xfer_path}",
        written = human_bytes(snap.written_done),
        xp = xfer_pct,
        os = human_bytes(out_bps),
        time = time,
        xfer_path = xfer_path,
    )));

    debug_assert_eq!(lines.len(), workers.len() + 2);
    lines
}

fn render_worker_line(i: usize, w: &WorkerView) -> String {
    if !w.active && (w.phase == "idle" || w.phase.is_empty()) {
        return format!("compress{i}  idle");
    }
    let cur = truncate_middle(&w.current, PATH_COLS);
    let phase = if w.phase.is_empty() { "…" } else { w.phase };
    fit_row(format!(
        "compress{i}  {speed}/s  [{phase}]  {cur}",
        speed = human_bytes(w.bps),
        phase = phase,
        cur = cur,
    ))
}

fn render_scan_or_plan(
    snap: &ProgressSnapshot,
    elapsed: f64,
    path: &str,
    time: &str,
    worker_slots: usize,
) -> Vec<String> {
    let mut lines = Vec::with_capacity(worker_slots + 2);
    if snap.phase == "scan" {
        let files_per_sec = (snap.units_done as f64 / elapsed.max(1e-6)) as u64;
        let bps = (snap.logical_done as f64 / elapsed.max(1e-6)) as u64;
        lines.push(fit_row(format!(
            "total     scan  {files} files  {bytes}  {fps}/s  {bps}/s  {time}  {path}",
            files = snap.units_done,
            bytes = human_bytes(snap.logical_done),
            fps = files_per_sec,
            bps = human_bytes(bps),
            time = time,
            path = path,
        )));
    } else {
        let plan_pct = pct(snap.units_done as u64, snap.units_total as u64);
        lines.push(fit_row(format!(
            "total     plan  {done}/{total} dirs  {pct:5.1}%  {time}  {path}",
            done = snap.units_done,
            total = snap.units_total,
            pct = plan_pct,
            time = time,
            path = path,
        )));
    }
    for i in 0..worker_slots {
        lines.push(format!("compress{i}  idle"));
    }
    lines.push("transfer  waiting…".into());
    lines
}

fn format_clock(elapsed: f64, snap: &ProgressSnapshot, in_bps: u64, out_bps: u64) -> String {
    let out_for_eta = if snap.written_done == 0
        && matches!(
            snap.phase,
            "hash" | "compress" | "verify-stage" | "start" | "scan" | "plan"
        ) {
        0
    } else {
        out_bps
    };
    let eta = eta_wall_clock(
        snap.phase,
        snap.logical_done,
        snap.logical_total,
        snap.written_done,
        snap.logical_transferred,
        in_bps,
        out_for_eta,
        prior(snap),
    );
    format_time_fields(elapsed, eta)
}

fn prior(snap: &ProgressSnapshot) -> f64 {
    if snap.ratio_prior.is_finite() && snap.ratio_prior > 0.0 {
        snap.ratio_prior
    } else {
        1.0
    }
}

fn pct(done: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (100.0 * done as f64 / total as f64).min(100.0)
    }
}

pub fn format_duration(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "--".into();
    }
    if secs > MAX_ETA_SECS {
        return "long".into();
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

fn format_time_fields(elapsed_secs: f64, eta: EtaBreakdown) -> String {
    let elapsed = format_duration(elapsed_secs.min(MAX_ETA_SECS * 10.0));
    match eta.wall {
        Some(w) if w.is_finite() && (0.0..MAX_ETA_SECS).contains(&w) => {
            format!("elapsed {elapsed}  ETA {}", format_duration(w))
        }
        _ => format!("elapsed {elapsed}  ETA --"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::width::display_width;
    use super::super::width::ROW_COLS;

    #[test]
    fn duration_formats() {
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(45.0), "45s");
        assert_eq!(format_duration(83.0), "1m23s");
        assert_eq!(format_duration(3725.0), "1h02m");
    }

    #[test]
    fn dashboard_height_fixed() {
        let workers = vec![WorkerView::default(); 4];
        let snap = ProgressSnapshot {
            units_done: 1,
            units_total: 31,
            logical_done: 1000,
            logical_total: 10_000,
            logical_transferred: 0,
            written_done: 0,
            current: "Documents/part-0001.zip".into(),
            phase: "hash",
            ratio_prior: 0.7,
        };
        let lines = render_dashboard(&snap, 10.0, 1_000_000, 0, &workers);
        assert_eq!(lines.len(), 6);
        assert!(lines[0].starts_with("total"));
        assert!(lines[1].starts_with("compress0"));
        assert!(lines[5].starts_with("transfer"));
        for line in &lines {
            assert!(display_width(line) <= ROW_COLS, "wide: {line}");
        }
    }

    #[test]
    fn scan_height_matches_pack() {
        let workers = vec![WorkerView::default(); 4];
        let snap = ProgressSnapshot {
            units_done: 100,
            units_total: 0,
            logical_done: 1_000_000,
            logical_total: 0,
            logical_transferred: 0,
            written_done: 0,
            current: "詳細設計書/ブロック/a.txt".into(),
            phase: "scan",
            ratio_prior: 1.0,
        };
        let lines = render_dashboard(&snap, 5.0, 0, 0, &workers);
        assert_eq!(lines.len(), 6);
        assert!(lines[0].contains("scan"));
        assert!(display_width(&lines[0]) <= ROW_COLS);
    }
}
