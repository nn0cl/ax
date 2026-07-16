use crate::core::domain::{ArchiveError, ArchiveFormat};
use crate::core::ports::{ArchiveEngine, FileSystem};
use std::path::{Path, PathBuf};

/// Use case: create an archive from input paths.
pub struct CreateArchive<'a, E: ArchiveEngine, F: FileSystem> {
    engine: &'a E,
    fs: &'a F,
}

impl<'a, E: ArchiveEngine, F: FileSystem> CreateArchive<'a, E, F> {
    pub fn new(engine: &'a E, fs: &'a F) -> Self {
        Self { engine, fs }
    }

    pub fn execute(
        &self,
        output: &Path,
        inputs: &[PathBuf],
        format: Option<ArchiveFormat>,
    ) -> Result<ArchiveFormat, ArchiveError> {
        if inputs.is_empty() {
            return Err(ArchiveError::EmptyInput);
        }

        for input in inputs {
            if !self.fs.exists(input) {
                return Err(ArchiveError::InputNotFound(input.display().to_string()));
            }
        }

        let format = match format {
            Some(f) => f,
            None => ArchiveFormat::from_path(output)?,
        };

        self.engine.create(format, output, inputs)?;
        Ok(format)
    }
}
