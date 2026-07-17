//! Pack command inputs and outcomes.

use crate::core::domain::{ArchiveError, ArchiveFormat, PackPlan};
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
    pub verbose: bool,
    pub jobs: usize,
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
            verbose: false,
            jobs: 1,
            work_dir: None,
            keep_stage: false,
            resume: false,
            check_resources: true,
            force_resources: false,
            resource_safety_factor: 1.0,
            compress_only: false,
            volumes: None,
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
