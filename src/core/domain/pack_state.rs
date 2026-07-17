//! Durable pack plan / progress file for archive-unit resume.

use super::{ArchiveFormat, MappingMember, PackUnit};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PLAN_FILE_VERSION: u32 = 1;
pub const PLAN_FILE_NAME: &str = "archiver-plan.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanUnitStatus {
    Pending,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanUnitRecord {
    pub id: String,
    pub archive_relpath: String,
    pub logical_bytes: u64,
    /// Stable id of member set (paths + sizes). Detects tree/content changes.
    pub fingerprint: String,
    pub members: Vec<MappingMember>,
    pub status: PlanUnitStatus,
    /// On-disk archive size when status is Done.
    #[serde(default)]
    pub archive_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackPlanFile {
    pub version: u32,
    pub budget: u64,
    pub format: String,
    pub size_basis: String,
    #[serde(default)]
    pub empty_directories: Vec<String>,
    pub units: Vec<PlanUnitRecord>,
}

impl PackPlanFile {
    pub fn from_planned_units(
        budget: u64,
        format: ArchiveFormat,
        empty_directories: Vec<String>,
        units: &[(String, String, &PackUnit)], // id, archive_relpath, unit
    ) -> Self {
        let records = units
            .iter()
            .map(|(id, rel, unit)| PlanUnitRecord::from_pack_unit(id, rel, unit))
            .collect();
        Self {
            version: PLAN_FILE_VERSION,
            budget,
            format: format.as_str().to_string(),
            size_basis: "logical".to_string(),
            empty_directories,
            units: records,
        }
    }
}

impl PlanUnitRecord {
    pub fn from_pack_unit(id: &str, archive_relpath: &str, unit: &PackUnit) -> Self {
        let mut members: Vec<MappingMember> = unit
            .files
            .iter()
            .map(|f| MappingMember {
                source_relpath: f.rel_path.clone(),
                size: f.size,
                sha256: String::new(),
            })
            .collect();
        members.sort_by(|a, b| a.source_relpath.cmp(&b.source_relpath));
        let fingerprint = fingerprint_members(&members);
        Self {
            id: id.to_string(),
            archive_relpath: archive_relpath.to_string(),
            logical_bytes: unit.logical_bytes(),
            fingerprint,
            members,
            status: PlanUnitStatus::Pending,
            archive_bytes: None,
        }
    }

    pub fn mark_done(&mut self, members: Vec<MappingMember>, archive_bytes: u64) {
        self.members = members;
        self.members
            .sort_by(|a, b| a.source_relpath.cmp(&b.source_relpath));
        self.fingerprint = fingerprint_members(&self.members);
        self.status = PlanUnitStatus::Done;
        self.archive_bytes = Some(archive_bytes);
    }
}

/// Fingerprint from relative path + size only (not content hash).
pub fn fingerprint_members(members: &[MappingMember]) -> String {
    let mut lines: Vec<String> = members
        .iter()
        .map(|m| format!("{}\t{}", m.source_relpath, m.size))
        .collect();
    lines.sort();
    let mut hasher = Sha256::new();
    for line in lines {
        hasher.update(line.as_bytes());
        hasher.update([0xff]);
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_stable_under_reorder() {
        let a = vec![
            MappingMember {
                source_relpath: "b.txt".into(),
                size: 2,
                sha256: String::new(),
            },
            MappingMember {
                source_relpath: "a.txt".into(),
                size: 1,
                sha256: String::new(),
            },
        ];
        let b = vec![
            MappingMember {
                source_relpath: "a.txt".into(),
                size: 1,
                sha256: String::new(),
            },
            MappingMember {
                source_relpath: "b.txt".into(),
                size: 2,
                sha256: String::new(),
            },
        ];
        assert_eq!(fingerprint_members(&a), fingerprint_members(&b));
    }

    #[test]
    fn fingerprint_changes_when_size_changes() {
        let a = vec![MappingMember {
            source_relpath: "a.txt".into(),
            size: 1,
            sha256: String::new(),
        }];
        let b = vec![MappingMember {
            source_relpath: "a.txt".into(),
            size: 2,
            sha256: String::new(),
        }];
        assert_ne!(fingerprint_members(&a), fingerprint_members(&b));
    }
}
