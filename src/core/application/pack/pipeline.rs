//! Compress (parallel) + single transfer thread pipeline.

use super::unit_work::{
    compress_one, finalize_staged, report_xfer, PreparedUnit, StagedUnit,
};
use crate::core::domain::{ArchiveError, ArchiveFormat, PackPlanFile, PlanUnitStatus};
use crate::core::interrupt;
use crate::core::ports::{
    ArchiveEngine, ContentHasher, FileSystem, MappingStore, PackProgress,
};
// hasher used after transfer for archive_sha256
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PipelineParams<'a> {
    pub prepared: Vec<PreparedUnit>,
    pub plan_file: PackPlanFile,
    pub plan_path: &'a Path,
    pub output: &'a Path,
    pub stage_root: Option<PathBuf>,
    pub format: ArchiveFormat,
    pub jobs: usize,
    pub transfer_queue: usize,
    pub compress_only: bool,
    /// Pre-transfer member verify + post-transfer archive SHA match.
    pub verify_stage: bool,
    /// Content+algorithm prior ratio for write ETA (until first transfer).
    pub ratio_prior: f64,
    /// When true, record a short per-unit transfer log line (capped).
    pub log_transfers: bool,
    pub logical_total: u64,
    pub progress: &'a dyn PackProgress,
}

pub struct PipelinePorts<'a, E, M, F, H> {
    pub engine: &'a E,
    pub mapping: &'a M,
    pub fs: &'a F,
    pub hasher: &'a H,
}

pub struct PipelineOutcome {
    pub plan_file: PackPlanFile,
    pub logical_done: u64,
    pub written_done: u64,
    pub logical_transferred: u64,
    pub verbose_lines: Vec<String>,
}

/// Run hash/compress workers + serial transfer; updates plan_file on each transfer.
pub fn run_pipeline<E, M, F, H>(
    params: PipelineParams<'_>,
    ports: PipelinePorts<'_, E, M, F, H>,
) -> Result<PipelineOutcome, ArchiveError>
where
    E: ArchiveEngine,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
{
    let PipelineParams {
        prepared,
        plan_file,
        plan_path,
        output,
        stage_root,
        format,
        jobs,
        transfer_queue,
        compress_only,
        verify_stage,
        ratio_prior,
        log_transfers,
        logical_total,
        progress,
    } = params;

    let units_total = plan_file.units.len();
    let already_done = plan_file
        .units
        .iter()
        .filter(|u| u.status == PlanUnitStatus::Done)
        .count();
    let units_done = Arc::new(AtomicUsize::new(already_done));
    let logical_done = Arc::new(AtomicU64::new(
        plan_file
            .units
            .iter()
            .filter(|u| u.status == PlanUnitStatus::Done)
            .map(|u| u.logical_bytes)
            .sum(),
    ));
    let written_done = Arc::new(AtomicU64::new(
        plan_file
            .units
            .iter()
            .filter_map(|u| u.archive_bytes)
            .sum(),
    ));
    let logical_transferred = Arc::new(AtomicU64::new(
        plan_file
            .units
            .iter()
            .filter(|u| u.status == PlanUnitStatus::Done)
            .map(|u| u.logical_bytes)
            .sum(),
    ));
    let verbose_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let plan_file_mu: Arc<Mutex<PackPlanFile>> = Arc::new(Mutex::new(plan_file));
    let ratio_prior = if ratio_prior.is_finite() && ratio_prior > 0.0 {
        ratio_prior.clamp(0.05, 2.0)
    } else {
        format.prior_archive_ratio()
    };

    let report = |phase: &'static str, current: &str, units: usize| {
        progress.report(&crate::core::ports::ProgressSnapshot {
            units_done: units,
            units_total,
            logical_done: logical_done.load(Ordering::Relaxed),
            logical_total,
            logical_transferred: logical_transferred.load(Ordering::Relaxed),
            written_done: written_done.load(Ordering::Relaxed),
            current: current.to_string(),
            phase,
            ratio_prior,
        });
    };
    report("start", "", already_done);

    if !prepared.is_empty() {
        let (tx, rx) = sync_channel::<Result<StagedUnit, ArchiveError>>(transfer_queue);
        let transfer_has_stage = stage_root.is_some();
        let transfer_cleanup = transfer_has_stage;
        let prefer_rename = compress_only || !transfer_has_stage;

        thread::scope(|scope| -> Result<(), ArchiveError> {
            let transfer_units_done = Arc::clone(&units_done);
            let transfer_written = Arc::clone(&written_done);
            let transfer_logical = Arc::clone(&logical_done);
            let transfer_logical_xferred = Arc::clone(&logical_transferred);
            let transfer_plan = Arc::clone(&plan_file_mu);
            let transfer_verbose = Arc::clone(&verbose_buf);
            let fs_xfer = ports.fs;
            let mapping_xfer = ports.mapping;
            let hasher_xfer = ports.hasher;

            let transfer_handle = scope.spawn(move || -> Result<(), ArchiveError> {
                while let Ok(item) = rx.recv() {
                    interrupt::check()?;
                    let staged = item?;
                    let final_path = output.join(&staged.archive_relpath);

                    report_xfer(
                        progress,
                        "transfer",
                        &staged.archive_relpath,
                        transfer_units_done.load(Ordering::Relaxed),
                        units_total,
                        transfer_logical.load(Ordering::Relaxed),
                        logical_total,
                        transfer_logical_xferred.load(Ordering::Relaxed),
                        transfer_written.load(Ordering::Relaxed),
                        ratio_prior,
                    );

                    if let Some(parent) = final_path.parent() {
                        fs_xfer.create_dir_all(parent)?;
                    }

                    let base_written = transfer_written.load(Ordering::Relaxed);
                    let base_logical = transfer_logical.load(Ordering::Relaxed);
                    let base_xferred = transfer_logical_xferred.load(Ordering::Relaxed);
                    let base_units = transfer_units_done.load(Ordering::Relaxed);

                    let written = finalize_staged(
                        fs_xfer,
                        &staged.staged_path,
                        &final_path,
                        transfer_has_stage,
                        prefer_rename,
                        transfer_cleanup,
                        &|partial, _total| {
                            report_xfer(
                                progress,
                                "transfer",
                                &staged.archive_relpath,
                                base_units,
                                units_total,
                                base_logical,
                                logical_total,
                                base_xferred,
                                base_written.saturating_add(partial),
                                ratio_prior,
                            );
                        },
                    )?;

                    transfer_written.fetch_add(written, Ordering::Relaxed);
                    transfer_logical_xferred.fetch_add(staged.logical_bytes, Ordering::Relaxed);
                    let n = transfer_units_done.fetch_add(1, Ordering::Relaxed) + 1;

                    // Pack-time archive content hash (required for resume; no soft-skip).
                    let archive_sha = hasher_xfer.hash_file(&final_path)?;
                    if let Some(ref expected) = staged.stage_sha256 {
                        if archive_sha != *expected {
                            return Err(ArchiveError::HashMismatch {
                                path: staged.archive_relpath.clone(),
                                expected: expected.clone(),
                                actual: archive_sha,
                            });
                        }
                    }

                    if let Ok(mut pf) = transfer_plan.lock() {
                        if let Some(rec) = pf.units.iter_mut().find(|u| u.id == staged.id) {
                            rec.mark_done(staged.mapped_members.clone(), written, archive_sha);
                        }
                        let _ = mapping_xfer.write_plan(plan_path, &pf);
                    }

                    report_xfer(
                        progress,
                        "transfer",
                        &staged.archive_relpath,
                        n,
                        units_total,
                        transfer_logical.load(Ordering::Relaxed),
                        logical_total,
                        transfer_logical_xferred.load(Ordering::Relaxed),
                        transfer_written.load(Ordering::Relaxed),
                        ratio_prior,
                    );

                    if log_transfers {
                        if let Ok(mut buf) = transfer_verbose.lock() {
                            // Cap transfer log flood on large packs.
                            const XFER_LOG_CAP: usize = 30;
                            if buf.len() < XFER_LOG_CAP {
                                let verify_note = if staged.stage_sha256.is_some() {
                                    " verify_stage=ok"
                                } else {
                                    ""
                                };
                                buf.push(format!(
                                    "transferred {n}/{units_total} {} ({written} bytes){verify_note}",
                                    staged.archive_relpath
                                ));
                            } else if buf.len() == XFER_LOG_CAP {
                                buf.push(format!(
                                    "transferred: … further unit logs omitted (cap {XFER_LOG_CAP})"
                                ));
                            }
                        }
                    }
                }
                Ok(())
            });

            let logical_done_c = Arc::clone(&logical_done);
            let units_done_c = Arc::clone(&units_done);
            let stage_for_compress = stage_root.as_ref();

            let compress_result = if jobs == 1 {
                for p in &prepared {
                    interrupt::check()?;
                    let staged = compress_one(
                        p,
                        ports.engine,
                        ports.hasher,
                        ports.fs,
                        format,
                        stage_for_compress,
                        output,
                        &logical_done_c,
                        &units_done_c,
                        &written_done,
                        &logical_transferred,
                        units_total,
                        logical_total,
                        jobs,
                        verify_stage,
                        ratio_prior,
                        progress,
                    );
                    tx.send(staged)
                        .map_err(|_| ArchiveError::Engine("transfer channel closed".into()))?;
                }
                Ok(())
            } else {
                let written_done_c = Arc::clone(&written_done);
                let logical_xferred_c = Arc::clone(&logical_transferred);
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(jobs)
                    .build()
                    .map_err(|e| ArchiveError::Engine(e.to_string()))?;
                pool.install(|| {
                    prepared.par_iter().try_for_each(|p| {
                        interrupt::check()?;
                        let staged = compress_one(
                            p,
                            ports.engine,
                            ports.hasher,
                            ports.fs,
                            format,
                            stage_for_compress,
                            output,
                            &logical_done_c,
                            &units_done_c,
                            &written_done_c,
                            &logical_xferred_c,
                            units_total,
                            logical_total,
                            jobs,
                            verify_stage,
                            ratio_prior,
                            progress,
                        );
                        tx.send(staged).map_err(|_| {
                            ArchiveError::Engine("transfer channel closed".into())
                        })?;
                        Ok(()) as Result<(), ArchiveError>
                    })
                })
            };
            drop(tx);

            let transfer_result = transfer_handle
                .join()
                .map_err(|_| ArchiveError::Engine("transfer thread panicked".into()))?;
            compress_result?;
            transfer_result?;
            Ok(())
        })?;
    }

    let final_plan = plan_file_mu
        .lock()
        .map_err(|e| ArchiveError::Engine(format!("plan lock: {e}")))?
        .clone();

    let verbose_lines = verbose_buf
        .lock()
        .map(|mut b| std::mem::take(&mut *b))
        .unwrap_or_default();

    Ok(PipelineOutcome {
        plan_file: final_plan,
        logical_done: logical_done.load(Ordering::Relaxed),
        written_done: written_done.load(Ordering::Relaxed),
        logical_transferred: logical_transferred.load(Ordering::Relaxed),
        verbose_lines,
    })
}
