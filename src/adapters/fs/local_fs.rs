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
}
