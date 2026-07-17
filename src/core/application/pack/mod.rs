//! Media-fit pack use case (orchestrates plan → pipeline → mapping).

mod pipeline;
mod plan_resume;
mod preflight;
mod request;
mod stage;
mod stage_verify;
mod unit_work;

pub use request::{default_mapping_path, PackFailure, PackRequest, PackResult};

use crate::core::domain::{
    budget_from_volume_count, estimate_archive_for_sources, human_bytes, ArchiveError,
    MappingArchive, MappingManifest, MemberSpec, PlanUnitStatus, PLAN_FILE_NAME,
};
use crate::core::ports::{
    ArchiveEngine, ContentHasher, FileScanner, FileSystem, MappingStore, NullProgress,
    PackProgress, ProgressSnapshot, ResourceProbe,
};
use pipeline::{run_pipeline, PipelineParams, PipelinePorts};
use plan_resume::{apply_resume, build_pack_plan};
use preflight::{check_resources, transfer_queue_depth};
use stage::{create_stage_dir, ensure_work_dir_base};
use std::path::PathBuf;
use std::time::Instant;
use unit_work::PreparedUnit;

pub struct PackSourceTree<'a, E, S, M, F, H, R>
where
    E: ArchiveEngine,
    S: FileScanner,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
    R: ResourceProbe,
{
    engine: &'a E,
    scanner: &'a S,
    mapping: &'a M,
    fs: &'a F,
    hasher: &'a H,
    resources: &'a R,
}

impl<'a, E, S, M, F, H, R> PackSourceTree<'a, E, S, M, F, H, R>
where
    E: ArchiveEngine,
    S: FileScanner,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
    R: ResourceProbe,
{
    pub fn new(
        engine: &'a E,
        scanner: &'a S,
        mapping: &'a M,
        fs: &'a F,
        hasher: &'a H,
        resources: &'a R,
    ) -> Self {
        Self {
            engine,
            scanner,
            mapping,
            fs,
            hasher,
            resources,
        }
    }

    pub fn execute(&self, req: &PackRequest) -> Result<PackResult, PackFailure> {
        self.execute_with_progress(req, &NullProgress)
    }

    pub fn execute_with_progress(
        &self,
        req: &PackRequest,
        progress: &dyn PackProgress,
    ) -> Result<PackResult, PackFailure> {
        let mut leftover_stage: Option<PathBuf> = None;
        match self.execute_with_progress_inner(req, progress, &mut leftover_stage) {
            Ok(mut result) => {
                if let Some(stage) = leftover_stage.take() {
                    if req.keep_stage {
                        result.messages.push(format!(
                            "kept stage {} (--keep-stage)",
                            stage.display()
                        ));
                    } else {
                        match self.fs.remove_dir_all(&stage) {
                            Ok(()) => result
                                .messages
                                .push(format!("cleaned stage {}", stage.display())),
                            Err(e) => result.messages.push(format!(
                                "warning: failed to remove stage {}: {e}",
                                stage.display()
                            )),
                        }
                    }
                }
                Ok(result)
            }
            Err(error) => {
                let left = if let Some(stage) = leftover_stage.take() {
                    if req.keep_stage {
                        Some(stage)
                    } else {
                        let _ = self.fs.remove_dir_all(&stage);
                        None
                    }
                } else {
                    None
                };
                Err(PackFailure {
                    error,
                    leftover_stage: left,
                })
            }
        }
    }

    fn execute_with_progress_inner(
        &self,
        req: &PackRequest,
        progress: &dyn PackProgress,
        leftover_stage: &mut Option<PathBuf>,
    ) -> Result<PackResult, ArchiveError> {
        if req.budget == 0 && req.volumes.is_none() {
            return Err(ArchiveError::InvalidBudget);
        }
        if let Some(n) = req.volumes {
            if n == 0 {
                return Err(ArchiveError::Engine("--volumes must be >= 1".into()));
            }
        }
        if !self.fs.is_dir(&req.source) {
            return Err(ArchiveError::SourceNotDirectory(
                req.source.display().to_string(),
            ));
        }
        if let Some(wd) = &req.work_dir {
            ensure_work_dir_base(self.fs, wd)?;
        }

        let wall_start = Instant::now();
        let plan_path = req.output.join(PLAN_FILE_NAME);
        let mut messages = Vec::new();

        let previous = if req.resume {
            if !self.fs.exists(&plan_path) {
                return Err(ArchiveError::Mapping(format!(
                    "--resume requires existing plan file: {}",
                    plan_path.display()
                )));
            }
            Some(self.mapping.read_plan(&plan_path)?)
        } else {
            None
        };

        let ratio_prior = req.format.prior_archive_ratio();
        progress.report(&ProgressSnapshot {
            units_done: 0,
            units_total: 0,
            logical_done: 0,
            logical_total: 0,
            logical_transferred: 0,
            written_done: 0,
            current: req.source.display().to_string(),
            phase: "scan",
            ratio_prior,
        });

        let (files, empty_dirs) = self.scanner.scan_tree_with_progress(
            &req.source,
            &|sp| {
                progress.report(&ProgressSnapshot {
                    units_done: sp.files_found,
                    units_total: 0,
                    logical_done: sp.bytes_found,
                    logical_total: 0,
                    logical_transferred: 0,
                    written_done: 0,
                    current: sp.current,
                    phase: "scan",
                    ratio_prior,
                });
            },
        )?;
        let logical_total: u64 = files.iter().map(|f| f.size).sum();

        let budget = if req.budget > 0 {
            req.budget
        } else if let Some(n) = req.volumes {
            let b = budget_from_volume_count(logical_total, n);
            messages.push(format!(
                "budget_from_volumes: volumes={n} total_logical={logical_total} budget={b}"
            ));
            b
        } else {
            return Err(ArchiveError::InvalidBudget);
        };

        let (plan, mut plan_file, plan_msgs) = build_pack_plan(
            files,
            empty_dirs,
            budget,
            req.format,
            req.volumes,
            progress,
            logical_total,
        )?;
        messages.extend(plan_msgs);

        let (skipped, redo) = if let Some(ref prev) = previous {
            apply_resume(self.fs, self.hasher, &req.output, &mut plan_file, prev)?
        } else {
            (0, plan_file.units.len())
        };

        messages.push(format!(
            "budget={} units={} empty_dirs={}",
            budget,
            plan.units.len(),
            plan.empty_directories.len()
        ));
        // Content class (extension) × algorithm prior for write ETA / size_estimate.
        let content_est = estimate_archive_for_sources(
            plan.units
                .iter()
                .flat_map(|u| u.files.iter().map(|f| (f.rel_path.as_str(), f.size))),
            req.format,
        );
        let ratio_prior = content_est.ratio;
        if req.verbose >= 2 {
            messages.push(content_est.summary_line());
        } else {
            messages.push(format!(
                "size_estimate: format={} logical={} archive≈{} ratio={:.3}",
                req.format.as_str(),
                crate::core::domain::human_bytes(content_est.logical_bytes),
                crate::core::domain::human_bytes(content_est.estimated_archive_bytes),
                content_est.ratio,
            ));
        }
        if req.compress_only {
            messages.push(
                "mode=compress-only (complete pack under --output; no external media assumed)"
                    .into(),
            );
        }
        if req.resume {
            messages.push(format!(
                "resume: skipped_done={skipped} need_build={redo} \
                 (fingerprint + size + archive_sha256; no legacy size-only)"
            ));
        }
        if let Some(wd) = &req.work_dir {
            messages.push(format!(
                "work_dir={} (private random subdir; default delete stage always{})",
                wd.display(),
                if req.keep_stage {
                    "; --keep-stage"
                } else {
                    ""
                }
            ));
        }
        // Per-unit plan lines only with -vv (capped).
        const PLAN_DETAIL_CAP: usize = 20;
        if req.verbose >= 2 {
            let n = plan_file.units.len();
            for rec in plan_file.units.iter().take(PLAN_DETAIL_CAP) {
                messages.push(format!(
                    "plan {} logical={} members={} status={:?} fp={}",
                    rec.archive_relpath,
                    rec.logical_bytes,
                    rec.members.len(),
                    rec.status,
                    &rec.fingerprint[..12.min(rec.fingerprint.len())]
                ));
            }
            if n > PLAN_DETAIL_CAP {
                messages.push(format!(
                    "plan: … and {} more unit(s) (use -vv; first {PLAN_DETAIL_CAP} only)",
                    n - PLAN_DETAIL_CAP
                ));
            }
        }

        let jobs = req.jobs.max(1);
        let transfer_queue = transfer_queue_depth(jobs, req.work_dir.is_some());
        if req.check_resources {
            check_resources(
                self.resources,
                &req.output,
                req.work_dir.as_deref(),
                &plan_file.units,
                jobs,
                transfer_queue,
                req.resource_safety_factor,
                req.force_resources,
                req.format,
                &mut messages,
            )?;
        }

        if req.dry_run {
            return Ok(PackResult {
                plan,
                dry_run: true,
                messages,
            });
        }

        self.fs.create_dir_all(&req.output)?;
        self.mapping.write_plan(&plan_path, &plan_file)?;
        messages.push(format!("plan_file={}", plan_path.display()));

        let stage_root: Option<PathBuf> = if let Some(wd) = &req.work_dir {
            let (stage, stage_msgs) = create_stage_dir(self.fs, wd, req.keep_stage)?;
            *leftover_stage = Some(stage.clone());
            messages.extend(stage_msgs);
            Some(stage)
        } else {
            None
        };

        let root_name = req
            .source
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "source".to_string());

        let mut prepared: Vec<PreparedUnit> = plan
            .units
            .iter()
            .zip(plan_file.units.iter())
            .filter(|(_, rec)| rec.status != PlanUnitStatus::Done)
            .map(|(unit, rec)| {
                let members: Vec<MemberSpec> = unit
                    .files
                    .iter()
                    .map(|f| MemberSpec {
                        source: req
                            .source
                            .join(f.rel_path.replace('/', std::path::MAIN_SEPARATOR_STR)),
                        member_name: f.rel_path.clone(),
                    })
                    .collect();
                PreparedUnit {
                    id: rec.id.clone(),
                    archive_relpath: rec.archive_relpath.clone(),
                    unit: unit.clone(),
                    members,
                    fingerprint: rec.fingerprint.clone(),
                }
            })
            .collect();
        prepared.sort_by_key(|p| p.unit.logical_bytes());

        let finalize_label = if req.compress_only {
            "local_finalize"
        } else if req.work_dir.is_some() {
            "USB_serial"
        } else {
            "rename_to_output"
        };
        messages.push(format!(
            "pipeline: compress_jobs={jobs} transfer_threads=1 queue={transfer_queue} \
             order=small_first finalize={finalize_label} verify_stage={} units_to_build={}",
            req.verify_stage,
            prepared.len()
        ));
        if req.verify_stage {
            messages.push(
                "verify_stage=on (pre-transfer member SHA-256 + post-transfer archive blob SHA-256)"
                    .into(),
            );
        }

        let units_total = plan_file.units.len();
        let outcome = run_pipeline(
            PipelineParams {
                prepared,
                plan_file,
                plan_path: &plan_path,
                output: &req.output,
                stage_root,
                format: req.format,
                jobs,
                transfer_queue,
                compress_only: req.compress_only,
                verify_stage: req.verify_stage,
                ratio_prior,
                log_transfers: req.verbose >= 2,
                logical_total,
                progress,
            },
            PipelinePorts {
                engine: self.engine,
                mapping: self.mapping,
                fs: self.fs,
                hasher: self.hasher,
            },
        )?;

        let final_plan = outcome.plan_file;
        for u in &final_plan.units {
            if u.status != PlanUnitStatus::Done {
                return Err(ArchiveError::Engine(format!(
                    "unit {} not completed",
                    u.archive_relpath
                )));
            }
        }

        progress.report(&ProgressSnapshot {
            units_done: units_total,
            units_total,
            logical_done: outcome.logical_done,
            logical_total,
            logical_transferred: outcome.logical_transferred,
            written_done: outcome.written_done,
            current: "archiver-map.json".into(),
            phase: "write-map",
            ratio_prior,
        });

        let archives: Vec<MappingArchive> = final_plan
            .units
            .iter()
            .map(|u| MappingArchive {
                id: u.id.clone(),
                archive_relpath: u.archive_relpath.clone(),
                logical_bytes: u.logical_bytes,
                members: u.members.clone(),
            })
            .collect();

        let manifest = MappingManifest::new(
            root_name,
            budget,
            req.format,
            final_plan.empty_directories.clone(),
            archives,
        );

        if let Some(parent) = req.mapping_path.parent() {
            if !parent.as_os_str().is_empty() {
                self.fs.create_dir_all(parent)?;
            }
        }
        self.mapping.write(&req.mapping_path, &manifest)?;
        self.mapping.write_plan(&plan_path, &final_plan)?;

        progress.report(&ProgressSnapshot {
            units_done: units_total,
            units_total,
            logical_done: outcome.logical_done,
            logical_total,
            logical_transferred: outcome.logical_transferred,
            written_done: outcome.written_done,
            current: req.mapping_path.display().to_string(),
            phase: "done",
            ratio_prior,
        });
        progress.finish();

        messages.extend(outcome.verbose_lines);
        messages.push(format!("mapping: {}", req.mapping_path.display()));
        let actual_ratio = if outcome.logical_done > 0 {
            outcome.written_done as f64 / outcome.logical_done as f64
        } else {
            0.0
        };
        messages.push(format!(
            "bytes: logical_in={} archive_out={} content_prior_est={} actual_ratio={:.3} \
             (content+algo prior was {:.3})",
            outcome.logical_done,
            outcome.written_done,
            content_est.estimated_archive_bytes,
            actual_ratio,
            ratio_prior,
        ));

        let elapsed = wall_start.elapsed().as_secs_f64();
        let ratio = if outcome.logical_done > 0 {
            outcome.written_done as f64 / outcome.logical_done as f64
        } else {
            0.0
        };
        messages.push(format!(
            "summary: ok units={units} empty_dirs={empty} format={fmt} budget={budget} \
             logical_in={lin} ({lin_h}) archive_out={aout} ({aout_h}) ratio={ratio:.3} \
             elapsed={elapsed:.1}s source={src} output={out} mapping={map} plan={plan}",
            units = units_total,
            empty = plan.empty_directories.len(),
            fmt = req.format.as_str(),
            budget = budget,
            lin = outcome.logical_done,
            lin_h = human_bytes(outcome.logical_done),
            aout = outcome.written_done,
            aout_h = human_bytes(outcome.written_done),
            ratio = ratio,
            elapsed = elapsed,
            src = req.source.display(),
            out = req.output.display(),
            map = req.mapping_path.display(),
            plan = plan_path.display(),
        ));

        Ok(PackResult {
            plan,
            dry_run: false,
            messages,
        })
    }
}
