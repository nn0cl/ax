//! Per-unit compress work and staged-file finalize helpers.

use super::stage_verify::verify_members_in_archive;
use crate::core::domain::{ArchiveError, ArchiveFormat, MappingMember, MemberSpec, PackUnit};
use crate::core::interrupt;
use crate::core::ports::{
    ArchiveEngine, ContentHasher, FileSystem, PackProgress, ProgressSnapshot,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct PreparedUnit {
    pub id: String,
    pub archive_relpath: String,
    pub unit: PackUnit,
    pub members: Vec<MemberSpec>,
    #[allow(dead_code)]
    pub fingerprint: String,
}

pub struct StagedUnit {
    pub id: String,
    pub archive_relpath: String,
    pub logical_bytes: u64,
    pub mapped_members: Vec<MappingMember>,
    pub staged_path: PathBuf,
    /// SHA-256 of staged archive blob when `verify_stage` ran (pre-transfer).
    pub stage_sha256: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn report_xfer(
    progress: &dyn PackProgress,
    phase: &'static str,
    current: &str,
    units_done: usize,
    units_total: usize,
    logical_done: u64,
    logical_total: u64,
    logical_transferred: u64,
    written_done: u64,
    ratio_prior: f64,
) {
    progress.report(&ProgressSnapshot {
        units_done,
        units_total,
        logical_done,
        logical_total,
        logical_transferred,
        written_done,
        current: current.to_string(),
        phase,
        ratio_prior,
    });
}

/// Move staged archive into final path: prefer rename, else copy with progress.
pub fn finalize_staged<F: FileSystem>(
    fs: &F,
    staged: &Path,
    final_path: &Path,
    has_stage: bool,
    prefer_rename: bool,
    cleanup_stage: bool,
    on_progress: &dyn Fn(u64, u64),
) -> Result<u64, ArchiveError> {
    if !has_stage {
        fs.rename(staged, final_path)?;
        let n = fs.file_len(final_path)?;
        on_progress(n, n);
        return Ok(n);
    }
    if prefer_rename && fs.rename(staged, final_path).is_ok() {
        let n = fs.file_len(final_path)?;
        on_progress(n, n);
        return Ok(n);
    }
    let n = fs.copy_file_with_progress(staged, final_path, on_progress)?;
    if cleanup_stage {
        let _ = fs.remove_file(staged);
    }
    Ok(n)
}

fn compress_worker_id(jobs: usize) -> usize {
    rayon::current_thread_index()
        .unwrap_or(0)
        .min(jobs.saturating_sub(1))
}

/// Hash members + compress one unit to stage or `.partial` under output.
#[allow(clippy::too_many_arguments)]
pub fn compress_one<E, H, F>(
    p: &PreparedUnit,
    engine: &E,
    hasher: &H,
    fs: &F,
    format: ArchiveFormat,
    stage_root: Option<&PathBuf>,
    output: &Path,
    logical_done: &AtomicU64,
    units_done: &AtomicUsize,
    written_done: &AtomicU64,
    logical_transferred: &AtomicU64,
    units_total: usize,
    logical_total: u64,
    jobs: usize,
    verify_stage: bool,
    ratio_prior: f64,
    progress: &dyn PackProgress,
) -> Result<StagedUnit, ArchiveError>
where
    E: ArchiveEngine,
    H: ContentHasher,
    F: FileSystem,
{
    let wid = compress_worker_id(jobs);
    let snap = |phase: &'static str| ProgressSnapshot {
        units_done: units_done.load(Ordering::Relaxed),
        units_total,
        logical_done: logical_done.load(Ordering::Relaxed),
        logical_total,
        logical_transferred: logical_transferred.load(Ordering::Relaxed),
        written_done: written_done.load(Ordering::Relaxed),
        current: p.archive_relpath.clone(),
        phase,
        ratio_prior,
    };

    // Worker path column: show the **work item** (source while hashing; archive
    // while packing). Prefer rel_path so the dashboard can keep the file name.
    let first_src = p
        .unit
        .files
        .first()
        .map(|f| f.rel_path.as_str())
        .unwrap_or(p.archive_relpath.as_str());
    progress.report_worker(wid, "hash", first_src, first_src, 0);
    progress.report(&snap("hash"));

    let mut mapped_members = Vec::with_capacity(p.unit.files.len());
    for (i, (file, member)) in p.unit.files.iter().zip(p.members.iter()).enumerate() {
        if i % 32 == 0 {
            interrupt::check()?;
        }
        let digest = hasher.hash_file(&member.source)?;
        mapped_members.push(MappingMember {
            source_relpath: file.rel_path.clone(),
            size: file.size,
            sha256: digest,
        });
        logical_done.fetch_add(file.size, Ordering::Relaxed);
        // Compression *input* file currently being hashed.
        progress.report_worker(wid, "hash", &file.rel_path, &file.rel_path, file.size);
        progress.report(&snap("hash"));
    }
    interrupt::check()?;

    let staged_path = if let Some(stage) = stage_root {
        stage.join(&p.archive_relpath)
    } else {
        output.join(format!("{}.partial", p.archive_relpath))
    };
    if let Some(parent) = staged_path.parent() {
        fs.create_dir_all(parent)?;
    }

    // Compression *output* unit; per-member callback updates source for wide UI.
    progress.report_worker(wid, "compress", &p.archive_relpath, first_src, 0);
    progress.report(&snap("compress"));
    engine.create_members_with_progress(format, &staged_path, &p.members, &|i, _m| {
        let src = p
            .unit
            .files
            .get(i)
            .map(|f| f.rel_path.as_str())
            .unwrap_or("");
        let sz = p.unit.files.get(i).map(|f| f.size).unwrap_or(0);
        progress.report_worker(wid, "compress", &p.archive_relpath, src, sz);
    })?;

    let stage_sha256 = if verify_stage {
        progress.report_worker(wid, "verify-stage", &p.archive_relpath, "", 0);
        progress.report(&snap("verify-stage"));
        verify_members_in_archive(
            engine,
            fs,
            hasher,
            format,
            &staged_path,
            &mapped_members,
        )?;
        interrupt::check()?;
        let sha = hasher.hash_file(&staged_path)?;
        progress.report_worker(wid, "idle", "", "", 0);
        Some(sha)
    } else {
        progress.report_worker(wid, "idle", "", "", 0);
        None
    };

    Ok(StagedUnit {
        id: p.id.clone(),
        archive_relpath: p.archive_relpath.clone(),
        logical_bytes: p.unit.logical_bytes(),
        mapped_members,
        staged_path,
        stage_sha256,
    })
}
