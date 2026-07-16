use crate::core::domain::{ArchiveError, ArchiveFormat};
use crate::core::ports::ArchiveEngine;
use std::path::Path;

/// Use case: extract an archive into a destination directory.
pub struct ExtractArchive<'a, E: ArchiveEngine> {
    engine: &'a E,
}

impl<'a, E: ArchiveEngine> ExtractArchive<'a, E> {
    pub fn new(engine: &'a E) -> Self {
        Self { engine }
    }

    pub fn execute(
        &self,
        archive: &Path,
        dest: &Path,
        format: Option<ArchiveFormat>,
    ) -> Result<(), ArchiveError> {
        if !archive.exists() {
            return Err(ArchiveError::ArchiveNotFound(archive.display().to_string()));
        }

        let format = match format {
            Some(f) => f,
            None => ArchiveFormat::from_path(archive)?,
        };

        self.engine.extract(format, archive, dest)
    }
}
