//! Pure string rendering for the progress dashboard (no I/O, no locks).
//!
//! Layout (fixed height = `workers.len() + 2`):
//! ```text
//! total  …
//! cmp0   …
//! cmp1   …
//! …
//! trn    …
//! ```
//!
//! # Geometry (ADR 0024)
//!
//! - Each row ≤ 80 display columns (Linux console baseline).
//! - **Metrics first, fixed-width right-aligned numerics** — no left/right jitter.
//! - Short row tags: `total` / `cmp0`… / `trn` (fixed label column + gutters).
//! - **`total` has no path** so there is room for full `elapsed` / `ETA` labels;
//!   paths live on `cmp*` / `trn` only.
//! - Path last on those rows, truncated only; **never** middle-truncate a whole row
//!   (that produced `…TA` from `ETA` / `…fer` from `transfer`).

use super::rate::WorkerView;
use super::width::{
    display_file_label, display_width, is_wide_layout, path_cols_for_row, take_cols_left, ROW_COLS,
};
use crate::core::domain::{eta_wall_clock, write_progress_pct, EtaBreakdown};
use crate::core::ports::ProgressSnapshot;

const MAX_ETA_SECS: f64 = 48.0 * 3600.0;

// ── Fixed field widths (ASCII; right-aligned where numeric) ─────────────────
/// Unit / file counts (`99999`).
const W_COUNT: usize = 5;
/// Compact size/rate body (`999.9M`, `1023B`).
const W_BYTES: usize = 7;
/// Rate field including `/s` (`999.9M/s`).
const W_RATE: usize = W_BYTES + 2;
/// Percent body without `%` (`100.0`).
const W_PCT: usize = 5;
/// One duration token (`59m59s`, `23h59m`).
const W_DURATION: usize = 6;
/// Fixed clock: `elapsed ` + dur + `  ETA ` + dur (e.g. `elapsed  4m17s  ETA 20h50m`).
/// Fits on `total` because that row has **no path** (paths are on worker/trn).
const W_CLOCK: usize = 8 + W_DURATION + 6 + W_DURATION; // 26
/// Row tag column: `total`, `cmp0`…`cmp9`, `trn` (left-aligned, space-padded).
const W_LABEL: usize = 5;
/// Gutter between major fields (readable separation without eating the 80-col budget).
const GUT: &str = "  ";
/// Short phase inside `[…]`.
const W_PHASE: usize = 5;

/// Column budget for one paint (usually 80, or narrower best-effort).
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub row_cols: usize,
    /// Soft path hint (residual after metrics is the real budget on `cmp`/`trn`).
    #[allow(dead_code)]
    pub path_cols: usize,
}

impl Layout {
    /// Design baseline: 80-column Linux console (ADR 0024).
    pub fn console_80() -> Self {
        Self::from_row_cols(ROW_COLS)
    }

    pub fn from_row_cols(row_cols: usize) -> Self {
        let row_cols = row_cols.max(1);
        Self {
            row_cols,
            path_cols: path_cols_for_row(row_cols),
        }
    }

    /// From optional terminal width probe.
    pub fn from_terminal_cols(terminal_cols: Option<usize>) -> Self {
        Self::from_row_cols(super::width::effective_row_cols(terminal_cols))
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::console_80()
    }
}

/// Build the fixed-height dashboard lines from a snapshot + worker rows.
pub fn render_dashboard(
    snap: &ProgressSnapshot,
    elapsed: f64,
    in_bps: u64,
    out_bps: u64,
    workers: &[WorkerView],
    layout: Layout,
) -> Vec<String> {
    let clock = format_clock(elapsed, snap, in_bps, out_bps);
    let path = snap.current.as_str();

    if snap.phase == "scan" || snap.phase == "plan" {
        return render_scan_or_plan(snap, elapsed, &clock, workers.len(), layout);
    }

    let in_pct = pct(snap.logical_done, snap.logical_total);
    let xfer_pct = write_progress_pct(
        snap.logical_total,
        snap.written_done,
        snap.logical_transferred,
        prior(snap),
    );

    let mut lines = Vec::with_capacity(workers.len() + 2);

    // total — metrics only (no path): room for full `elapsed` / `ETA` labels.
    // Paths are shown on cmp* / trn rows instead.
    let total_line = format!(
        "{lab}{g}{done}/{total}{g}{pct}%{g}{rate}{g}{clock}{g}[{phase}]",
        lab = fmt_label("total"),
        g = GUT,
        done = fmt_count(snap.units_done as u64),
        total = fmt_count(snap.units_total as u64),
        pct = fmt_pct(in_pct),
        rate = fmt_rate(in_bps),
        clock = clock,
        phase = fmt_phase(snap.phase),
    );
    lines.push(fit_metrics_row(total_line, layout.row_cols));

    for (i, w) in workers.iter().enumerate() {
        lines.push(render_worker_line(i, w, layout));
    }

    let xfer_path = if snap.phase == "transfer" { path } else { "-" };
    // trn — path only while actively transferring; else `-`
    let xfer_prefix = format!(
        "{lab}{g}{written}{g}{xp}%{g}{os}{g}{clock}{g}",
        lab = fmt_label("trn"),
        g = GUT,
        written = fmt_bytes(snap.written_done),
        xp = fmt_pct(xfer_pct),
        os = fmt_rate(out_bps),
        clock = clock,
    );
    lines.push(finish_row(xfer_prefix, xfer_path, layout));

    debug_assert_eq!(lines.len(), workers.len() + 2);
    lines
}

fn render_worker_line(i: usize, w: &WorkerView, layout: Layout) -> String {
    let lab = fmt_label(&worker_label(i));
    if !w.active && (w.phase == "idle" || w.phase.is_empty()) {
        let prefix = format!(
            "{lab}{g}{rate}{g}[{phase}]{g}",
            lab = lab,
            g = GUT,
            rate = fmt_rate_idle(),
            phase = fmt_phase("idle"),
        );
        return finish_row(prefix, "-", layout);
    }
    let phase = if w.phase.is_empty() { "…" } else { w.phase };
    let prefix = format!(
        "{lab}{g}{rate}{g}[{phase}]{g}",
        lab = lab,
        g = GUT,
        rate = fmt_rate(w.bps),
        phase = fmt_phase(phase),
    );
    let path = worker_path_display(w, layout);
    finish_row(prefix, &path, layout)
}

/// Path column for a worker row — file name + stable path tag (see `display_file_label`).
///
/// Full relative paths are passed through so tags differ for same basenames.
/// - **80-col**: one label (source on hash, archive on compr).
/// - **≥120-col wide**: on compress/verify, `archive | source` labels.
fn worker_path_display(w: &WorkerView, layout: Layout) -> String {
    let wide = is_wide_layout(layout.row_cols);
    let dual = wide
        && matches!(w.phase, "compress" | "compr" | "verify-stage" | "vrfy")
        && !w.source.is_empty()
        && w.source != w.current;
    if !dual {
        return w.current.clone();
    }
    format!("{}{}{}", w.current, DUAL_SEP, w.source)
}

fn worker_label(i: usize) -> String {
    // cmp0..cmp9 fit W_LABEL=5; rare high -j uses c10+
    if i < 10 {
        format!("cmp{i}")
    } else {
        format!("c{i}")
    }
}

fn render_scan_or_plan(
    snap: &ProgressSnapshot,
    elapsed: f64,
    clock: &str,
    worker_slots: usize,
    layout: Layout,
) -> Vec<String> {
    let mut lines = Vec::with_capacity(worker_slots + 2);
    // scan/plan: no path on total — sticky `scanning …` already has the root;
    // keep room for full elapsed/ETA labels.
    if snap.phase == "scan" {
        let files_per_sec = (snap.units_done as f64 / elapsed.max(1e-6)) as u64;
        let bps = (snap.logical_done as f64 / elapsed.max(1e-6)) as u64;
        let total_line = format!(
            "{lab}{g}scan{g}{files}{g}{bytes}{g}{fps}/s{g}{bps}{g}{clock}",
            lab = fmt_label("total"),
            g = GUT,
            files = fmt_count(snap.units_done as u64),
            bytes = fmt_bytes(snap.logical_done),
            fps = fmt_count(files_per_sec),
            bps = fmt_rate(bps),
            clock = clock,
        );
        lines.push(fit_metrics_row(total_line, layout.row_cols));
    } else {
        let plan_pct = pct(snap.units_done as u64, snap.units_total as u64);
        let total_line = format!(
            "{lab}{g}plan{g}{done}/{total}{g}{pct}%{g}{clock}",
            lab = fmt_label("total"),
            g = GUT,
            done = fmt_count(snap.units_done as u64),
            total = fmt_count(snap.units_total as u64),
            pct = fmt_pct(plan_pct),
            clock = clock,
        );
        lines.push(fit_metrics_row(total_line, layout.row_cols));
    }
    for i in 0..worker_slots {
        let lab = fmt_label(&worker_label(i));
        let prefix = format!(
            "{lab}{g}{rate}{g}[{phase}]{g}",
            lab = lab,
            g = GUT,
            rate = fmt_rate_idle(),
            phase = fmt_phase("idle"),
        );
        lines.push(finish_row(prefix, "-", layout));
    }
    let xfer_prefix = format!(
        "{lab}{g}{written}{g}{xp}%{g}{os}{g}{clock}{g}",
        lab = fmt_label("trn"),
        g = GUT,
        written = ralign("-", W_BYTES),
        xp = fmt_pct(0.0),
        os = fmt_rate_idle(),
        clock = clock,
    );
    lines.push(finish_row(xfer_prefix, "waiting", layout));
    lines
}

/// Metrics-only row (e.g. `total`): never append a path; left-clip if narrow.
fn fit_metrics_row(line: String, row_cols: usize) -> String {
    let row_cols = row_cols.max(1);
    if display_width(&line) <= row_cols {
        line
    } else {
        take_cols_left(&line, row_cols)
    }
}

/// Metrics prefix + path label. Labels may use `…`; metrics never do (ADR 0024).
///
/// Path column is **file-name level only** (not full directories). Dual form is
/// `archive | source` names on wide terminals.
fn finish_row(prefix: String, path: &str, layout: Layout) -> String {
    let row_cols = layout.row_cols.max(1);
    let pw = display_width(&prefix);
    if pw >= row_cols {
        // Narrow terminal: keep left (numbers / labels), do not middle-mangle.
        return take_cols_left(&prefix, row_cols);
    }
    let budget = row_cols - pw;
    if path.is_empty() || budget == 0 {
        return prefix;
    }
    let p = fit_path_column(path, budget);
    format!("{prefix}{p}")
}

/// Dual-path separator on wide `cmp` rows (ASCII; no arrows).
const DUAL_SEP: &str = " | ";

/// Single file name or `left | right` dual names.
fn fit_path_column(path: &str, budget: usize) -> String {
    if let Some((left, right)) = path.split_once(DUAL_SEP) {
        let sep_w = display_width(DUAL_SEP);
        if budget <= sep_w + 4 {
            return display_file_label(right, budget);
        }
        // Short archive name, remaining budget for source name.
        let left_budget = ((budget - sep_w) / 3).clamp(8, 24);
        let right_budget = budget.saturating_sub(sep_w + left_budget);
        return format!(
            "{}{}{}",
            display_file_label(left, left_budget),
            DUAL_SEP,
            display_file_label(right, right_budget)
        );
    }
    display_file_label(path, budget)
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

// ── Fixed-width formatters (ADR 0024 §2.5) ──────────────────────────────────

fn ralign(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{:>pad$}", s, pad = width)
    }
}

fn fmt_label(s: &str) -> String {
    let w = display_width(s);
    if w >= W_LABEL {
        s.to_string()
    } else {
        format!("{:<pad$}", s, pad = W_LABEL)
    }
}

fn fmt_count(n: u64) -> String {
    ralign(&n.to_string(), W_COUNT)
}

fn fmt_pct(p: f64) -> String {
    let body = if p.is_finite() {
        format!("{p:5.1}")
    } else {
        "  --.-".into()
    };
    ralign(&body, W_PCT)
}

fn short_phase(phase: &str) -> &str {
    match phase {
        "compress" => "compr",
        "verify-stage" => "vrfy",
        "transfer" => "xfer",
        "start" => "start",
        other => other,
    }
}

fn fmt_phase(phase: &str) -> String {
    let p = short_phase(phase);
    let w = display_width(p);
    if w >= W_PHASE {
        take_cols_left(p, W_PHASE)
    } else {
        format!("{:<pad$}", p, pad = W_PHASE)
    }
}

fn fmt_bytes(n: u64) -> String {
    // human_compact is always exactly W_BYTES; never rely on post-hoc pad
    // (variable "54.5M" vs "144.3M" vs "1023.9M" was still shifting digits).
    let s = human_compact(n);
    debug_assert_eq!(
        display_width(&s),
        W_BYTES,
        "human_compact width: {s:?}"
    );
    s
}

fn fmt_rate(bps: u64) -> String {
    // Always `######U/s` = W_BYTES + "/s" = W_RATE (e.g. "  54.5M/s", "1023.9M/s").
    let s = format!("{}/s", human_compact(bps));
    debug_assert_eq!(display_width(&s), W_RATE, "fmt_rate width: {s:?}");
    s
}

/// Idle placeholder with the same width as [`fmt_rate`].
fn fmt_rate_idle() -> String {
    ralign("-/s", W_RATE)
}

/// Fixed-width compact size: **always** [`W_BYTES`] (7) columns.
///
/// | value   | render   |
/// |---------|----------|
/// | 500 B   | `   500B` |
/// | 54.5 MiB| `  54.5M` |
/// | 144 MiB | ` 144.3M` |
/// | 1023 MiB| `1023.9M` |
/// | 5 GiB   | `   5.0G` |
///
/// Mantissa uses `{v:6.1}` so 2-/3-/4-digit magnitudes share the same decimal
/// column; unit is a single letter (K/M/G/T…).
fn human_compact(n: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P", "E"];
    if n < 1024 {
        // "     0B" … "  1023B" — 6 digits/spaces + unit = 7.
        return format!("{n:>6}B");
    }
    let mut v = n as f64;
    let mut i = 0usize;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    // Minimum width 6 + 1 decimal → "   1.0" … "1023.9", then 1-letter unit.
    format!("{v:6.1}{}", UNITS[i])
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

fn fmt_duration_field(secs: f64) -> String {
    ralign(&format_duration(secs), W_DURATION)
}

/// Fixed clock with readable labels (room reserved by dropping path on `total`):
/// `elapsed  4m17s  ETA 20h50m` — always [`W_CLOCK`] columns.
fn format_time_fields(elapsed_secs: f64, eta: EtaBreakdown) -> String {
    let elapsed = fmt_duration_field(elapsed_secs.min(MAX_ETA_SECS * 10.0));
    let eta_s = match eta.wall {
        Some(w) if w.is_finite() && (0.0..MAX_ETA_SECS).contains(&w) => fmt_duration_field(w),
        _ => ralign("--", W_DURATION),
    };
    let out = format!("elapsed {elapsed}  ETA {eta_s}");
    debug_assert_eq!(
        display_width(&out),
        W_CLOCK,
        "clock width: {out:?} ({})",
        display_width(&out)
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snap(phase: &'static str) -> ProgressSnapshot {
        ProgressSnapshot {
            units_done: 1,
            units_total: 31,
            logical_done: 1000,
            logical_total: 10_000,
            logical_transferred: 0,
            written_done: 0,
            current: "Documents/part-0001.zip".into(),
            phase,
            ratio_prior: 0.7,
        }
    }

    #[test]
    fn duration_formats() {
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(45.0), "45s");
        assert_eq!(format_duration(83.0), "1m23s");
        assert_eq!(format_duration(3725.0), "1h02m");
    }

    #[test]
    fn numeric_fields_fixed_width() {
        assert_eq!(display_width(&fmt_count(1)), W_COUNT);
        assert_eq!(display_width(&fmt_count(99_999)), W_COUNT);
        assert_eq!(display_width(&fmt_pct(0.0)), W_PCT);
        assert_eq!(display_width(&fmt_pct(100.0)), W_PCT);
        assert_eq!(display_width(&fmt_rate(0)), W_RATE);
        assert_eq!(display_width(&fmt_rate(144_330_000)), W_RATE);
        assert_eq!(display_width(&fmt_bytes(153 * 1024 * 1024)), W_BYTES);

        for secs in [0.0, 9.0, 59.0, 83.0, 3599.0, 3725.0, 48.0 * 3600.0 + 1.0] {
            assert_eq!(display_width(&fmt_duration_field(secs)), W_DURATION);
        }

        let t0 = format_time_fields(
            1.0,
            EtaBreakdown {
                wall: None,
                input: None,
                xfer: None,
            },
        );
        let t1 = format_time_fields(
            3725.0,
            EtaBreakdown {
                wall: Some(83.0),
                input: None,
                xfer: None,
            },
        );
        assert_eq!(display_width(&t0), W_CLOCK);
        assert_eq!(display_width(&t1), W_CLOCK);
        assert!(t0.starts_with("elapsed "));
        assert!(t0.contains("ETA"));
        assert!(!t0.contains('…'));
        assert!(!t0.contains('>'));
    }

    #[test]
    fn metrics_prefix_fits_without_path_truncation() {
        // Metrics alone must fit in 80 so path truncation never eats numbers.
        let workers = vec![WorkerView::default(); 4];
        let mut snap = sample_snap("hash");
        snap.units_done = 12_345;
        snap.units_total = 67_890;
        snap.written_done = 5 * 1024 * 1024 * 1024;
        let lines = render_dashboard(
            &snap,
            186.0,
            95_940_000,
            1_040_000,
            &workers,
            Layout::console_80(),
        );
        for line in &lines {
            assert!(
                display_width(line) <= ROW_COLS,
                "over 80: {:?} w={}",
                line,
                display_width(line)
            );
        }
        let total = &lines[0];
        // total has no path — must never middle-truncate metrics (`…TA` bug).
        assert!(
            !total.contains('…'),
            "total must not ellipsize (no path): {total}"
        );
        assert!(total.contains("elapsed "), "elapsed missing: {total}");
        assert!(total.contains("ETA"), "ETA missing: {total}");
        assert!(total.contains('%'), "pct missing: {total}");
        assert!(total.contains("/s"), "rate missing: {total}");
        // No file path on total (paths live on cmp*/trn).
        assert!(
            !total.contains(".zip"),
            "total should not show path: {total}"
        );
    }

    #[test]
    fn counts_do_not_shift_total_line() {
        let workers = vec![WorkerView::default(); 2];
        let mut small = sample_snap("hash");
        small.units_done = 1;
        small.units_total = 9;
        let mut large = sample_snap("hash");
        large.units_done = 12_345;
        large.units_total = 67_890;

        let a = render_dashboard(&small, 10.0, 100, 0, &workers, Layout::console_80());
        let b = render_dashboard(&large, 10.0, 100, 0, &workers, Layout::console_80());

        // Percent field starts at the same column.
        let pos_a = a[0].find('%').expect("% in small");
        let pos_b = b[0].find('%').expect("% in large");
        assert_eq!(pos_a, pos_b, "\nA: {}\nB: {}", a[0], b[0]);
    }

    #[test]
    fn rates_do_not_shift_worker_line() {
        let layout = Layout::console_80();
        let mut slow = WorkerView::default();
        slow.active = true;
        slow.phase = "hash";
        slow.current = "part-0001.zip".into();
        slow.bps = 1_000;
        let mut fast = WorkerView::default();
        fast.active = true;
        fast.phase = "compress";
        fast.current = "part-0002.zip".into();
        fast.bps = 466_240_000;

        let a = render_worker_line(0, &slow, layout);
        let b = render_worker_line(0, &fast, layout);
        let pos_a = a.find('[').expect("phase slow");
        let pos_b = b.find('[').expect("phase fast");
        assert_eq!(pos_a, pos_b, "\nA: {a}\nB: {b}");
    }

    #[test]
    fn idle_and_active_worker_share_column_skeleton() {
        let layout = Layout::console_80();
        let idle = render_worker_line(0, &WorkerView::default(), layout);
        let mut active_view = WorkerView::default();
        active_view.active = true;
        active_view.phase = "hash";
        active_view.current = "part-0001.zip".into();
        active_view.bps = 1_500_000;
        let active = render_worker_line(0, &active_view, layout);
        let pos_i = idle.find('[').expect("idle");
        let pos_a = active.find('[').expect("active");
        assert_eq!(pos_i, pos_a, "\nidle:   {idle}\nactive: {active}");
    }

    #[test]
    fn dashboard_height_fixed_at_80_layout() {
        let workers = vec![WorkerView::default(); 4];
        let lines = render_dashboard(
            &sample_snap("hash"),
            10.0,
            1_000_000,
            0,
            &workers,
            Layout::console_80(),
        );
        assert_eq!(lines.len(), 6);
        assert!(lines[0].starts_with("total"));
        assert!(lines[1].starts_with("cmp0"));
        assert!(lines[5].starts_with("trn"));
        // Label column is fixed width; metrics start at the same index on all rows.
        let metric_start = |s: &str| s.chars().take(W_LABEL).count();
        assert_eq!(metric_start(&lines[0]), W_LABEL);
        assert_eq!(metric_start(&lines[1]), W_LABEL);
        assert_eq!(metric_start(&lines[5]), W_LABEL);
        for line in &lines {
            assert!(
                display_width(line) <= ROW_COLS,
                "line wider than 80: {line:?} w={}",
                display_width(line)
            );
        }
    }

    #[test]
    fn cmp_path_shows_compression_target_filename() {
        let layout = Layout::console_80();
        let mut w = WorkerView::default();
        w.active = true;
        w.phase = "hash";
        w.bps = 200_000_000;
        // Source file under deep tree (hash phase reports rel_path).
        w.current = "Home/Documents/backup/deep/folder/report-2025-final.docx".into();
        w.source = w.current.clone();
        let line = render_worker_line(0, &w, layout);
        assert!(display_width(&line) <= ROW_COLS, "{line}");
        assert!(
            line.contains("report-2025-final.docx·"),
            "filename+tag must be visible on cmp: {line}"
        );
        assert!(
            !line.contains("Documents/"),
            "dirs should not appear: {line}"
        );
        // compress phase (80-col): archive file name + tag only
        w.phase = "compress";
        w.current = "Home/part-0004.zip".into();
        w.source = "Home/Documents/backup/deep/folder/report-2025-final.docx".into();
        let line2 = render_worker_line(1, &w, layout);
        assert!(
            line2.contains("part-0004.zip·"),
            "archive name+tag must be visible: {line2}"
        );
        assert!(
            !line2.contains(DUAL_SEP),
            "80-col should not dual-path: {line2}"
        );
        assert!(!line2.contains("Home/"), "dirs should not appear: {line2}");
    }

    #[test]
    fn wide_cmp_shows_archive_and_source_names() {
        use super::super::width::WIDE_CONSOLE_COLS;
        let layout = Layout::from_row_cols(WIDE_CONSOLE_COLS);
        assert!(is_wide_layout(layout.row_cols));
        let mut w = WorkerView::default();
        w.active = true;
        w.phase = "compress";
        w.bps = 100_000_000;
        w.current = "Documents/2025_NBOM/VirtualMachine_BOM/part-0004.zip".into();
        w.source = "Home/Documents/photos/vacation/IMG_0001.JPG".into();
        let line = render_worker_line(0, &w, layout);
        assert!(
            display_width(&line) <= WIDE_CONSOLE_COLS,
            "w={} {line}",
            display_width(&line)
        );
        assert!(line.contains(DUAL_SEP), "dual path missing: {line}");
        assert!(line.contains("part-0004.zip·"), "archive: {line}");
        assert!(
            line.contains("IMG_0001.JPG·"),
            "source file name+tag must show on wide: {line}"
        );
        assert!(
            !line.contains("VirtualMachine_BOM"),
            "directory noise: {line}"
        );
    }

    #[test]
    fn same_basename_different_tags_on_cmp() {
        let layout = Layout::console_80();
        let mut a = WorkerView::default();
        a.active = true;
        a.phase = "hash";
        a.current = "docs/a/readme.txt".into();
        a.source = a.current.clone();
        let mut b = WorkerView::default();
        b.active = true;
        b.phase = "hash";
        b.current = "docs/b/readme.txt".into();
        b.source = b.current.clone();
        let la = render_worker_line(0, &a, layout);
        let lb = render_worker_line(1, &b, layout);
        assert!(la.contains("readme.txt·"), "{la}");
        assert!(lb.contains("readme.txt·"), "{lb}");
        // Extract tag suffix after last ·
        let tag = |s: &str| s.rsplit('·').next().unwrap_or("").chars().take(4).collect::<String>();
        assert_ne!(tag(&la), tag(&lb), "\n{la}\n{lb}");
    }

    #[test]
    fn short_labels_and_gutters() {
        assert_eq!(fmt_label("total"), "total");
        assert_eq!(fmt_label("cmp0"), "cmp0 ");
        assert_eq!(fmt_label("cmp3"), "cmp3 ");
        assert_eq!(fmt_label("trn"), "trn  ");
        assert_eq!(display_width(&fmt_label("total")), W_LABEL);
        assert_eq!(display_width(&fmt_label("cmp0")), W_LABEL);
        assert_eq!(display_width(&fmt_label("trn")), W_LABEL);

        let workers = vec![WorkerView::default(); 4];
        let lines = render_dashboard(
            &sample_snap("hash"),
            10.0,
            1_000_000,
            0,
            &workers,
            Layout::console_80(),
        );
        // Double-space gutters between major fields on total.
        assert!(lines[0].contains("  "), "expected gutters: {}", lines[0]);
        assert!(lines[0].contains("elapsed "));
        // Worker: rate then gutter then phase.
        assert!(lines[1].contains("/s  [") || lines[1].contains("-/s  ["), "{}", lines[1]);
        assert!(lines[5].starts_with("trn  ") || lines[5].starts_with("trn"), "{}", lines[5]);
    }

    #[test]
    fn scan_cjk_fits_80() {
        let workers = vec![WorkerView::default(); 4];
        let mut snap = sample_snap("scan");
        snap.current = "詳細設計書/ブロック情報管理/深い/パス/file.txt".into();
        snap.units_done = 100;
        snap.logical_done = 1_000_000;
        let lines = render_dashboard(&snap, 5.0, 0, 0, &workers, Layout::console_80());
        assert_eq!(lines.len(), 6);
        assert!(display_width(&lines[0]) <= 80);
        // total has no path — CJK must not appear / ellipsize there.
        assert!(!lines[0].contains('…'), "total={}", lines[0]);
        assert!(lines[0].contains("elapsed "));
    }

    #[test]
    fn narrow_layout_still_bounded() {
        let workers = vec![WorkerView::default(); 2];
        let layout = Layout::from_row_cols(50);
        let lines = render_dashboard(&sample_snap("hash"), 1.0, 100, 0, &workers, layout);
        assert_eq!(lines.len(), 4);
        for line in &lines {
            assert!(display_width(line) <= 50);
        }
    }

    #[test]
    fn human_compact_steps_units() {
        assert_eq!(human_compact(500), "   500B");
        assert_eq!(human_compact(1024), "   1.0K");
        assert_eq!(human_compact(1024 * 1024), "   1.0M");
        assert_eq!(human_compact(16 * 1024u64.pow(4)), "  16.0T");
        assert_eq!(display_width(&human_compact(500)), W_BYTES);
        assert_eq!(display_width(&human_compact(1024 * 1024 - 1)), W_BYTES);
    }

    #[test]
    fn rate_2_3_4_digit_mib_share_columns() {
        // Operator report: 2-digit / 3-digit / 4-digit MiB-class rates still shifted.
        let samples = [
            54_480_000u64,   // ~52.0M
            144_330_000,     // ~137.6M
            466_240_000,     // ~444.6M
            1_023 * 1024 * 1024, // ~1023.0M
            1_040_000,       // ~1015.6K (4 digits before decimal in K)
            5 * 1024 * 1024 * 1024, // ~5.0G
        ];
        let mut bodies = Vec::new();
        let mut rates = Vec::new();
        for n in samples {
            let b = human_compact(n);
            let r = fmt_rate(n);
            assert_eq!(display_width(&b), W_BYTES, "bytes {n} → {b:?}");
            assert_eq!(display_width(&r), W_RATE, "rate {n} → {r:?}");
            // Decimal point column (within the 6-char mantissa) must be fixed for
            // scaled units (not pure-byte branch).
            if n >= 1024 {
                let dot = b.find('.').expect("dot");
                assert_eq!(dot, 4, "decimal column for {b:?}");
            }
            bodies.push(b);
            rates.push(r);
        }
        // `/s` always starts at the same index (after fixed 7-col body).
        for r in &rates {
            assert_eq!(&r[W_BYTES..], "/s", "rate={r:?}");
        }
        // Worker lines: '[' after rate must not move across 2/3/4-digit M rates.
        let layout = Layout::console_80();
        let mut positions = Vec::new();
        for &bps in &[54_480_000u64, 144_330_000, 1_023 * 1024 * 1024] {
            let mut w = WorkerView::default();
            w.active = true;
            w.phase = "compress";
            w.current = "part.zip".into();
            w.bps = bps;
            let line = render_worker_line(0, &w, layout);
            positions.push(line.find('[').expect("phase bracket"));
        }
        assert_eq!(positions[0], positions[1]);
        assert_eq!(positions[1], positions[2]);
        eprintln!("aligned rates: {rates:?}");
    }

    #[test]
    fn example_pack_line_stable_snapshot() {
        // Mirrors the operator screenshot magnitudes without row wrap/mangle.
        let workers = vec![WorkerView::default(); 4];
        let mut snap = sample_snap("hash");
        snap.units_done = 1;
        snap.units_total = 31;
        snap.logical_done = 2_500;
        snap.logical_total = 10_000;
        snap.written_done = 153 * 1024 * 1024;
        snap.current = "Documents/part-0003.zip".into();
        let lines = render_dashboard(
            &snap,
            186.0, // ~3m06s
            95_940_000,
            1_040_000,
            &workers,
            Layout::console_80(),
        );
        let total = &lines[0];
        assert!(!total.contains("…TA"), "mangled ETA: {total}");
        assert!(total.contains("elapsed "), "expected elapsed label: {total}");
        assert!(total.contains("ETA"), "expected ETA label: {total}");
        assert!(!total.contains(".zip"), "total must drop path: {total}");
        assert!(display_width(total) <= 80, "{total}");
        // transfer still carries clock labels too.
        assert!(lines[5].contains("elapsed "));
        eprintln!("sample total: [{total}] w={}", display_width(total));
        eprintln!("sample xfer:  [{}] w={}", lines[5], display_width(&lines[5]));
    }

    #[test]
    fn total_metrics_fit_with_elapsed_eta_no_path() {
        let workers = vec![WorkerView::default(); 4];
        let lines = render_dashboard(
            &sample_snap("hash"),
            257.0,
            6_100_000_000,
            800_000,
            &workers,
            Layout::console_80(),
        );
        let total = &lines[0];
        assert!(display_width(total) <= ROW_COLS, "w={} {total}", display_width(total));
        assert!(total.contains("elapsed "));
        assert!(total.contains("ETA"));
        // Clock block is fixed width regardless of magnitude.
        let a = format_time_fields(
            1.0,
            EtaBreakdown {
                wall: Some(10.0),
                input: None,
                xfer: None,
            },
        );
        let b = format_time_fields(
            4.0 * 3600.0 + 17.0 * 60.0,
            EtaBreakdown {
                wall: Some(20.0 * 3600.0 + 50.0 * 60.0),
                input: None,
                xfer: None,
            },
        );
        assert_eq!(display_width(&a), display_width(&b));
        assert_eq!(display_width(&a), W_CLOCK);
    }
}
