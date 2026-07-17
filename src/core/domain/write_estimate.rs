//! Pure write-progress and ETA estimates (no unit-count extrapolation).
//!
//! Used by delivery progress UI; kept free of I/O and formatting.

/// Compression ratio from finished transfers only.
///
/// `r = written_done / logical_transferred` after at least one transfer;
/// otherwise 1.0 (pessimistic: archive ≈ logical).
pub fn compression_ratio(written_done: u64, logical_transferred: u64) -> f64 {
    if logical_transferred > 0 && written_done > 0 {
        written_done as f64 / logical_transferred as f64
    } else {
        1.0
    }
}

/// Estimated total archive bytes: `logical_total * r` (at least `written_done`).
pub fn estimated_total_write_bytes(
    logical_total: u64,
    written_done: u64,
    logical_transferred: u64,
) -> f64 {
    if logical_total == 0 {
        return 0.0;
    }
    let r = compression_ratio(written_done, logical_transferred);
    (logical_total as f64 * r).max(written_done as f64)
}

/// Write progress 0–100: written / estimated total archive bytes.
pub fn write_progress_pct(
    logical_total: u64,
    written_done: u64,
    logical_transferred: u64,
) -> f64 {
    let est = estimated_total_write_bytes(logical_total, written_done, logical_transferred);
    if est <= 0.0 {
        return 0.0;
    }
    (100.0 * written_done as f64 / est).clamp(0.0, 100.0)
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
    ) - written_done as f64)
        .max(0.0);
    if remaining_write <= 1.0 {
        if write_progress_pct(logical_total, written_done, logical_transferred) >= 99.9 {
            return Some(0.0);
        }
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
pub fn eta_wall_clock(
    phase: &str,
    logical_done: u64,
    logical_total: u64,
    written_done: u64,
    logical_transferred: u64,
    in_bps: u64,
    out_bps: u64,
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
    );
    let pct = write_progress_pct(logical_total, written_done, logical_transferred);
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
        // r=0.4, est total=40, written=10 → 25%
        let pct = write_progress_pct(100, 10, 25);
        assert!((pct - 25.0).abs() < 1e-9);
    }

    #[test]
    fn wall_prefers_slower_side() {
        let e = eta_wall_clock("hash", 50, 100, 10, 25, 100, 10);
        assert_eq!(e.input, Some(0.5));
        assert_eq!(e.xfer, Some(3.0));
        assert_eq!(e.wall, Some(3.0));
    }

    #[test]
    fn input_done_still_has_xfer_eta() {
        let e = eta_wall_clock("transfer", 1000, 1000, 952, 952, 100, 16);
        assert_eq!(e.input, Some(0.0));
        assert!(e.xfer.unwrap() > 2.0);
        assert!(e.wall.unwrap() > 2.0);
    }
}
