//! Estimate a pack unit size that transfers efficiently for a given link.

use super::media::{mapping_reserve, ConnectionKind};

/// Facts about a destination volume (from probe or manual input).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaFacts {
    /// Free bytes available for writing (0 if unknown).
    pub free_bytes: u64,
    /// Total volume size (0 if unknown).
    pub total_bytes: u64,
    pub connection: ConnectionKind,
    /// Optional measured/advertised link throughput in bytes/sec.
    /// When set, overrides the connection's default practical throughput.
    pub throughput_bytes_per_sec: Option<u64>,
}

/// Inputs for unit-size estimation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstimateRequest {
    pub facts: MediaFacts,
    /// Desired wall-clock seconds to finish transferring one archive unit.
    /// Smaller → smaller units (more parallel feel / less risk of long stalls).
    pub target_transfer_secs: u64,
}

impl Default for EstimateRequest {
    fn default() -> Self {
        Self {
            facts: MediaFacts {
                free_bytes: 0,
                total_bytes: 0,
                connection: ConnectionKind::Local,
                throughput_bytes_per_sec: None,
            },
            target_transfer_secs: DEFAULT_TARGET_SECS,
        }
    }
}

/// Result of transfer-oriented unit sizing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferEstimate {
    /// Recommended pack budget (logical bytes per archive unit).
    pub recommended_unit_bytes: u64,
    /// Hard ceiling from free space minus mapping reserve (if free known).
    pub media_ceiling_bytes: Option<u64>,
    /// Throughput used for the calculation (bytes/sec).
    pub assumed_throughput_bytes_per_sec: u64,
    pub target_transfer_secs: u64,
    pub connection: ConnectionKind,
}

pub const DEFAULT_TARGET_SECS: u64 = 45;
pub const MIN_UNIT_BYTES: u64 = 4 * 1024 * 1024; // 4 MiB
pub const MAX_UNIT_BYTES: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB

/// Practical sustained *write* throughput defaults (not wire peak).
///
/// These are conservative real-world USB flash / network write rates used when
/// the host does not report a measured speed.
pub fn practical_throughput_bytes_per_sec(connection: ConnectionKind) -> u64 {
    const MIB: u64 = 1024 * 1024;
    match connection {
        // Local SSD/NVMe: large units are fine; not the limiting factor.
        ConnectionKind::Local => 400 * MIB,
        ConnectionKind::Thunderbolt => 800 * MIB,
        // USB 3.x flash sticks often write far below SuperSpeed peak.
        ConnectionKind::Usb3 => 120 * MIB,
        // USB 2.0 High Speed practical bulk write.
        ConnectionKind::Usb2 => 25 * MIB,
        // SMB/NFS-ish shared folders.
        ConnectionKind::Network => 40 * MIB,
    }
}

/// Map USB bcdUSB-style version nibble to a connection class when available.
///
/// - bcdUSB 0x0200 / 0x0210 → USB2
/// - bcdUSB 0x0300+ → USB3
pub fn connection_from_bcd_usb(bcd_usb: u16) -> ConnectionKind {
    let major = bcd_usb >> 8;
    if major >= 3 {
        ConnectionKind::Usb3
    } else {
        // USB 1.x / 2.x → treat as USB2-class practical rates.
        ConnectionKind::Usb2
    }
}

/// Estimate a pack unit size optimized for fast, predictable transfers.
///
/// Formula:
/// ```text
/// raw = throughput_B/s × target_secs
/// unit = clamp(raw, MIN_UNIT, MAX_UNIT)
/// if free_bytes known:
///   ceiling = free - mapping_reserve
///   unit = min(unit, ceiling)
/// ```
pub fn estimate_transfer_unit(req: &EstimateRequest) -> TransferEstimate {
    let secs = req.target_transfer_secs.max(1);
    let throughput = req
        .facts
        .throughput_bytes_per_sec
        .filter(|&t| t > 0)
        .unwrap_or_else(|| practical_throughput_bytes_per_sec(req.facts.connection));

    let raw = throughput.saturating_mul(secs);
    let mut unit = raw.clamp(MIN_UNIT_BYTES, MAX_UNIT_BYTES);

    let ceiling = if req.facts.free_bytes > 0 {
        let reserve = mapping_reserve(req.facts.free_bytes);
        Some(req.facts.free_bytes.saturating_sub(reserve).max(1))
    } else {
        None
    };

    if let Some(c) = ceiling {
        unit = unit.min(c).max(1);
    }

    TransferEstimate {
        recommended_unit_bytes: unit,
        media_ceiling_bytes: ceiling,
        assumed_throughput_bytes_per_sec: throughput,
        target_transfer_secs: secs,
        connection: req.facts.connection,
    }
}

/// Human-oriented explanation lines for CLI output.
pub fn format_estimate_report(est: &TransferEstimate) -> String {
    format_estimate_report_with_meta(est, None, None, None)
}

/// Extended report including detection / free space.
pub fn format_estimate_report_with_meta(
    est: &TransferEstimate,
    detection: Option<&str>,
    free_bytes: Option<u64>,
    measured: Option<bool>,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "recommended_unit: {} ({})",
        est.recommended_unit_bytes,
        human_bytes(est.recommended_unit_bytes)
    ));
    lines.push(format!("connection: {:?}", est.connection));
    if let Some(d) = detection {
        lines.push(format!("detection: {d}"));
    }
    if let Some(m) = measured {
        lines.push(format!("measured: {m}"));
    }
    lines.push(format!(
        "assumed_throughput: {}/s",
        human_bytes(est.assumed_throughput_bytes_per_sec)
    ));
    lines.push(format!(
        "target_transfer_window: {}s",
        est.target_transfer_secs
    ));
    if let Some(free) = free_bytes {
        if free > 0 {
            lines.push(format!("free_bytes: {} ({})", free, human_bytes(free)));
        }
    }
    if let Some(c) = est.media_ceiling_bytes {
        lines.push(format!("media_ceiling: {} ({})", c, human_bytes(c)));
    }
    lines.push(
        "note: unit size targets ~one archive transfer within the window; \
         not the full media capacity."
            .into(),
    );
    lines.join("\n")
}

pub fn human_bytes(n: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let x = n as f64;
    if x >= GIB {
        format!("{:.2} GiB", x / GIB)
    } else if x >= MIB {
        format!("{:.2} MiB", x / MIB)
    } else if x >= KIB {
        format!("{:.2} KiB", x / KIB)
    } else {
        format!("{n} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usb2_unit_smaller_than_usb3() {
        let usb2 = estimate_transfer_unit(&EstimateRequest {
            facts: MediaFacts {
                free_bytes: 64 * 1024 * 1024 * 1024,
                total_bytes: 64 * 1024 * 1024 * 1024,
                connection: ConnectionKind::Usb2,
                throughput_bytes_per_sec: None,
            },
            target_transfer_secs: 45,
        });
        let usb3 = estimate_transfer_unit(&EstimateRequest {
            facts: MediaFacts {
                free_bytes: 64 * 1024 * 1024 * 1024,
                total_bytes: 64 * 1024 * 1024 * 1024,
                connection: ConnectionKind::Usb3,
                throughput_bytes_per_sec: None,
            },
            target_transfer_secs: 45,
        });
        assert!(usb2.recommended_unit_bytes < usb3.recommended_unit_bytes);
        // USB2: 25MiB/s * 45s ≈ 1125 MiB
        assert_eq!(usb2.recommended_unit_bytes, 25 * 1024 * 1024 * 45);
    }

    #[test]
    fn free_space_caps_unit() {
        let est = estimate_transfer_unit(&EstimateRequest {
            facts: MediaFacts {
                free_bytes: 8 * 1024 * 1024, // 8 MiB free
                total_bytes: 16 * 1024 * 1024,
                connection: ConnectionKind::Usb3,
                throughput_bytes_per_sec: None,
            },
            target_transfer_secs: 45,
        });
        assert!(est.recommended_unit_bytes <= 8 * 1024 * 1024);
        assert!(est.media_ceiling_bytes.is_some());
    }

    #[test]
    fn measured_throughput_overrides_default() {
        let est = estimate_transfer_unit(&EstimateRequest {
            facts: MediaFacts {
                free_bytes: 0,
                total_bytes: 0,
                connection: ConnectionKind::Usb2,
                throughput_bytes_per_sec: Some(10 * 1024 * 1024),
            },
            target_transfer_secs: 10,
        });
        assert_eq!(est.recommended_unit_bytes, 10 * 1024 * 1024 * 10);
    }

    #[test]
    fn bcd_usb_mapping() {
        assert_eq!(connection_from_bcd_usb(0x0200), ConnectionKind::Usb2);
        assert_eq!(connection_from_bcd_usb(0x0210), ConnectionKind::Usb2);
        assert_eq!(connection_from_bcd_usb(0x0300), ConnectionKind::Usb3);
        assert_eq!(connection_from_bcd_usb(0x0310), ConnectionKind::Usb3);
    }
}
