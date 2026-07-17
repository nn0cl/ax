//! Pack command inputs and outcomes.

use crate::core::domain::{ArchiveError, ArchiveFormat, ConnectionKind, PackPlan};
use std::path::{Path, PathBuf};

/// Input for a media-fit pack operation.
#[derive(Debug, Clone)]
pub struct PackRequest {
    pub source: PathBuf,
    pub output: PathBuf,
    pub budget: u64,
    pub format: ArchiveFormat,
    pub mapping_path: PathBuf,
    pub dry_run: bool,
    /// 0 quiet, 1 (`-v`) light, 2+ (`-vv`) detail (workers / per-unit logs).
    pub verbose: u8,
    /// Compress workers. With `auto_jobs`, this is a provisional upper bound until plan.
    pub jobs: usize,
    /// When true and jobs were not fixed by `-j`, retune after plan (CPU × units/files).
    pub auto_jobs: bool,
    /// When true, `jobs` came from `-j` / config and must not be replaced by auto formula
    /// (still clamped to at least 1).
    pub jobs_explicit: bool,
    /// Destination connection class for I/O-aware auto-jobs (`--connection`).
    pub connection: Option<ConnectionKind>,
    /// Optional write throughput (bytes/s) for auto-jobs (CLI `--throughput` or measure).
    pub throughput_bps: Option<u64>,
    /// Base directory for a private random stage subdir (other files left alone).
    pub work_dir: Option<PathBuf>,
    /// Keep the random stage after pack. Default: always delete.
    pub keep_stage: bool,
    /// Resume using existing `archiver-plan.json` under output.
    pub resume: bool,
    /// Check free disk and soft memory before writing.
    pub check_resources: bool,
    /// Continue even if resource check fails.
    pub force_resources: bool,
    /// Multiplier for disk estimates (1.0 ≈ remaining logical on output).
    pub resource_safety_factor: f64,
    /// Complete local pack under `--output` (no external-media assumption).
    pub compress_only: bool,
    /// Place archives under `vol-001/` … (max this many volumes).
    pub volumes: Option<u32>,
    /// After compress: re-extract stage archive and match member SHA-256;
    /// after transfer: require dest archive SHA-256 == stage archive SHA-256.
    pub verify_stage: bool,
}

impl Default for PackRequest {
    fn default() -> Self {
        Self {
            source: PathBuf::new(),
            output: PathBuf::new(),
            budget: 0,
            format: ArchiveFormat::Zip,
            mapping_path: PathBuf::new(),
            dry_run: false,
            verbose: 0,
            jobs: 1,
            auto_jobs: false,
            jobs_explicit: false,
            connection: None,
            throughput_bps: None,
            work_dir: None,
            keep_stage: false,
            resume: false,
            check_resources: true,
            force_resources: false,
            resource_safety_factor: 1.0,
            compress_only: false,
            volumes: None,
            verify_stage: false,
        }
    }
}

#[derive(Debug)]
pub struct PackResult {
    pub plan: PackPlan,
    pub dry_run: bool,
    pub messages: Vec<String>,
}

/// Pack failure, optionally with a stage directory still on disk.
#[derive(Debug)]
pub struct PackFailure {
    pub error: ArchiveError,
    /// Present when stage was kept (`--keep-stage`) or could not be removed.
    pub leftover_stage: Option<PathBuf>,
}

impl From<ArchiveError> for PackFailure {
    fn from(error: ArchiveError) -> Self {
        Self {
            error,
            leftover_stage: None,
        }
    }
}

impl std::fmt::Display for PackFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

pub fn default_mapping_path(output: &Path) -> PathBuf {
    output.join("archiver-map.json")
}
