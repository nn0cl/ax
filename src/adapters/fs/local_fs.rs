use crate::core::domain::ArchiveError;
use crate::core::interrupt;
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
        self.copy_file_with_progress(from, to, &|_, _| {})
    }

    fn copy_file_with_progress(
        &self,
        from: &Path,
        to: &Path,
        on_progress: &dyn Fn(u64, u64),
    ) -> Result<u64, ArchiveError> {
        use std::io::{Read, Write};

        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let total = std::fs::metadata(from)?.len();
        let mut src = std::fs::File::open(from)?;
        let mut dst = std::fs::File::create(to)?;
        let mut buf = vec![0u8; 1024 * 1024]; // 1 MiB
        let mut copied = 0u64;
        let mut last_report = 0u64;
        on_progress(0, total);
        loop {
            interrupt::check()?;
            let n = src.read(&mut buf)?;
            if n == 0 {
                break;
            }
            dst.write_all(&buf[..n])?;
            copied += n as u64;
            // Throttle UI updates (~every 8 MiB).
            if copied == total || copied.saturating_sub(last_report) >= 8 * 1024 * 1024 {
                on_progress(copied, total);
                last_report = copied;
            }
        }
        dst.flush()?;
        on_progress(copied, total);
        Ok(copied)
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

    fn create_new_file(&self, path: &Path, contents: &[u8]) -> Result<(), ArchiveError> {
        use std::io::Write;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut f) => {
                f.write_all(contents)?;
                f.sync_all()?;
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
