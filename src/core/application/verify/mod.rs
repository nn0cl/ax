//! Verify packed archives against a mapping (no user restore path).

mod one_archive;
mod report;
mod stage;

pub use report::VerifyReport;

use crate::core::domain::{normalize_member_path, ArchiveError, ArchiveFormat};
use crate::core::ports::{
    ArchiveEngine, ContentHasher, FileSystem, MappingStore, NullProgress, PackProgress,
    ProgressSnapshot,
};
use one_archive::verify_one_archive;
use std::path::{Path, PathBuf};

/// Input for mapping-based integrity verify.
#[derive(Debug, Clone)]
pub struct VerifyRequest {
    pub mapping_path: PathBuf,
    /// Root that `archive_relpath` entries resolve against.
    pub archives_root: PathBuf,
    /// When true: presence + member path listing only (no extract / hash).
    pub quick: bool,
}

/// Use case: integrity-check packed media against the JSON mapping.
pub struct VerifyMapping<'a, E, M, F, H>
where
    E: ArchiveEngine,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
{
    engine: &'a E,
    mapping: &'a M,
    fs: &'a F,
    hasher: &'a H,
}

impl<'a, E, M, F, H> VerifyMapping<'a, E, M, F, H>
where
    E: ArchiveEngine,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
{
    pub fn new(engine: &'a E, mapping: &'a M, fs: &'a F, hasher: &'a H) -> Self {
        Self {
            engine,
            mapping,
            fs,
            hasher,
        }
    }

    pub fn execute(&self, req: &VerifyRequest) -> Result<VerifyReport, ArchiveError> {
        self.execute_with_progress(req, &NullProgress)
    }

    pub fn execute_with_progress(
        &self,
        req: &VerifyRequest,
        progress: &dyn PackProgress,
    ) -> Result<VerifyReport, ArchiveError> {
        let manifest = self.mapping.read(&req.mapping_path)?;
        let format = ArchiveFormat::parse(&manifest.format)?;
        let mut report = VerifyReport {
            archives_total: manifest.archives.len(),
            empty_dirs_total: manifest.empty_directories.len(),
            ..Default::default()
        };

        for m in &manifest.archives {
            report.members_total += m.members.len();
        }

        let logical_total: u64 = if req.quick {
            manifest
                .archives
                .iter()
                .flat_map(|a| a.members.iter())
                .map(|m| m.size)
                .sum()
        } else {
            manifest
                .archives
                .iter()
                .flat_map(|a| a.members.iter())
                .filter(|m| !m.sha256.is_empty())
                .map(|m| m.size)
                .sum()
        };

        report.messages.push(format!(
            "mapping={} format={} archives={} members={} quick={}",
            req.mapping_path.display(),
            format.as_str(),
            report.archives_total,
            report.members_total,
            req.quick
        ));
        report.messages.push(format!(
            "archives_root={}",
            req.archives_root.display()
        ));

        for empty in &manifest.empty_directories {
            if let Err(e) = normalize_member_path(empty) {
                report.issues.push(format!(
                    "invalid empty_directory path {empty:?}: {e}"
                ));
            }
        }

        let mut logical_done: u64 = 0;
        let mut archives_done: usize = 0;
        let archives_total = report.archives_total;

        let snap = |phase: &'static str,
                    archives_done: usize,
                    logical_done: u64,
                    current: &str| ProgressSnapshot {
            units_done: archives_done,
            units_total: archives_total,
            logical_done,
            logical_total,
            logical_transferred: logical_done,
            written_done: logical_done,
            current: current.to_string(),
            phase,
        };

        progress.report(&snap(
            "verify",
            0,
            0,
            &req.mapping_path.display().to_string(),
        ));

        for archive in &manifest.archives {
            verify_one_archive(
                self.engine,
                self.fs,
                self.hasher,
                &req.archives_root,
                req.quick,
                format,
                archive,
                &mut report,
                progress,
                &snap,
                &mut logical_done,
                &mut archives_done,
            )?;
        }

        progress.report(&snap(
            "done",
            report.archives_total,
            logical_total,
            "done",
        ));
        progress.finish();

        Ok(report)
    }
}

/// When archives_root is omitted, use the mapping file's parent directory.
pub fn default_verify_archives_root(mapping_path: &Path) -> PathBuf {
    mapping_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}
