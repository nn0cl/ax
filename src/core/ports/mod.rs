use crate::core::domain::{
    ArchiveError, ArchiveFormat, ArchiveMember, ConnectionKind, MappingManifest, MemberSpec,
    SourceFile,
};
use std::path::{Path, PathBuf};

/// Port for archive format engines (ZIP, TAR, TAR.GZ, TAR.ZST, ...).
pub trait ArchiveEngine: Sync {
    fn create(
        &self,
        format: ArchiveFormat,
        output: &Path,
        inputs: &[PathBuf],
    ) -> Result<(), ArchiveError>;

    fn create_members(
        &self,
        format: ArchiveFormat,
        output: &Path,
        members: &[MemberSpec],
    ) -> Result<(), ArchiveError>;

    fn list(
        &self,
        format: ArchiveFormat,
        archive: &Path,
    ) -> Result<Vec<ArchiveMember>, ArchiveError>;

    fn extract(
        &self,
        format: ArchiveFormat,
        archive: &Path,
        dest: &Path,
    ) -> Result<(), ArchiveError>;
}

/// Port for scanning a source tree for pack.
pub trait FileScanner {
    fn scan_tree(&self, root: &Path) -> Result<(Vec<SourceFile>, Vec<String>), ArchiveError>;
}

/// Port for reading/writing the JSON mapping manifest.
pub trait MappingStore {
    fn write(&self, path: &Path, manifest: &MappingManifest) -> Result<(), ArchiveError>;
    fn read(&self, path: &Path) -> Result<MappingManifest, ArchiveError>;
}

/// Minimal filesystem operations used by application use cases.
pub trait FileSystem: Sync {
    fn is_dir(&self, path: &Path) -> bool;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> Result<(), ArchiveError>;
}

/// Streaming content hash (SHA-256 hex lowercase).
pub trait ContentHasher: Sync {
    fn hash_file(&self, path: &Path) -> Result<String, ArchiveError>;
}

/// Probed destination volume characteristics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbedMedia {
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub connection: ConnectionKind,
    pub throughput_bytes_per_sec: Option<u64>,
    /// Human-readable detection source (e.g. `cli_hint`, `path_heuristic:/Volumes`).
    pub detection: String,
}

/// Port for inspecting a destination path.
pub trait MediaProbe {
    fn probe(
        &self,
        path: &Path,
        connection_hint: Option<ConnectionKind>,
    ) -> Result<ProbedMedia, ArchiveError>;
}

/// Optional sequential write sample for throughput estimation.
pub trait ThroughputMeter {
    /// Write a temporary sample under `path`'s volume and return bytes/sec.
    fn measure_write_throughput(
        &self,
        path: &Path,
        sample_bytes: u64,
    ) -> Result<u64, ArchiveError>;
}
