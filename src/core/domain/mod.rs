mod config_file;
mod content_class;
mod error;
mod format;
mod mapping;
mod media;
mod pack_plan;
mod pack_state;
mod path_safety;
mod resource_check;
mod transfer_estimate;
mod volume_assign;
mod write_estimate;

pub use config_file::{load_ini_file, parse_ini, IniConfig};
pub use content_class::{
    classify_extension, classify_path, combined_ratio, estimate_archive_for_source_files,
    estimate_archive_for_sources, ClassBreakdown, ContentAwareEstimate, ContentClass,
};
pub use error::ArchiveError;
pub use format::ArchiveFormat;
pub use mapping::{MappingArchive, MappingManifest, MappingMember, MAPPING_VERSION};
pub use media::{
    effective_budget, mapping_reserve, parse_size, parse_size_with_preset, ConnectionKind,
    MediaKind, SizePreset,
};
pub use pack_plan::{
    assign_archive_paths, plan_pack, plan_pack_with_progress, PackPlan, PackUnit, SourceFile,
};
pub use pack_state::{
    fingerprint_members, PackPlanFile, PlanUnitRecord, PlanUnitStatus, PLAN_FILE_NAME,
    PLAN_FILE_VERSION,
};
pub use resource_check::{
    estimate_disk_requirement, evaluate_resources, DiskRequirement, ResourceFinding, ResourceLevel,
    ResourcePlanInput,
};
pub use path_safety::{is_safe_member_path, normalize_member_path, safe_join};
pub use transfer_estimate::{
    connection_from_bcd_usb, estimate_transfer_unit, format_estimate_report,
    format_estimate_report_with_meta, human_bytes, practical_throughput_bytes_per_sec,
    EstimateRequest, MediaFacts, TransferEstimate, DEFAULT_TARGET_SECS, MAX_UNIT_BYTES,
    MIN_UNIT_BYTES,
};
pub use volume_assign::{
    assign_volume_paths, budget_from_volume_count, count_volumes_needed,
};
pub use write_estimate::{
    compression_ratio, compression_ratio_for_format, estimate_archive_bytes,
    estimated_total_write_bytes, eta_input_secs, eta_transfer_secs, eta_wall_clock,
    write_progress_pct, EtaBreakdown,
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
