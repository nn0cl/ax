use crate::core::domain::{ArchiveError, ArchiveFormat, ArchiveMember};
use crate::core::ports::ArchiveEngine;
use std::path::Path;

/// Use case: list members of an archive.
pub struct ListArchive<'a, E: ArchiveEngine> {
    engine: &'a E,
}

impl<'a, E: ArchiveEngine> ListArchive<'a, E> {
    pub fn new(engine: &'a E) -> Self {
        Self { engine }
    }

    pub fn execute(
        &self,
        archive: &Path,
        format: Option<ArchiveFormat>,
    ) -> Result<Vec<ArchiveMember>, ArchiveError> {
        if !archive.exists() {
            return Err(ArchiveError::ArchiveNotFound(archive.display().to_string()));
        }

        let format = match format {
            Some(f) => f,
            None => ArchiveFormat::from_path(archive)?,
        };

        self.engine.list(format, archive)
    }
}
