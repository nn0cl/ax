//! Progress reporting for long-running pack operations.

/// Live snapshot for UI rendering (aggregate status).
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub units_done: usize,
    pub units_total: usize,
    /// Source logical bytes processed so far (mainly hash progress).
    pub logical_done: u64,
    /// Total source logical bytes in the pack.
    pub logical_total: u64,
    /// Logical bytes of units fully written to the final output (transfer done).
    pub logical_transferred: u64,
    /// Archive bytes written to the final output so far.
    pub written_done: u64,
    /// Current archive relative path (or phase label).
    pub current: String,
    /// Short phase: "hash", "compress", "write-map", "done", "scan", "plan", "transfer".
    pub phase: &'static str,
    /// Format prior `archive/logical` used until the first transfer completes.
    /// See [`ArchiveFormat::prior_archive_ratio`](crate::core::domain::ArchiveFormat::prior_archive_ratio).
    pub ratio_prior: f64,
}

/// Optional sink for pack progress (implemented in delivery).
pub trait PackProgress: Send + Sync {
    fn report(&self, snap: &ProgressSnapshot);

    /// Per compress-worker update.
    /// `logical_delta` is bytes processed in this tick (usually one file size).
    ///
    /// `primary` — archive path while packing, or the sole path on hash.
    /// `source` — input file being processed (same as `primary` on hash; set on
    /// each compress member so wide terminals can show both). Empty keeps the
    /// previous source.
    fn report_worker(
        &self,
        worker_id: usize,
        phase: &'static str,
        primary: &str,
        source: &str,
        logical_delta: u64,
    ) {
        let _ = (worker_id, phase, primary, source, logical_delta);
    }

    /// Finish the display (e.g. emit newline after status block).
    fn finish(&self);

    /// Resize compress worker rows (e.g. after auto-jobs retune). Default: no-op.
    fn set_jobs(&self, jobs: usize) {
        let _ = jobs;
    }
}

/// No-op reporter.
pub struct NullProgress;

impl PackProgress for NullProgress {
    fn report(&self, _snap: &ProgressSnapshot) {}
    fn finish(&self) {}
}
