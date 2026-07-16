use crate::core::domain::{ArchiveError, MappingManifest};
use crate::core::ports::MappingStore;
use std::fs;
use std::path::Path;

#[derive(Debug, Default)]
pub struct JsonMappingStore;

impl JsonMappingStore {
    pub fn new() -> Self {
        Self
    }
}

impl MappingStore for JsonMappingStore {
    fn write(&self, path: &Path, manifest: &MappingManifest) -> Result<(), ArchiveError> {
        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| ArchiveError::Mapping(e.to_string()))?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, json)?;
        Ok(())
    }

    fn read(&self, path: &Path) -> Result<MappingManifest, ArchiveError> {
        let data = fs::read_to_string(path)?;
        serde_json::from_str(&data).map_err(|e| ArchiveError::Mapping(e.to_string()))
    }
}
