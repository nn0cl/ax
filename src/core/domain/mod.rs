mod error;
mod format;
mod mapping;
mod media;
mod pack_plan;
mod path_safety;
mod transfer_estimate;

pub use error::ArchiveError;
pub use format::ArchiveFormat;
pub use mapping::{MappingArchive, MappingManifest, MappingMember, MAPPING_VERSION};
pub use media::{effective_budget, mapping_reserve, parse_size, ConnectionKind, MediaKind};
pub use pack_plan::{
    assign_archive_paths, plan_pack, PackPlan, PackUnit, SourceFile,
};
pub use path_safety::{is_safe_member_path, normalize_member_path, safe_join};
pub use transfer_estimate::{
    connection_from_bcd_usb, estimate_transfer_unit, format_estimate_report,
    format_estimate_report_with_meta, human_bytes, practical_throughput_bytes_per_sec,
    EstimateRequest, MediaFacts, TransferEstimate, DEFAULT_TARGET_SECS, MAX_UNIT_BYTES,
    MIN_UNIT_BYTES,
};

/// Metadata for one member inside an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveMember {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Explicit member for archive creation: absolute source path + name inside archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberSpec {
    pub source: std::path::PathBuf,
    pub member_name: String,
}
