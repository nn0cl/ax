use crate::core::domain::{
    assign_archive_paths, assign_volume_paths, budget_from_volume_count, estimate_disk_requirement,
    evaluate_resources, plan_pack_with_progress, ArchiveError, ArchiveFormat, MappingArchive,
    MappingManifest, MappingMember, MemberSpec, PackPlan, PackPlanFile, PackUnit, PlanUnitRecord,
    PlanUnitStatus, ResourceLevel, ResourcePlanInput, PLAN_FILE_NAME,
};
use crate::core::ports::{
    ArchiveEngine, ContentHasher, FileScanner, FileSystem, MappingStore, NullProgress,
    PackProgress, ProgressSnapshot, ResourceProbe,
};
use crate::core::interrupt;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Input for a media-fit pack operation.
#[derive(Debug, Clone)]
pub struct PackRequest {
    pub source: PathBuf,
    pub output: PathBuf,
    pub budget: u64,
    pub format: ArchiveFormat,
    pub mapping_path: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
    pub jobs: usize,
    pub work_dir: Option<PathBuf>,
    pub cleanup_work_dir: bool,
    /// Resume using existing `archiver-plan.json` under output; re-plan and skip done units.
    pub resume: bool,
    /// Check free disk (output/work-dir) and soft memory before writing.
    pub check_resources: bool,
    /// Continue even if resource check fails.
    pub force_resources: bool,
    /// Multiplier for disk estimates (1.0 ≈ need up to remaining logical on output).
    pub resource_safety_factor: f64,
    /// Complete local pack under `--output` (no external-media assumption).
    /// With `--work-dir`, still stages then finalizes to output (prefer rename, then copy).
    pub compress_only: bool,
    /// Place archives under `vol-001/` … (max this many volumes, capacity = budget).
    pub volumes: Option<u32>,
}

impl Default for PackRequest {
    fn default() -> Self {
        Self {
            source: PathBuf::new(),
            output: PathBuf::new(),
            budget: 0,
            format: ArchiveFormat::Zip,
            mapping_path: PathBuf::new(),
            dry_run: false,
            verbose: false,
            jobs: 1,
            work_dir: None,
            cleanup_work_dir: false,
            resume: false,
            check_resources: true,
            force_resources: false,
            resource_safety_factor: 1.0,
            compress_only: false,
            volumes: None,
        }
    }
}

#[derive(Debug)]
pub struct PackResult {
    pub plan: PackPlan,
    pub dry_run: bool,
    pub messages: Vec<String>,
}

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

    pub fn execute(&self, req: &PackRequest) -> Result<PackResult, ArchiveError> {
        self.execute_with_progress(req, &NullProgress)
    }

    pub fn execute_with_progress(
        &self,
        req: &PackRequest,
        progress: &dyn PackProgress,
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
        if req.cleanup_work_dir && req.work_dir.is_none() {
            return Err(ArchiveError::Engine(
                "--cleanup-work-dir requires --work-dir".into(),
            ));
        }

        let plan_path = req.output.join(PLAN_FILE_NAME);

        // Load previous plan for resume (structure-change handling via fingerprints).
        // First run: no plan is normal — fall through to a fresh pack.
        let mut messages = Vec::new();
        let previous: Option<PackPlanFile> = if req.resume {
            if self.fs.exists(&plan_path) {
                Some(self.mapping.read_plan(&plan_path)?)
            } else {
                messages.push(format!(
                    "resume: no plan at {} — starting fresh pack (first run is OK)",
                    plan_path.display()
                ));
                None
            }
        } else {
            None
        };

        progress.report(&ProgressSnapshot {
            units_done: 0,
            units_total: 0,
            logical_done: 0,
            logical_total: 0,
            written_done: 0,
            current: req.source.display().to_string(),
            phase: "scan",
        });

        let (files, empty_dirs) = self.scanner.scan_tree_with_progress(
            &req.source,
            &|sp| {
                progress.report(&ProgressSnapshot {
                    units_done: sp.files_found,
                    units_total: 0,
                    logical_done: sp.bytes_found,
                    logical_total: 0,
                    written_done: 0,
                    current: sp.current,
                    phase: "scan",
                });
            },
        )?;
        let file_count = files.len();
        let logical_total: u64 = files.iter().map(|f| f.size).sum();

        // Budget: explicit, or ceil(total / volumes) when only --volumes N.
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

        progress.report(&ProgressSnapshot {
            units_done: 0,
            units_total: 0,
            logical_done: logical_total,
            logical_total,
            written_done: 0,
            current: format!("planning {file_count} files…"),
            phase: "plan",
        });

        let plan = plan_pack_with_progress(files, empty_dirs, budget, &|dirs_done,
                                                                        dirs_total,
                                                                        units| {
            progress.report(&ProgressSnapshot {
                units_done: dirs_done,
                units_total: dirs_total,
                logical_done: logical_total,
                logical_total,
                written_done: 0,
                current: format!("{units} units so far"),
                phase: "plan",
            });
        });

        let mut archive_paths = assign_archive_paths(&plan.units, req.format);
        if let Some(n) = req.volumes {
            let sizes: Vec<u64> = plan.units.iter().map(|u| u.logical_bytes()).collect();
            archive_paths = assign_volume_paths(&archive_paths, &sizes, budget, n as usize)?;
            messages.push(format!(
                "volumes={n} capacity≈{budget} layout=vol-NNN/…"
            ));
        }

        // Build plan file records for current tree.
        let unit_refs: Vec<(String, String, &PackUnit)> = plan
            .units
            .iter()
            .zip(archive_paths.iter())
            .enumerate()
            .map(|(idx, (unit, rel))| (format!("a{:04}", idx + 1), rel.clone(), unit))
            .collect();

        let mut plan_file = PackPlanFile::from_planned_units(
            budget,
            req.format,
            plan.empty_directories.clone(),
            &unit_refs
                .iter()
                .map(|(id, rel, u)| (id.clone(), rel.clone(), *u))
                .collect::<Vec<_>>(),
        );

        // Match completed units from previous plan by fingerprint (survives path/placement change).
        let mut done_by_fp: HashMap<String, PlanUnitRecord> = HashMap::new();
        if let Some(prev) = &previous {
            for u in &prev.units {
                if u.status == PlanUnitStatus::Done {
                    done_by_fp.insert(u.fingerprint.clone(), u.clone());
                }
            }
        }

        let mut skipped = 0usize;
        let mut redo = 0usize;
        for rec in &mut plan_file.units {
            if let Some(prev) = done_by_fp.get(&rec.fingerprint) {
                let prev_path = req.output.join(&prev.archive_relpath);
                let ok = prev.archive_bytes.is_some()
                    && self.fs.exists(&prev_path)
                    && self.fs.file_len(&prev_path).ok() == prev.archive_bytes;
                if ok {
                    // Structure may have reassigned archive_relpath; relocate if needed.
                    let new_path = req.output.join(&rec.archive_relpath);
                    if prev.archive_relpath != rec.archive_relpath {
                        if let Some(parent) = new_path.parent() {
                            self.fs.create_dir_all(parent)?;
                        }
                        if !self.fs.exists(&new_path) {
                            self.fs.copy_file(&prev_path, &new_path)?;
                        }
                    }
                    rec.members = prev.members.clone();
                    rec.status = PlanUnitStatus::Done;
                    rec.archive_bytes = prev.archive_bytes;
                    skipped += 1;
                    continue;
                }
            }
            redo += 1;
        }

        messages.push(format!(
            "budget={} units={} empty_dirs={}",
            budget,
            plan.units.len(),
            plan.empty_directories.len()
        ));
        if req.compress_only {
            messages.push(
                "mode=compress-only (complete pack under --output; no external media assumed)"
                    .into(),
            );
        }
        if req.resume {
            if previous.is_some() {
                messages.push(format!(
                    "resume: skipped_done={skipped} need_build={redo} (fingerprint match; tree changes redo units)"
                ));
            }
        }
        if let Some(wd) = &req.work_dir {
            messages.push(format!(
                "work_dir={} cleanup={}",
                wd.display(),
                req.cleanup_work_dir
            ));
        }
        for rec in &plan_file.units {
            messages.push(format!(
                "plan {} logical={} members={} status={:?} fp={}",
                rec.archive_relpath,
                rec.logical_bytes,
                rec.members.len(),
                rec.status,
                &rec.fingerprint[..12.min(rec.fingerprint.len())]
            ));
        }

        // --- Resource check (own machine + destination) after plan is known ---
        let jobs = req.jobs.max(1);
        let transfer_queue = if req.work_dir.is_some() {
            (jobs * 4).max(8)
        } else {
            jobs.max(1)
        };
        let pending: Vec<&PlanUnitRecord> = plan_file
            .units
            .iter()
            .filter(|u| u.status != PlanUnitStatus::Done)
            .collect();
        let remaining_logical: u64 = pending.iter().map(|u| u.logical_bytes).sum();
        let max_unit = pending.iter().map(|u| u.logical_bytes).max().unwrap_or(0);
        let disk_need = estimate_disk_requirement(&ResourcePlanInput {
            remaining_logical_bytes: remaining_logical,
            max_unit_logical_bytes: max_unit,
            compress_jobs: jobs,
            transfer_queue,
            use_work_dir: req.work_dir.is_some(),
            safety_factor: req.resource_safety_factor,
        });

        if req.check_resources && !pending.is_empty() {
            let out_sp = self.resources.disk_space(&req.output)?;
            let work_sp = if let Some(wd) = &req.work_dir {
                Some(self.resources.disk_space(wd)?)
            } else {
                None
            };
            let mem = self.resources.available_memory_bytes();
            let findings = evaluate_resources(
                out_sp.free_bytes,
                work_sp.map(|s| s.free_bytes),
                mem,
                &disk_need,
                req.work_dir.is_some(),
                jobs,
            );
            for f in &findings {
                messages.push(format!("resource: {}", f.message));
            }
            let hard = findings
                .iter()
                .filter(|f| f.level == ResourceLevel::Fail)
                .map(|f| f.message.clone())
                .collect::<Vec<_>>();
            if !hard.is_empty() && !req.force_resources {
                return Err(ArchiveError::InsufficientResources(hard.join("; ")));
            }
            if !hard.is_empty() && req.force_resources {
                messages.push(
                    "resource: FORCED continue despite insufficient free space/memory".into(),
                );
            }
        }

        if req.dry_run {
            return Ok(PackResult {
                plan,
                dry_run: true,
                messages,
            });
        }

        self.fs.create_dir_all(&req.output)?;
        // Persist plan first so interrupt can --resume.
        self.mapping.write_plan(&plan_path, &plan_file)?;
        messages.push(format!("plan_file={}", plan_path.display()));

        let stage_root: Option<PathBuf> = if let Some(wd) = &req.work_dir {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let stage = wd
                .join(".archiver-work")
                .join(format!("{}-{}", std::process::id(), nanos));
            self.fs.create_dir_all(&stage)?;
            messages.push(format!("stage_root={}", stage.display()));
            Some(stage)
        } else {
            None
        };

        let root_name = req
            .source
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "source".to_string());

        // Units still needing build; **smallest first** so USB transfer can start early.
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
             order=small_first finalize={finalize_label} units_to_build={}",
            prepared.len()
        ));

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
        let verbose_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let plan_file_mu: Arc<Mutex<PackPlanFile>> = Arc::new(Mutex::new(plan_file));

        let report = |phase: &'static str, current: &str, units: usize| {
            progress.report(&ProgressSnapshot {
                units_done: units,
                units_total,
                logical_done: logical_done.load(Ordering::Relaxed),
                logical_total,
                written_done: written_done.load(Ordering::Relaxed),
                current: current.to_string(),
                phase,
            });
        };

        report("start", "", already_done);

        if !prepared.is_empty() {
            // Bounded queue between compress workers and the serial transfer thread.
            let (tx, rx) = sync_channel::<Result<StagedUnit, ArchiveError>>(transfer_queue);
            let transfer_has_stage = stage_root.is_some();
            let transfer_cleanup = req.cleanup_work_dir;
            let prefer_rename = req.compress_only || !transfer_has_stage;
            let transfer_output = &req.output;
            let transfer_plan_path = &plan_path;

            // thread::scope lets compress + transfer threads borrow progress / ports safely.
            thread::scope(|scope| -> Result<(), ArchiveError> {
                let transfer_units_done = Arc::clone(&units_done);
                let transfer_written = Arc::clone(&written_done);
                let transfer_logical = Arc::clone(&logical_done);
                let transfer_plan = Arc::clone(&plan_file_mu);
                let transfer_verbose = Arc::clone(&verbose_buf);
                let fs_xfer = self.fs;
                let mapping_xfer = self.mapping;

                let transfer_handle = scope.spawn(move || -> Result<(), ArchiveError> {
                    while let Ok(item) = rx.recv() {
                        interrupt::check()?;
                        let staged = item?;
                        let final_path = transfer_output.join(&staged.archive_relpath);

                        report_xfer(
                            progress,
                            "transfer",
                            &staged.archive_relpath,
                            transfer_units_done.load(Ordering::Relaxed),
                            units_total,
                            transfer_logical.load(Ordering::Relaxed),
                            logical_total,
                            transfer_written.load(Ordering::Relaxed),
                        );

                        if let Some(parent) = final_path.parent() {
                            fs_xfer.create_dir_all(parent)?;
                        }

                        let written = finalize_staged(
                            fs_xfer,
                            &staged.staged_path,
                            &final_path,
                            transfer_has_stage,
                            prefer_rename,
                            transfer_cleanup,
                        )?;

                        transfer_written.fetch_add(written, Ordering::Relaxed);
                        let n = transfer_units_done.fetch_add(1, Ordering::Relaxed) + 1;

                        if let Ok(mut pf) = transfer_plan.lock() {
                            if let Some(rec) = pf.units.iter_mut().find(|u| u.id == staged.id) {
                                rec.mark_done(staged.mapped_members.clone(), written);
                            }
                            let _ = mapping_xfer.write_plan(transfer_plan_path, &pf);
                        }

                        report_xfer(
                            progress,
                            "transfer",
                            &staged.archive_relpath,
                            n,
                            units_total,
                            transfer_logical.load(Ordering::Relaxed),
                            logical_total,
                            transfer_written.load(Ordering::Relaxed),
                        );

                        if let Ok(mut buf) = transfer_verbose.lock() {
                            buf.push(format!(
                                "transferred {n}/{units_total} {} ({written} bytes)",
                                staged.archive_relpath
                            ));
                        }
                    }
                    Ok(())
                });

                // Compress workers (separate threads via rayon): hash + compress only.
                let logical_done_c = Arc::clone(&logical_done);
                let units_done_c = Arc::clone(&units_done);
                let stage_for_compress = stage_root.as_ref();
                let output_for_partial = &req.output;

                let compress_result = if jobs == 1 {
                    for p in &prepared {
                        interrupt::check()?;
                        let staged = compress_one(
                            p,
                            self.engine,
                            self.hasher,
                            self.fs,
                            req.format,
                            stage_for_compress,
                            output_for_partial,
                            &logical_done_c,
                            &units_done_c,
                            &written_done,
                            units_total,
                            logical_total,
                            jobs,
                            progress,
                        );
                        tx.send(staged).map_err(|_| {
                            ArchiveError::Engine("transfer channel closed".into())
                        })?;
                    }
                    Ok(())
                } else {
                    let written_done_c = Arc::clone(&written_done);
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(jobs)
                        .build()
                        .map_err(|e| ArchiveError::Engine(e.to_string()))?;
                    pool.install(|| {
                        prepared.par_iter().try_for_each(|p| {
                            interrupt::check()?;
                            let staged = compress_one(
                                p,
                                self.engine,
                                self.hasher,
                                self.fs,
                                req.format,
                                stage_for_compress,
                                output_for_partial,
                                &logical_done_c,
                                &units_done_c,
                                &written_done_c,
                                units_total,
                                logical_total,
                                jobs,
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

        // Assemble final archives list from plan file (includes skipped).
        let final_plan = plan_file_mu
            .lock()
            .map_err(|e| ArchiveError::Engine(format!("plan lock: {e}")))?;

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
            logical_done: logical_done.load(Ordering::Relaxed),
            logical_total,
            written_done: written_done.load(Ordering::Relaxed),
            current: "archiver-map.json".into(),
            phase: "write-map",
        });

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

        if req.cleanup_work_dir {
            if let Some(stage) = &stage_root {
                let _ = self.fs.remove_dir_all(stage);
                messages.push(format!("cleaned stage {}", stage.display()));
            }
        }

        progress.report(&ProgressSnapshot {
            units_done: units_total,
            units_total,
            logical_done: logical_done.load(Ordering::Relaxed),
            logical_total,
            written_done: written_done.load(Ordering::Relaxed),
            current: req.mapping_path.display().to_string(),
            phase: "done",
        });
        progress.finish();

        if let Ok(mut buf) = verbose_buf.lock() {
            messages.append(&mut buf);
        }
        messages.push(format!("mapping: {}", req.mapping_path.display()));
        messages.push(format!(
            "bytes: logical_in={} archive_out={}",
            logical_done.load(Ordering::Relaxed),
            written_done.load(Ordering::Relaxed)
        ));

        Ok(PackResult {
            plan,
            dry_run: false,
            messages,
        })
    }
}

struct PreparedUnit {
    id: String,
    archive_relpath: String,
    unit: PackUnit,
    members: Vec<MemberSpec>,
    #[allow(dead_code)]
    fingerprint: String,
}

struct StagedUnit {
    id: String,
    archive_relpath: String,
    mapped_members: Vec<MappingMember>,
    staged_path: PathBuf,
}

fn report_xfer(
    progress: &dyn PackProgress,
    phase: &'static str,
    current: &str,
    units_done: usize,
    units_total: usize,
    logical_done: u64,
    logical_total: u64,
    written_done: u64,
) {
    progress.report(&ProgressSnapshot {
        units_done,
        units_total,
        logical_done,
        logical_total,
        written_done,
        current: current.to_string(),
        phase,
    });
}

/// Move staged archive into final path: prefer rename (same volume / compress-only), else copy.
fn finalize_staged<F: FileSystem>(
    fs: &F,
    staged: &Path,
    final_path: &Path,
    has_stage: bool,
    prefer_rename: bool,
    cleanup_stage: bool,
) -> Result<u64, ArchiveError> {
    if !has_stage {
        fs.rename(staged, final_path)?;
        return fs.file_len(final_path);
    }
    if prefer_rename {
        if fs.rename(staged, final_path).is_ok() {
            return fs.file_len(final_path);
        }
    }
    let n = fs.copy_file(staged, final_path)?;
    if cleanup_stage || prefer_rename {
        let _ = fs.remove_file(staged);
    }
    Ok(n)
}

fn compress_worker_id(jobs: usize) -> usize {
    rayon::current_thread_index()
        .unwrap_or(0)
        .min(jobs.saturating_sub(1))
}

fn compress_one<E, H, F>(
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
    units_total: usize,
    logical_total: u64,
    jobs: usize,
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
        written_done: written_done.load(Ordering::Relaxed),
        current: p.archive_relpath.clone(),
        phase,
    };

    progress.report_worker(wid, "hash", &p.archive_relpath, 0);
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
        progress.report_worker(wid, "hash", &p.archive_relpath, file.size);
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

    progress.report_worker(wid, "compress", &p.archive_relpath, 0);
    progress.report(&snap("compress"));
    engine.create_members(format, &staged_path, &p.members)?;
    progress.report_worker(wid, "idle", "", 0);

    Ok(StagedUnit {
        id: p.id.clone(),
        archive_relpath: p.archive_relpath.clone(),
        mapped_members,
        staged_path,
    })
}

pub fn default_mapping_path(output: &Path) -> PathBuf {
    output.join("archiver-map.json")
}


