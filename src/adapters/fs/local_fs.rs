use crate::core::domain::ArchiveError;
use crate::core::ports::FileSystem;
use std::path::Path;

#[derive(Debug, Default)]
pub struct LocalFileSystem;

impl LocalFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for LocalFileSystem {
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

    fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, ArchiveError> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::copy(from, to)?)
    }

    fn remove_file(&self, path: &Path) -> Result<(), ArchiveError> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(ArchiveError::Io(e)),
        }
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), ArchiveError> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(ArchiveError::Io(e)),
        }
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), ArchiveError> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(from, to)?;
        Ok(())
    }
}
