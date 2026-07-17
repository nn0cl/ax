//! Verify packed archives against a mapping without restoring to a user path.

use crate::core::domain::{normalize_member_path, safe_join, ArchiveError, ArchiveFormat};
use crate::core::ports::{ArchiveEngine, ContentHasher, FileSystem, MappingStore};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Input for mapping-based integrity verify.
#[derive(Debug, Clone)]
pub struct VerifyRequest {
    pub mapping_path: PathBuf,
    /// Root that `archive_relpath` entries resolve against.
    pub archives_root: PathBuf,
    /// When true: presence + member path listing only (no extract / hash).
    pub quick: bool,
}

/// Aggregate result of a verify run.
#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    pub archives_total: usize,
    pub archives_ok: usize,
    pub members_total: usize,
    pub members_checked: usize,
    pub members_skipped_no_hash: usize,
    pub empty_dirs_total: usize,
    /// Human-readable issues (empty ⇒ success).
    pub issues: Vec<String>,
    pub messages: Vec<String>,
}

impl VerifyReport {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn summary_line(&self) -> String {
        if self.is_ok() {
            format!(
                "verify ok: {} archive(s), {} member(s) checked, {} empty dir(s){}",
                self.archives_ok,
                self.members_checked,
                self.empty_dirs_total,
                if self.members_skipped_no_hash > 0 {
                    format!(
                        ", {} member(s) without sha256 skipped",
                        self.members_skipped_no_hash
                    )
                } else {
                    String::new()
                }
            )
        } else {
            format!(
                "verify failed: {} issue(s); {}/{} archive(s) ok, {}/{} member(s) checked",
                self.issues.len(),
                self.archives_ok,
                self.archives_total,
                self.members_checked,
                self.members_total
            )
        }
    }
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

        // Empty directories are only in the mapping (not inside archives).
        for empty in &manifest.empty_directories {
            if let Err(e) = normalize_member_path(empty) {
                report.issues.push(format!(
                    "invalid empty_directory path {empty:?}: {e}"
                ));
            }
        }

        for archive in &manifest.archives {
            self.verify_one_archive(req, format, archive, &mut report)?;
        }

        Ok(report)
    }

    fn verify_one_archive(
        &self,
        req: &VerifyRequest,
        format: ArchiveFormat,
        archive: &crate::core::domain::MappingArchive,
        report: &mut VerifyReport,
    ) -> Result<(), ArchiveError> {
        let archive_path = req.archives_root.join(&archive.archive_relpath);
        if !self.fs.exists(&archive_path) {
            report.issues.push(format!(
                "archive missing: {} ({})",
                archive.archive_relpath,
                archive_path.display()
            ));
            return Ok(());
        }

        let listed = match self.engine.list(format, &archive_path) {
            Ok(m) => m,
            Err(e) => {
                report.issues.push(format!(
                    "cannot list {}: {e}",
                    archive.archive_relpath
                ));
                return Ok(());
            }
        };

        let mut listed_files: HashSet<String> = HashSet::new();
        for m in &listed {
            if m.is_dir {
                continue;
            }
            match normalize_member_path(&m.path) {
                Ok(p) => {
                    listed_files.insert(p);
                }
                Err(_) => {
                    // Ignore unsafe/odd entries for set membership; extract would reject them.
                }
            }
        }

        let mut archive_ok = true;
        for member in &archive.members {
            let path = match normalize_member_path(&member.source_relpath) {
                Ok(p) => p,
                Err(e) => {
                    report.issues.push(format!(
                        "{}: invalid member path {}: {e}",
                        archive.archive_relpath, member.source_relpath
                    ));
                    archive_ok = false;
                    continue;
                }
            };
            if !listed_files.contains(&path) {
                report.issues.push(format!(
                    "{}: member missing from archive listing: {path}",
                    archive.archive_relpath
                ));
                archive_ok = false;
            }
        }

        if req.quick {
            if archive_ok {
                report.archives_ok += 1;
                report.members_checked += archive.members.len();
            }
            return Ok(());
        }

        // Full verify: extract to temp, hash members against mapping.
        let stage = make_stage_dir(self.fs, &archive.id)?;
        let stage_cleanup = StageGuard {
            fs: self.fs,
            path: stage.clone(),
        };

        if let Err(e) = self.engine.extract(format, &archive_path, &stage) {
            report.issues.push(format!(
                "{}: extract failed: {e}",
                archive.archive_relpath
            ));
            drop(stage_cleanup);
            return Ok(());
        }

        for member in &archive.members {
            if member.sha256.is_empty() {
                report.members_skipped_no_hash += 1;
                continue;
            }
            let path = match safe_join(&stage, &member.source_relpath) {
                Ok(p) => p,
                Err(e) => {
                    report.issues.push(format!(
                        "{}: {}: {e}",
                        archive.archive_relpath, member.source_relpath
                    ));
                    archive_ok = false;
                    continue;
                }
            };
            if !self.fs.exists(&path) {
                report.issues.push(format!(
                    "{}: extracted file missing: {}",
                    archive.archive_relpath, member.source_relpath
                ));
                archive_ok = false;
                continue;
            }
            match self.hasher.hash_file(&path) {
                Ok(actual) if actual == member.sha256 => {
                    report.members_checked += 1;
                }
                Ok(actual) => {
                    report.issues.push(format!(
                        "{}: hash mismatch for {}: expected {}, got {}",
                        archive.archive_relpath, member.source_relpath, member.sha256, actual
                    ));
                    archive_ok = false;
                }
                Err(e) => {
                    report.issues.push(format!(
                        "{}: hash failed for {}: {e}",
                        archive.archive_relpath, member.source_relpath
                    ));
                    archive_ok = false;
                }
            }
        }

        if archive_ok {
            report.archives_ok += 1;
        }
        drop(stage_cleanup);
        Ok(())
    }
}

struct StageGuard<'a, F: FileSystem> {
    fs: &'a F,
    path: PathBuf,
}

impl<F: FileSystem> Drop for StageGuard<'_, F> {
    fn drop(&mut self) {
        let _ = self.fs.remove_dir_all(&self.path);
    }
}

fn make_stage_dir(fs: &dyn FileSystem, archive_id: &str) -> Result<PathBuf, ArchiveError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let safe_id: String = archive_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    // Prefer OS temp when available; fall back to relative .archiver-verify for pure ports.
    let base = std::env::var_os("TMPDIR")
        .or_else(|| std::env::var_os("TEMP"))
        .or_else(|| std::env::var_os("TMP"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".archiver-verify"));
    let dir = base.join(format!(
        "archiver-verify-{}-{}-{}",
        std::process::id(),
        safe_id,
        nanos
    ));
    fs.create_dir_all(&dir)?;
    Ok(dir)
}

/// When archives_root is omitted, use the mapping file's parent directory.
pub fn default_verify_archives_root(mapping_path: &Path) -> PathBuf {
    mapping_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}
