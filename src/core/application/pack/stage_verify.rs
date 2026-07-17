//! Optional pre-transfer member integrity + archive blob hash compare.

use crate::core::domain::{
    normalize_member_path, safe_join, ArchiveError, ArchiveFormat, MappingMember,
};
use crate::core::interrupt;
use crate::core::ports::{ArchiveEngine, ContentHasher, FileSystem};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Extract staged archive and re-hash members against pack-time member SHA-256.
pub fn verify_members_in_archive<E, F, H>(
    engine: &E,
    fs: &F,
    hasher: &H,
    format: ArchiveFormat,
    archive_path: &Path,
    expected: &[MappingMember],
) -> Result<(), ArchiveError>
where
    E: ArchiveEngine,
    F: FileSystem,
    H: ContentHasher,
{
    let stage = make_tmp_dir(fs, "member-verify")?;
    let cleanup = Guard {
        fs,
        path: stage.clone(),
    };

    engine.extract(format, archive_path, &stage)?;

    for m in expected {
        interrupt::check()?;
        if m.sha256.is_empty() {
            return Err(ArchiveError::Mapping(format!(
                "verify-stage: empty member sha256: {}",
                m.source_relpath
            )));
        }
        let path = safe_join(&stage, &m.source_relpath)?;
        if !fs.exists(&path) {
            return Err(ArchiveError::Engine(format!(
                "verify-stage: member missing after extract: {}",
                m.source_relpath
            )));
        }
        let actual = hasher.hash_file(&path)?;
        if actual != m.sha256 {
            return Err(ArchiveError::HashMismatch {
                path: m.source_relpath.clone(),
                expected: m.sha256.clone(),
                actual,
            });
        }
        // size check
        let len = fs.file_len(&path)?;
        if len != m.size {
            return Err(ArchiveError::Engine(format!(
                "verify-stage: size mismatch for {}: got {len}, expected {}",
                m.source_relpath, m.size
            )));
        }
        let _ = normalize_member_path(&m.source_relpath)?;
    }

    drop(cleanup);
    Ok(())
}

struct Guard<'a, F: FileSystem> {
    fs: &'a F,
    path: std::path::PathBuf,
}

impl<F: FileSystem> Drop for Guard<'_, F> {
    fn drop(&mut self) {
        let _ = self.fs.remove_dir_all(&self.path);
    }
}

fn make_tmp_dir(fs: &dyn FileSystem, tag: &str) -> Result<std::path::PathBuf, ArchiveError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let base = std::env::var_os("TMPDIR")
        .or_else(|| std::env::var_os("TEMP"))
        .or_else(|| std::env::var_os("TMP"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(".archiver-verify"));
    let dir = base.join(format!(
        "archiver-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    fs.create_dir_all(&dir)?;
    Ok(dir)
}
