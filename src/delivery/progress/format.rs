//! Status line text generation (no I/O).

use super::rate::WorkerView;
use crate::core::domain::{eta_wall_clock, human_bytes, write_progress_pct, EtaBreakdown};
use crate::core::ports::ProgressSnapshot;

pub fn truncate_middle(s: &str, max: usize) -> String {
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
pub fn format_duration(secs: f64) -> String {
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

pub fn format_time_fields(elapsed_secs: f64, eta: EtaBreakdown) -> String {
    let elapsed = format_duration(elapsed_secs);
    match eta.wall {
        Some(w) if w.is_finite() && w >= 0.0 => {
            let mut s = format!("elapsed {elapsed}  ETA {}", format_duration(w));
            if eta.input.is_some() || eta.xfer.is_some() {
                let in_s = eta
                    .input
                    .map(format_duration)
                    .unwrap_or_else(|| "--".into());
                let x_s = eta
                    .xfer
                    .map(format_duration)
                    .unwrap_or_else(|| "--".into());
                s.push_str(&format!(" (in {in_s} / xfer {x_s})"));
            }
            s
        }
        _ => format!("elapsed {elapsed}  ETA --"),
    }
}

pub fn format_block(
    snap: &ProgressSnapshot,
    elapsed: f64,
    in_bps: u64,
    out_bps: u64,
    show_workers: bool,
    workers: &[WorkerView],
) -> Vec<String> {
    let prior = if snap.ratio_prior.is_finite() && snap.ratio_prior > 0.0 {
        snap.ratio_prior
    } else {
        1.0
    };
    let time = format_time_fields(
        elapsed,
        eta_wall_clock(
            snap.phase,
            snap.logical_done,
            snap.logical_total,
            snap.written_done,
            snap.logical_transferred,
            in_bps,
            out_bps,
            prior,
        ),
    );

    let in_pct = if snap.logical_total > 0 {
        (100.0 * snap.logical_done as f64 / snap.logical_total as f64).min(100.0)
    } else {
        0.0
    };
    let xfer_pct = write_progress_pct(
        snap.logical_total,
        snap.written_done,
        snap.logical_transferred,
        prior,
    );
    let plan_pct = if snap.units_total > 0 {
        100.0 * snap.units_done as f64 / snap.units_total as f64
    } else {
        0.0
    };
    let cur_w = if show_workers { 28 } else { 32 };
    let current = truncate_middle(&snap.current, cur_w);

    if snap.phase == "scan" || snap.phase == "plan" {
        return format_scan_plan(snap, elapsed, plan_pct, &current, &time);
    }

    let mut lines = Vec::new();
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

    if show_workers {
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

    let second = if matches!(snap.phase, "verify" | "extract" | "list") {
        format!(
            "checked   {written} hashed  {xfer_pct:5.1}%  {speed}/s  {time}  {current}",
            written = human_bytes(snap.written_done),
            xfer_pct = xfer_pct,
            speed = human_bytes(in_bps),
            time = time,
            current = current,
        )
    } else {
        format!(
            "transfer  {written} written  {xfer_pct:5.1}%  {speed}/s  {time}  {current}",
            written = human_bytes(snap.written_done),
            xfer_pct = xfer_pct,
            speed = human_bytes(out_bps),
            time = time,
            current = current,
        )
    };
    lines.push(second);
    lines
}

fn format_scan_plan(
    snap: &ProgressSnapshot,
    scan_elapsed: f64,
    plan_pct: f64,
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
                pct = plan_pct,
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
    fn duration_formats() {
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(45.0), "45s");
        assert_eq!(format_duration(83.0), "1m23s");
        assert_eq!(format_duration(3725.0), "1h02m");
    }

    #[test]
    fn time_fields_include_elapsed_and_eta() {
        let s = format_time_fields(
            83.0,
            EtaBreakdown {
                wall: Some(125.0),
                input: Some(60.0),
                xfer: Some(125.0),
            },
        );
        assert!(s.contains("elapsed 1m23s"));
        assert!(s.contains("ETA 2m05s"));
        assert!(s.contains("in 1m00s"));
        assert!(s.contains("xfer 2m05s"));
    }
}
