//! Temp extract dir for full verify.

use crate::core::domain::ArchiveError;
use crate::core::ports::FileSystem;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct StageGuard<'a, F: FileSystem> {
    pub fs: &'a F,
    pub path: PathBuf,
}

impl<F: FileSystem> Drop for StageGuard<'_, F> {
    fn drop(&mut self) {
        let _ = self.fs.remove_dir_all(&self.path);
    }
}

pub fn make_stage_dir(fs: &dyn FileSystem, archive_id: &str) -> Result<PathBuf, ArchiveError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let safe_id: String = archive_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let base = std::env::var_os("TMPDIR")
        .or_else(|| std::env::var_os("TEMP"))
        .or_else(|| std::env::var_os("TMP"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".archiver-verify"));
    let dir = base.join(format!(
        "archiver-verify-{}-{}-{}",
        std::process::id(),
        safe_id,
        nanos
    ));
    fs.create_dir_all(&dir)?;
    Ok(dir)
}
