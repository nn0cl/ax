mod progress;

pub use progress::{NullProgress, PackProgress, ProgressSnapshot};

use crate::core::domain::{
    ArchiveError, ArchiveFormat, ArchiveMember, ConnectionKind, MappingManifest, MemberSpec,
    PackPlanFile, SourceFile,
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
    ) -> Result<(), ArchiveError> {
        self.create_members_with_progress(format, output, members, &|_, _| {})
    }

    /// Like [`create_members`], calling `on_member(index, spec)` before each member is packed.
    fn create_members_with_progress(
        &self,
        format: ArchiveFormat,
        output: &Path,
        members: &[MemberSpec],
        on_member: &dyn Fn(usize, &MemberSpec),
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

/// Live counters while walking the source tree.
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub files_found: usize,
    pub bytes_found: u64,
    pub dirs_seen: usize,
    pub current: String,
}

/// Port for scanning a source tree for pack.
pub trait FileScanner {
    fn scan_tree(&self, root: &Path) -> Result<(Vec<SourceFile>, Vec<String>), ArchiveError> {
        self.scan_tree_with_progress(root, &|_| {})
    }

    /// Walk `root`, optionally reporting progress (throttling is caller's choice).
    fn scan_tree_with_progress(
        &self,
        root: &Path,
        on_progress: &dyn Fn(ScanProgress),
    ) -> Result<(Vec<SourceFile>, Vec<String>), ArchiveError>;
}

/// Port for reading/writing the JSON mapping manifest and pack plan.
pub trait MappingStore: Sync {
    fn write(&self, path: &Path, manifest: &MappingManifest) -> Result<(), ArchiveError>;
    fn read(&self, path: &Path) -> Result<MappingManifest, ArchiveError>;
    fn write_plan(&self, path: &Path, plan: &PackPlanFile) -> Result<(), ArchiveError>;
    fn read_plan(&self, path: &Path) -> Result<PackPlanFile, ArchiveError>;
}

/// Minimal filesystem operations used by application use cases.
pub trait FileSystem: Sync {
    fn is_dir(&self, path: &Path) -> bool;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> Result<(), ArchiveError>;
    fn file_len(&self, path: &Path) -> Result<u64, ArchiveError>;
    fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, ArchiveError>;

    /// Copy with progress: `on_progress(bytes_copied_so_far, total_bytes)`.
    /// Default calls [`copy_file`] once and reports completion only.
    fn copy_file_with_progress(
        &self,
        from: &Path,
        to: &Path,
        on_progress: &dyn Fn(u64, u64),
    ) -> Result<u64, ArchiveError> {
        let n = self.copy_file(from, to)?;
        on_progress(n, n);
        Ok(n)
    }

    fn remove_file(&self, path: &Path) -> Result<(), ArchiveError>;
    fn remove_dir_all(&self, path: &Path) -> Result<(), ArchiveError>;
    fn rename(&self, from: &Path, to: &Path) -> Result<(), ArchiveError>;

    /// Atomically create a new file with `contents` (fails if it already exists).
    /// Used for exclusive pack locks.
    fn create_new_file(&self, path: &Path, contents: &[u8]) -> Result<(), ArchiveError>;

    /// Read entire file as UTF-8 (lossy not used — invalid UTF-8 → error string path).
    fn read_to_string(&self, path: &Path) -> Result<String, ArchiveError>;
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

/// Free disk bytes and optional system memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeSpace {
    pub free_bytes: u64,
    pub total_bytes: u64,
}

/// Port for local resource inspection (own machine + destination volumes).
pub trait ResourceProbe: Sync {
    fn disk_space(&self, path: &Path) -> Result<VolumeSpace, ArchiveError>;
    /// Best-effort available memory (None if unknown).
    fn available_memory_bytes(&self) -> Option<u64>;
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
