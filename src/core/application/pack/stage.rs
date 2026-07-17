//! Ephemeral work-dir stage (random subdir; never reused for resume).

use crate::core::domain::ArchiveError;
use crate::core::ports::FileSystem;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique directory name under `--work-dir` (does not touch sibling files).
pub fn new_stage_dirname() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id() as u128;
    let token = nanos
        ^ (pid << 48)
        ^ pid.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ nanos.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    format!("archiver-{token:032x}")
}

/// Create a private stage under `work_dir`. Returns stage path and log lines.
pub fn create_stage_dir(
    fs: &dyn FileSystem,
    work_dir: &Path,
    keep_stage: bool,
) -> Result<(PathBuf, Vec<String>), ArchiveError> {
    let stage = work_dir.join(new_stage_dirname());
    fs.create_dir_all(&stage)?;
    let mut messages = Vec::new();
    messages.push(format!("stage_root={}", stage.display()));
    messages.push(format!(
        "stage_policy: random subdir under --work-dir; \
         {} after pack; resume trusts only output+plan, never stage",
        if keep_stage {
            "kept (--keep-stage)"
        } else {
            "deleted by default (success or fail)"
        }
    ));
    Ok((stage, messages))
}

/// Ensure `--work-dir` exists and is a directory (create if missing).
pub fn ensure_work_dir_base(fs: &dyn FileSystem, work_dir: &Path) -> Result<(), ArchiveError> {
    if !fs.exists(work_dir) {
        fs.create_dir_all(work_dir)?;
    } else if !fs.is_dir(work_dir) {
        return Err(ArchiveError::Engine(format!(
            "--work-dir exists but is not a directory: {}",
            work_dir.display()
        )));
    }
    Ok(())
}
