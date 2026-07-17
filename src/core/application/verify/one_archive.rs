//! Verify a single archive from the mapping.

use super::report::VerifyReport;
use super::stage::{make_stage_dir, StageGuard};
use crate::core::domain::{
    normalize_member_path, safe_join, ArchiveError, ArchiveFormat, MappingArchive,
};
use crate::core::interrupt;
use crate::core::ports::{
    ArchiveEngine, ContentHasher, FileSystem, PackProgress, ProgressSnapshot,
};
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_one_archive<E, F, H>(
    engine: &E,
    fs: &F,
    hasher: &H,
    archives_root: &std::path::Path,
    quick: bool,
    format: ArchiveFormat,
    archive: &MappingArchive,
    report: &mut VerifyReport,
    progress: &dyn PackProgress,
    snap: &dyn Fn(&'static str, usize, u64, &str) -> ProgressSnapshot,
    logical_done: &mut u64,
    archives_done: &mut usize,
) -> Result<(), ArchiveError>
where
    E: ArchiveEngine,
    F: FileSystem,
    H: ContentHasher,
{
    let archive_path = archives_root.join(&archive.archive_relpath);
    progress.report(&snap(
        if quick { "list" } else { "extract" },
        *archives_done,
        *logical_done,
        &archive.archive_relpath,
    ));

    if !fs.exists(&archive_path) {
        report.issues.push(format!(
            "archive missing: {} ({})",
            archive.archive_relpath,
            archive_path.display()
        ));
        *archives_done += 1;
        progress.report(&snap(
            "verify",
            *archives_done,
            *logical_done,
            &archive.archive_relpath,
        ));
        return Ok(());
    }

    let listed = match engine.list(format, &archive_path) {
        Ok(m) => m,
        Err(e) => {
            report.issues.push(format!(
                "cannot list {}: {e}",
                archive.archive_relpath
            ));
            *archives_done += 1;
            progress.report(&snap(
                "verify",
                *archives_done,
                *logical_done,
                &archive.archive_relpath,
            ));
            return Ok(());
        }
    };

    let mut listed_files: HashSet<String> = HashSet::new();
    for m in &listed {
        if m.is_dir {
            continue;
        }
        if let Ok(p) = normalize_member_path(&m.path) {
            listed_files.insert(p);
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

    if quick {
        if archive_ok {
            report.archives_ok += 1;
            report.members_checked += archive.members.len();
        }
        for member in &archive.members {
            *logical_done = logical_done.saturating_add(member.size);
        }
        *archives_done += 1;
        progress.report(&snap(
            "list",
            *archives_done,
            *logical_done,
            &archive.archive_relpath,
        ));
        return Ok(());
    }

    interrupt::check()?;
    progress.report(&snap(
        "extract",
        *archives_done,
        *logical_done,
        &archive.archive_relpath,
    ));

    let stage = make_stage_dir(fs, &archive.id)?;
    let stage_cleanup = StageGuard {
        fs,
        path: stage.clone(),
    };

    interrupt::check()?;
    if let Err(e) = engine.extract(format, &archive_path, &stage) {
        report.issues.push(format!(
            "{}: extract failed: {e}",
            archive.archive_relpath
        ));
        drop(stage_cleanup);
        *archives_done += 1;
        progress.report(&snap(
            "verify",
            *archives_done,
            *logical_done,
            &archive.archive_relpath,
        ));
        return Ok(());
    }

    let mut member_i = 0usize;
    for member in &archive.members {
        interrupt::check()?;
        if member.sha256.is_empty() {
            report.issues.push(format!(
                "{}: member missing sha256 (mapping v2 requires pack-time hash): {}",
                archive.archive_relpath, member.source_relpath
            ));
            archive_ok = false;
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
        if !fs.exists(&path) {
            report.issues.push(format!(
                "{}: extracted file missing: {}",
                archive.archive_relpath, member.source_relpath
            ));
            archive_ok = false;
            continue;
        }
        match hasher.hash_file(&path) {
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
        *logical_done = logical_done.saturating_add(member.size);
        member_i += 1;
        if member_i % 32 == 0 || member.size >= 8 * 1024 * 1024 {
            progress.report(&snap(
                "verify",
                *archives_done,
                *logical_done,
                &format!("{} / {}", archive.archive_relpath, member.source_relpath),
            ));
        }
    }

    if archive_ok {
        report.archives_ok += 1;
    }
    drop(stage_cleanup);
    *archives_done += 1;
    progress.report(&snap(
        "verify",
        *archives_done,
        *logical_done,
        &archive.archive_relpath,
    ));
    Ok(())
}
