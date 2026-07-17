//! Progress reporting for long-running pack operations.

/// Live snapshot for UI rendering (aggregate status).
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub units_done: usize,
    pub units_total: usize,
    pub logical_done: u64,
    pub logical_total: u64,
    pub written_done: u64,
    /// Current archive relative path (or phase label).
    pub current: String,
    /// Short phase: "hash", "compress", "write-map", "done", "scan", "plan", "transfer".
    pub phase: &'static str,
}

/// Optional sink for pack progress (implemented in delivery).
pub trait PackProgress: Send + Sync {
    fn report(&self, snap: &ProgressSnapshot);

    /// Per compress-worker update (used with `-v` for thread-level speeds).
    /// `logical_delta` is bytes processed in this tick (usually one file size).
    fn report_worker(
        &self,
        worker_id: usize,
        phase: &'static str,
        current: &str,
        logical_delta: u64,
    ) {
        let _ = (worker_id, phase, current, logical_delta);
    }

    /// Finish the display (e.g. emit newline after status block).
    fn finish(&self);
}

/// No-op reporter.
pub struct NullProgress;

impl PackProgress for NullProgress {
    fn report(&self, _snap: &ProgressSnapshot) {}
    fn finish(&self) {}
}
