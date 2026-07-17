//! Exclusive pack locks for output / work-dir (prevent concurrent writers).

use crate::core::domain::ArchiveError;
use crate::core::ports::FileSystem;
use std::path::{Path, PathBuf};

/// Lock file name under a pack target directory.
pub const PACK_LOCK_NAME: &str = ".ax-pack.lock";

/// RAII guard that deletes lock files on drop (best-effort).
#[derive(Debug)]
pub struct PackLockGuard {
    paths: Vec<PathBuf>,
}

impl Drop for PackLockGuard {
    fn drop(&mut self) {
        for p in self.paths.drain(..) {
            let _ = std::fs::remove_file(&p);
        }
    }
}

/// Acquire exclusive locks on `output` and optional `work_dir` base.
///
/// Uses create-new lock files (atomic via `create_new` on local FS).
/// On conflict returns [`ArchiveError::PackBusy`] with lock path and contents.
pub fn acquire_pack_locks(
    fs: &dyn FileSystem,
    output: &Path,
    work_dir: Option<&Path>,
) -> Result<PackLockGuard, ArchiveError> {
    let mut paths = Vec::new();

    fs.create_dir_all(output)?;
    let out_lock = output.join(PACK_LOCK_NAME);
    create_lock(fs, &out_lock)?;
    paths.push(out_lock);

    if let Some(wd) = work_dir {
        fs.create_dir_all(wd)?;
        let wd_lock = wd.join(PACK_LOCK_NAME);
        if !paths.iter().any(|p| p == &wd_lock) {
            match create_lock(fs, &wd_lock) {
                Ok(()) => paths.push(wd_lock),
                Err(e) => {
                    for p in &paths {
                        let _ = fs.remove_file(p);
                    }
                    return Err(e);
                }
            }
        }
    }

    Ok(PackLockGuard { paths })
}

fn create_lock(fs: &dyn FileSystem, path: &Path) -> Result<(), ArchiveError> {
    let body = format!(
        "ax pack lock\npid={}\ncreated_unix={}\n",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    match fs.create_new_file(path, body.as_bytes()) {
        Ok(()) => Ok(()),
        Err(ArchiveError::AlreadyExists(_)) => {
            let hint = fs
                .read_to_string(path)
                .unwrap_or_else(|_| "(unreadable lock file)".into());
            let hint = hint.trim().replace('\n', "; ");
            Err(ArchiveError::PackBusy(format!(
                "another pack holds {} ({hint}). Wait for it to finish, or remove the lock if stale",
                path.display()
            )))
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ports::FileSystem;
    use std::path::Path;

    /// Minimal FS backed by the real OS for lock tests only.
    struct OsFs;
    impl FileSystem for OsFs {
        fn is_dir(&self, path: &Path) -> bool {
            path.is_dir()
        }
        fn exists(&self, path: &Path) -> bool {
            path.exists()
        }
        fn create_dir_all(&self, path: &Path) -> Result<(), ArchiveError> {
            std::fs::create_dir_all(path)?;
            Ok(())
        }
        fn file_len(&self, path: &Path) -> Result<u64, ArchiveError> {
            Ok(std::fs::metadata(path)?.len())
        }
        fn copy_file(&self, _: &Path, _: &Path) -> Result<u64, ArchiveError> {
            Err(ArchiveError::Engine("unused".into()))
        }
        fn remove_file(&self, path: &Path) -> Result<(), ArchiveError> {
            let _ = std::fs::remove_file(path);
            Ok(())
        }
        fn remove_dir_all(&self, path: &Path) -> Result<(), ArchiveError> {
            let _ = std::fs::remove_dir_all(path);
            Ok(())
        }
        fn rename(&self, _: &Path, _: &Path) -> Result<(), ArchiveError> {
            Err(ArchiveError::Engine("unused".into()))
        }
        fn create_new_file(&self, path: &Path, contents: &[u8]) -> Result<(), ArchiveError> {
            use std::io::Write;
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                Ok(mut f) => {
                    f.write_all(contents)?;
                    Ok(())
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    Err(ArchiveError::AlreadyExists(path.display().to_string()))
                }
                Err(e) => Err(ArchiveError::Io(e)),
            }
        }
        fn read_to_string(&self, path: &Path) -> Result<String, ArchiveError> {
            Ok(std::fs::read_to_string(path)?)
        }
    }

    #[test]
    fn second_acquire_fails() {
        let dir = tempfile::tempdir().unwrap();
        let fs = OsFs;
        let g1 = acquire_pack_locks(&fs, dir.path(), None).unwrap();
        let err = acquire_pack_locks(&fs, dir.path(), None).unwrap_err();
        match err {
            ArchiveError::PackBusy(msg) => assert!(msg.contains(PACK_LOCK_NAME), "{msg}"),
            other => panic!("expected PackBusy, got {other}"),
        }
        drop(g1);
        let g2 = acquire_pack_locks(&fs, dir.path(), None).unwrap();
        drop(g2);
    }

    #[test]
    fn work_dir_and_output_both_locked() {
        let out = tempfile::tempdir().unwrap();
        let wd = tempfile::tempdir().unwrap();
        let fs = OsFs;
        let g = acquire_pack_locks(&fs, out.path(), Some(wd.path())).unwrap();
        assert!(out.path().join(PACK_LOCK_NAME).exists());
        assert!(wd.path().join(PACK_LOCK_NAME).exists());
        drop(g);
        assert!(!out.path().join(PACK_LOCK_NAME).exists());
        assert!(!wd.path().join(PACK_LOCK_NAME).exists());
    }
}
