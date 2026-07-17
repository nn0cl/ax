use crate::core::domain::{ArchiveError, MappingManifest, PackPlanFile};
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

fn write_json_atomic(path: &Path, json: &str) -> Result<(), ArchiveError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let tmp = path.with_extension("json.partial");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

impl MappingStore for JsonMappingStore {
    fn write(&self, path: &Path, manifest: &MappingManifest) -> Result<(), ArchiveError> {
        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| ArchiveError::Mapping(e.to_string()))?;
        write_json_atomic(path, &json)
    }

    fn read(&self, path: &Path) -> Result<MappingManifest, ArchiveError> {
        let data = fs::read_to_string(path)?;
        let manifest: MappingManifest =
            serde_json::from_str(&data).map_err(|e| ArchiveError::Mapping(e.to_string()))?;
        if manifest.version != crate::core::domain::MAPPING_VERSION {
            return Err(ArchiveError::Mapping(format!(
                "unsupported mapping version {} (need {})",
                manifest.version,
                crate::core::domain::MAPPING_VERSION
            )));
        }
        for a in &manifest.archives {
            for m in &a.members {
                if m.sha256.is_empty() {
                    return Err(ArchiveError::Mapping(format!(
                        "member missing sha256: {}",
                        m.source_relpath
                    )));
                }
            }
        }
        Ok(manifest)
    }

    fn write_plan(&self, path: &Path, plan: &PackPlanFile) -> Result<(), ArchiveError> {
        let json =
            serde_json::to_string_pretty(plan).map_err(|e| ArchiveError::Mapping(e.to_string()))?;
        write_json_atomic(path, &json)
    }

    fn read_plan(&self, path: &Path) -> Result<PackPlanFile, ArchiveError> {
        let data = fs::read_to_string(path)?;
        let plan: PackPlanFile =
            serde_json::from_str(&data).map_err(|e| ArchiveError::Mapping(e.to_string()))?;
        if plan.version != crate::core::domain::PLAN_FILE_VERSION {
            return Err(ArchiveError::Mapping(format!(
                "unsupported plan version {} (need {})",
                plan.version,
                crate::core::domain::PLAN_FILE_VERSION
            )));
        }
        Ok(plan)
    }
}
