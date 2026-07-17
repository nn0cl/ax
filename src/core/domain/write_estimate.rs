//! Pure write-progress and ETA estimates (no unit-count extrapolation).
//!
//! Used by delivery progress UI; kept free of I/O and formatting.
//!
//! Before any archive is transferred, the write total uses a **format prior**
//! (`ArchiveFormat::prior_archive_ratio`). After the first transfer completes,
//! the empirical ratio `written / logical_transferred` takes over.

use super::format::ArchiveFormat;

/// Empirical compression ratio from finished transfers only.
///
/// `r = written_done / logical_transferred` after at least one transfer;
/// otherwise falls back to `prior_ratio` (format algorithm prior).
pub fn compression_ratio(
    written_done: u64,
    logical_transferred: u64,
    prior_ratio: f64,
) -> f64 {
    if logical_transferred > 0 && written_done > 0 {
        written_done as f64 / logical_transferred as f64
    } else {
        sanitize_prior(prior_ratio)
    }
}

/// Effective archive/logical ratio for a pack format.
pub fn compression_ratio_for_format(
    written_done: u64,
    logical_transferred: u64,
    format: ArchiveFormat,
) -> f64 {
    compression_ratio(
        written_done,
        logical_transferred,
        format.prior_archive_ratio(),
    )
}

/// Estimated total archive bytes: `logical_total * r` (at least `written_done`).
pub fn estimated_total_write_bytes(
    logical_total: u64,
    written_done: u64,
    logical_transferred: u64,
    prior_ratio: f64,
) -> f64 {
    if logical_total == 0 {
        return 0.0;
    }
    let r = compression_ratio(written_done, logical_transferred, prior_ratio);
    (logical_total as f64 * r).max(written_done as f64)
}

/// Format-only prior archive size (mixed content). Prefer content-aware
/// [`super::estimate_archive_for_sources`] when file paths are available.
pub fn estimate_archive_bytes(logical_bytes: u64, format: ArchiveFormat) -> u64 {
    let r = format.prior_archive_ratio();
    let v = logical_bytes as f64 * r;
    if v.is_finite() && v > 0.0 {
        v.ceil() as u64
    } else {
        logical_bytes
    }
}

/// Write progress 0–100: written / estimated total archive bytes.
pub fn write_progress_pct(
    logical_total: u64,
    written_done: u64,
    logical_transferred: u64,
    prior_ratio: f64,
) -> f64 {
    let est = estimated_total_write_bytes(
        logical_total,
        written_done,
        logical_transferred,
        prior_ratio,
    );
    if est <= 0.0 {
        return 0.0;
    }
    (100.0 * written_done as f64 / est).clamp(0.0, 100.0)
}

fn sanitize_prior(prior_ratio: f64) -> f64 {
    if prior_ratio.is_finite() && prior_ratio > 0.0 {
        prior_ratio.clamp(0.05, 2.0)
    } else {
        1.0
    }
}

/// Input-side remaining seconds: unread logical / input rate.
pub fn eta_input_secs(logical_done: u64, logical_total: u64, in_bps: u64) -> Option<f64> {
    if logical_total == 0 || in_bps == 0 {
        return None;
    }
    if logical_done >= logical_total {
        return Some(0.0);
    }
    Some((logical_total - logical_done) as f64 / in_bps as f64)
}

/// Transfer-side remaining seconds from the same model as write %.
pub fn eta_transfer_secs(
    logical_total: u64,
    written_done: u64,
    logical_transferred: u64,
    out_bps: u64,
    prior_ratio: f64,
) -> Option<f64> {
    if out_bps == 0 {
        return None;
    }
    if logical_total == 0 && written_done == 0 {
        return None;
    }
    let remaining_write = (estimated_total_write_bytes(
        logical_total,
        written_done,
        logical_transferred,
        prior_ratio,
    ) - written_done as f64)
        .max(0.0);
    if remaining_write <= 1.0
        && write_progress_pct(
            logical_total,
            written_done,
            logical_transferred,
            prior_ratio,
        ) >= 99.9
    {
        return Some(0.0);
    }
    if remaining_write <= 0.0 {
        return Some(0.0);
    }
    Some(remaining_write / out_bps as f64)
}

/// Wall-clock ETA breakdown: max(input, transfer).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EtaBreakdown {
    pub wall: Option<f64>,
    pub input: Option<f64>,
    pub xfer: Option<f64>,
}

/// Full-job ETA. `phase` of `"done"` / `"scan"` / `"plan"` short-circuits.
#[allow(clippy::too_many_arguments)]
pub fn eta_wall_clock(
    phase: &str,
    logical_done: u64,
    logical_total: u64,
    written_done: u64,
    logical_transferred: u64,
    in_bps: u64,
    out_bps: u64,
    prior_ratio: f64,
) -> EtaBreakdown {
    if phase == "done" {
        return EtaBreakdown {
            wall: Some(0.0),
            input: Some(0.0),
            xfer: Some(0.0),
        };
    }
    if phase == "scan" || phase == "plan" {
        return EtaBreakdown {
            wall: None,
            input: None,
            xfer: None,
        };
    }

    let input = eta_input_secs(logical_done, logical_total, in_bps);
    let xfer = eta_transfer_secs(
        logical_total,
        written_done,
        logical_transferred,
        out_bps,
        prior_ratio,
    );
    let pct = write_progress_pct(
        logical_total,
        written_done,
        logical_transferred,
        prior_ratio,
    );
    let wall = match (input, xfer) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => {
            if a == 0.0 && pct < 99.9 {
                None
            } else {
                Some(a)
            }
        }
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    EtaBreakdown { wall, input, xfer }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_progress_matches_ratio() {
        // empirical r=0.4, est total=40, written=10 → 25%
        let pct = write_progress_pct(100, 10, 25, 1.0);
        assert!((pct - 25.0).abs() < 1e-9);
    }

    #[test]
    fn prior_used_before_first_transfer() {
        // no transfer yet; tar.zst prior 0.55 → est = 55, written 0 → 0%
        let prior = ArchiveFormat::TarZst.prior_archive_ratio();
        let est = estimated_total_write_bytes(100, 0, 0, prior);
        assert!((est - 100.0 * prior).abs() < 1e-9);
        let pct = write_progress_pct(100, 0, 0, prior);
        assert!((pct - 0.0).abs() < 1e-9);
    }

    #[test]
    fn empirical_overrides_prior_after_transfer() {
        // prior 0.55 would give est 55; empirical 40/100=0.4 → est 40
        let prior = 0.55;
        let est = estimated_total_write_bytes(100, 40, 100, prior);
        assert!((est - 40.0).abs() < 1e-9);
    }

    #[test]
    fn estimate_archive_bytes_by_format() {
        assert_eq!(
            estimate_archive_bytes(1000, ArchiveFormat::Tar),
            (1000.0 * ArchiveFormat::Tar.prior_archive_ratio()).ceil() as u64
        );
        let zst = estimate_archive_bytes(10_000, ArchiveFormat::TarZst);
        let zip = estimate_archive_bytes(10_000, ArchiveFormat::Zip);
        assert!(zst < zip); // zstd prior tighter than zip deflate prior
        assert!(zip < 10_000);
    }

    #[test]
    fn wall_prefers_slower_side() {
        let e = eta_wall_clock("hash", 50, 100, 10, 25, 100, 10, 1.0);
        assert_eq!(e.input, Some(0.5));
        assert_eq!(e.xfer, Some(3.0));
        assert_eq!(e.wall, Some(3.0));
    }

    #[test]
    fn input_done_still_has_xfer_eta() {
        let e = eta_wall_clock("transfer", 1000, 1000, 952, 952, 100, 16, 1.0);
        assert_eq!(e.input, Some(0.0));
        assert!(e.xfer.unwrap() > 2.0);
        assert!(e.wall.unwrap() > 2.0);
    }
}
