//! Build pack plan file and apply resume fingerprints (strict: no legacy size-only).

use crate::core::domain::{
    assign_archive_paths, assign_volume_paths, plan_pack_with_progress, ArchiveError, ArchiveFormat,
    PackPlan, PackPlanFile, PackUnit, PlanUnitRecord, PlanUnitStatus, SourceFile, PLAN_FILE_VERSION,
};
use crate::core::ports::{ContentHasher, FileSystem, PackProgress, ProgressSnapshot};
use std::collections::HashMap;
use std::path::Path;

/// Plan + optional volume layout + plan-file records (before resume matching).
pub fn build_pack_plan(
    files: Vec<SourceFile>,
    empty_dirs: Vec<String>,
    budget: u64,
    format: ArchiveFormat,
    volumes: Option<u32>,
    progress: &dyn PackProgress,
    logical_total: u64,
) -> Result<(PackPlan, PackPlanFile, Vec<String>), ArchiveError> {
    let mut messages = Vec::new();
    let file_count = files.len();

    let ratio_prior = format.prior_archive_ratio();
    progress.report(&ProgressSnapshot {
        units_done: 0,
        units_total: 0,
        logical_done: logical_total,
        logical_total,
        logical_transferred: 0,
        written_done: 0,
        current: format!("planning {file_count} files…"),
        phase: "plan",
        ratio_prior,
    });

    let plan = plan_pack_with_progress(files, empty_dirs, budget, &|dirs_done,
                                                                    dirs_total,
                                                                    units| {
        progress.report(&ProgressSnapshot {
            units_done: dirs_done,
            units_total: dirs_total,
            logical_done: logical_total,
            logical_total,
            logical_transferred: 0,
            written_done: 0,
            current: format!("{units} units so far"),
            phase: "plan",
            ratio_prior,
        });
    });

    let mut archive_paths = assign_archive_paths(&plan.units, format);
    if let Some(n) = volumes {
        let sizes: Vec<u64> = plan.units.iter().map(|u| u.logical_bytes()).collect();
        archive_paths = assign_volume_paths(&archive_paths, &sizes, budget, n as usize)?;
        messages.push(format!(
            "volumes={n} capacity≈{budget} layout=vol-NNN/…"
        ));
    }

    let unit_refs: Vec<(String, String, &PackUnit)> = plan
        .units
        .iter()
        .zip(archive_paths.iter())
        .enumerate()
        .map(|(idx, (unit, rel))| (format!("a{:04}", idx + 1), rel.clone(), unit))
        .collect();

    let plan_file = PackPlanFile::from_planned_units(
        budget,
        format,
        plan.empty_directories.clone(),
        &unit_refs
            .iter()
            .map(|(id, rel, u)| (id.clone(), rel.clone(), *u))
            .collect::<Vec<_>>(),
    );

    Ok((plan, plan_file, messages))
}

/// Mark plan units done when previous fingerprints match complete archives on output.
///
/// **No legacy fallback:** Done units without non-empty `archive_sha256` are not skipped.
/// Size + SHA-256 of the archive file must both match.
pub fn apply_resume(
    fs: &dyn FileSystem,
    hasher: &dyn ContentHasher,
    output: &Path,
    plan_file: &mut PackPlanFile,
    previous: &PackPlanFile,
) -> Result<(usize, usize), ArchiveError> {
    if previous.version != PLAN_FILE_VERSION {
        return Err(ArchiveError::Mapping(format!(
            "unsupported plan version {} (need {PLAN_FILE_VERSION}); re-pack without --resume",
            previous.version
        )));
    }

    let mut done_by_fp: HashMap<String, PlanUnitRecord> = HashMap::new();
    for u in &previous.units {
        if u.is_complete_done() {
            done_by_fp.insert(u.fingerprint.clone(), u.clone());
        }
    }

    let mut skipped = 0usize;
    let mut redo = 0usize;
    for rec in &mut plan_file.units {
        if let Some(prev) = done_by_fp.get(&rec.fingerprint) {
            let prev_path = output.join(&prev.archive_relpath);
            let expected_bytes = match prev.archive_bytes {
                Some(b) => b,
                None => {
                    redo += 1;
                    continue;
                }
            };
            let expected_sha = match prev.archive_sha256.as_ref() {
                Some(h) if !h.is_empty() => h,
                _ => {
                    redo += 1;
                    continue;
                }
            };
            if !fs.exists(&prev_path) {
                redo += 1;
                continue;
            }
            if fs.file_len(&prev_path)? != expected_bytes {
                redo += 1;
                continue;
            }
            let actual = hasher.hash_file(&prev_path)?;
            if actual != *expected_sha {
                redo += 1;
                continue;
            }

            let new_path = output.join(&rec.archive_relpath);
            if prev.archive_relpath != rec.archive_relpath {
                if let Some(parent) = new_path.parent() {
                    fs.create_dir_all(parent)?;
                }
                if !fs.exists(&new_path) {
                    fs.copy_file(&prev_path, &new_path)?;
                }
            }
            rec.members = prev.members.clone();
            rec.status = PlanUnitStatus::Done;
            rec.archive_bytes = prev.archive_bytes;
            rec.archive_sha256 = prev.archive_sha256.clone();
            skipped += 1;
            continue;
        }
        redo += 1;
    }
    Ok((skipped, redo))
}
