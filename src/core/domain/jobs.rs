//! Compress worker count: default cap and I/O-aware auto-tune (`--at` / `--auto-jobs`).

use super::media::ConnectionKind;
use super::transfer_estimate::practical_throughput_bytes_per_sec;

/// Hard ceiling for auto-tuned jobs (dashboard height / resource sanity).
pub const AUTO_JOBS_MAX: usize = 32;

/// Default jobs without `--at`: min(CPU, 8).
pub const DEFAULT_JOBS_MAX: usize = 8;

/// Inputs for auto-tuning after plan (and optional media context).
#[derive(Debug, Clone, Copy)]
pub struct AutoJobsContext {
    pub cpu: usize,
    pub units_pending: usize,
    pub files: usize,
    /// Destination attachment class (`--connection`). Affects I/O cap.
    pub connection: Option<ConnectionKind>,
    /// Measured or CLI override write throughput (bytes/s). When set, refines I/O cap.
    pub throughput_bps: Option<u64>,
    /// Stage on a separate work-dir (compress less contended with media write).
    pub work_dir: bool,
    /// Local pack under output (no external-media assumption).
    pub compress_only: bool,
}

impl Default for AutoJobsContext {
    fn default() -> Self {
        Self {
            cpu: available_cpu_threads(),
            units_pending: 1,
            files: 0,
            connection: None,
            throughput_bps: None,
            work_dir: false,
            compress_only: false,
        }
    }
}

/// Logical CPU threads available to this process (at least 1).
pub fn available_cpu_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .max(1)
}

/// Classic default: clamp CPU count to [`DEFAULT_JOBS_MAX`].
pub fn default_compress_jobs() -> usize {
    available_cpu_threads().min(DEFAULT_JOBS_MAX).max(1)
}

/// Provisional upper bound when auto-jobs is on (before plan is known).
/// Progress UI may start with this many rows, then shrink after plan.
pub fn auto_jobs_provisional_cap() -> usize {
    available_cpu_threads().min(AUTO_JOBS_MAX).max(1)
}

/// Max concurrent compress workers suggested by media / disk class.
///
/// Transfer is single-threaded, but compress workers still contend on **source
/// reads** and (without `--work-dir`) on the **output volume**. Slow media →
/// fewer threads so archive completion stays smoother and stage pressure drops.
pub fn io_jobs_cap(
    connection: Option<ConnectionKind>,
    throughput_bps: Option<u64>,
    work_dir: bool,
    compress_only: bool,
) -> usize {
    let conn = connection.unwrap_or(if compress_only {
        ConnectionKind::Local
    } else {
        // Media pack without --connection: assume USB3-class, not full local SSD.
        ConnectionKind::Usb3
    });

    let t = throughput_bps
        .filter(|&x| x > 0)
        .unwrap_or_else(|| practical_throughput_bytes_per_sec(conn));

    const MIB: u64 = 1024 * 1024;
    // Throughput bands → concurrency ceiling.
    let mut cap = if t >= 350 * MIB {
        AUTO_JOBS_MAX
    } else if t >= 200 * MIB {
        10
    } else if t >= 100 * MIB {
        6 // typical USB3 flash practical write
    } else if t >= 50 * MIB {
        4
    } else if t >= 20 * MIB {
        3 // USB2 / modest network
    } else {
        2 // slow stick / bad path
    };

    // Connection class hard ceiling (throughput alone can be optimistic).
    let class_cap = match conn {
        ConnectionKind::Local | ConnectionKind::Thunderbolt => AUTO_JOBS_MAX,
        ConnectionKind::Usb3 => 6,
        ConnectionKind::Usb2 => 3,
        ConnectionKind::Network => 3,
    };
    cap = cap.min(class_cap);

    // Compress writing into the same volume as the final output → more contention.
    if !work_dir && !compress_only {
        cap = ((cap * 3) / 4).max(1);
    }

    // Separate work-dir: transfer is isolated; allow one extra slot up to class_cap.
    if work_dir && !compress_only {
        cap = (cap + 1).min(class_cap).min(AUTO_JOBS_MAX);
    }

    cap.max(1)
}

/// Auto-tune compress workers from CPU, plan size, and I/O class.
///
/// Rules (when `auto` and no explicit `-j`):
/// 1. Never exceed logical CPU threads (capped at [`AUTO_JOBS_MAX`]).
/// 2. Never exceed **units still to build**.
/// 3. Never exceed **I/O cap** from connection / throughput / work-dir.
/// 4. Tiny file counts keep jobs small.
/// 5. Dense trees may raise toward the above caps (not above them).
///
/// `explicit_jobs` (from `-j`) always wins when `Some`.
pub fn resolve_compress_jobs(
    explicit_jobs: Option<usize>,
    auto: bool,
    ctx: &AutoJobsContext,
) -> usize {
    if let Some(j) = explicit_jobs {
        return j.max(1);
    }
    let cpu = ctx.cpu.max(1);
    if !auto {
        return cpu.min(DEFAULT_JOBS_MAX).max(1);
    }

    let cpu_cap = cpu.min(AUTO_JOBS_MAX);
    let units = ctx.units_pending.max(1);
    let io_cap = io_jobs_cap(
        ctx.connection,
        ctx.throughput_bps,
        ctx.work_dir,
        ctx.compress_only,
    );

    let mut j = cpu_cap.min(units).min(io_cap);

    // Tiny trees: avoid oversubscription.
    if ctx.files > 0 && ctx.files < 4 {
        j = j.min(ctx.files);
    }

    // Many files + multiple units: nudge up, still bounded by cpu/units/io.
    if ctx.files >= 128 && units > 1 {
        let from_files = ctx.files.saturating_add(127) / 128;
        let raised = from_files.min(cpu_cap).min(units).min(io_cap);
        j = j.max(raised);
    }

    j.max(1)
}

/// Short label for logs (e.g. `usb3`).
pub fn connection_label(c: Option<ConnectionKind>) -> &'static str {
    match c {
        None => "default",
        Some(ConnectionKind::Local) => "local",
        Some(ConnectionKind::Usb2) => "usb2",
        Some(ConnectionKind::Usb3) => "usb3",
        Some(ConnectionKind::Thunderbolt) => "thunderbolt",
        Some(ConnectionKind::Network) => "network",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(
        cpu: usize,
        units: usize,
        files: usize,
        conn: Option<ConnectionKind>,
        work_dir: bool,
    ) -> AutoJobsContext {
        AutoJobsContext {
            cpu,
            units_pending: units,
            files,
            connection: conn,
            throughput_bps: None,
            work_dir,
            compress_only: false,
        }
    }

    #[test]
    fn explicit_wins() {
        let c = ctx(16, 100, 10_000, Some(ConnectionKind::Local), true);
        assert_eq!(resolve_compress_jobs(Some(3), true, &c), 3);
        assert_eq!(resolve_compress_jobs(Some(1), false, &c), 1);
    }

    #[test]
    fn default_caps_at_eight() {
        let c = ctx(16, 100, 10_000, Some(ConnectionKind::Local), true);
        assert_eq!(resolve_compress_jobs(None, false, &c), 8);
        let c4 = ctx(4, 100, 10_000, None, false);
        assert_eq!(resolve_compress_jobs(None, false, &c4), 4);
    }

    #[test]
    fn auto_local_uses_cpu_up_to_units() {
        let c = ctx(16, 31, 5_000, Some(ConnectionKind::Local), true);
        assert_eq!(resolve_compress_jobs(None, true, &c), 16);
        let c2 = ctx(16, 4, 5_000, Some(ConnectionKind::Local), true);
        assert_eq!(resolve_compress_jobs(None, true, &c2), 4);
    }

    #[test]
    fn auto_usb2_caps_io() {
        let c = ctx(16, 31, 5_000, Some(ConnectionKind::Usb2), true);
        let j = resolve_compress_jobs(None, true, &c);
        // class cap 3, work_dir allows +1 → min(3+1, class 3) = 3
        assert!(j <= 3, "j={j}");
        assert!(j >= 2, "j={j}");
    }

    #[test]
    fn auto_usb3_with_work_dir() {
        let c = ctx(16, 31, 5_000, Some(ConnectionKind::Usb3), true);
        let j = resolve_compress_jobs(None, true, &c);
        // class 6, work_dir +1 capped at class → 6
        assert_eq!(j, 6, "j={j}");
    }

    #[test]
    fn auto_usb3_without_work_dir_tighter() {
        let with = resolve_compress_jobs(
            None,
            true,
            &ctx(16, 31, 5_000, Some(ConnectionKind::Usb3), true),
        );
        let without = resolve_compress_jobs(
            None,
            true,
            &ctx(16, 31, 5_000, Some(ConnectionKind::Usb3), false),
        );
        assert!(
            without <= with,
            "without work_dir should not exceed with: {without} vs {with}"
        );
        assert!(without <= 5, "without={without}");
    }

    #[test]
    fn auto_slow_throughput_overrides() {
        let mut c = ctx(16, 31, 5_000, Some(ConnectionKind::Usb3), false);
        c.throughput_bps = Some(10 * 1024 * 1024); // 10 MiB/s → band 2, no work_dir tighter
        let j = resolve_compress_jobs(None, true, &c);
        assert!(j <= 2, "j={j}");
    }

    #[test]
    fn auto_tiny_file_count() {
        let c = ctx(16, 10, 2, Some(ConnectionKind::Local), true);
        assert_eq!(resolve_compress_jobs(None, true, &c), 2);
    }

    #[test]
    fn io_cap_local_is_open() {
        assert_eq!(
            io_jobs_cap(Some(ConnectionKind::Local), None, true, false),
            AUTO_JOBS_MAX
        );
    }
}
