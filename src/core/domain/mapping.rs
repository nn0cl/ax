use super::ArchiveFormat;
use serde::{Deserialize, Serialize};

/// Mapping schema version. v2 requires member sha256.
pub const MAPPING_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MappingManifest {
    pub version: u32,
    pub source_root_name: String,
    pub media_budget_bytes: u64,
    pub format: String,
    pub size_basis: String,
    #[serde(default)]
    pub empty_directories: Vec<String>,
    pub archives: Vec<MappingArchive>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MappingArchive {
    pub id: String,
    pub archive_relpath: String,
    pub logical_bytes: u64,
    pub members: Vec<MappingMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MappingMember {
    pub source_relpath: String,
    pub size: u64,
    /// Lowercase hex SHA-256 of source file bytes at pack time.
    #[serde(default)]
    pub sha256: String,
}

impl MappingManifest {
    pub fn new(
        source_root_name: String,
        media_budget_bytes: u64,
        format: ArchiveFormat,
        empty_directories: Vec<String>,
        archives: Vec<MappingArchive>,
    ) -> Self {
        Self {
            version: MAPPING_VERSION,
            source_root_name,
            media_budget_bytes,
            format: format.as_str().to_string(),
            size_basis: "logical".to_string(),
            empty_directories,
            archives,
        }
    }
}
