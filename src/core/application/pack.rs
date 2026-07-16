use crate::core::domain::{
    assign_archive_paths, plan_pack, ArchiveError, ArchiveFormat, MappingArchive, MappingManifest,
    MappingMember, MemberSpec, PackPlan, PackUnit,
};
use crate::core::ports::{
    ArchiveEngine, ContentHasher, FileScanner, FileSystem, MappingStore,
};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Input for a media-fit pack operation.
#[derive(Debug, Clone)]
pub struct PackRequest {
    pub source: PathBuf,
    pub output: PathBuf,
    pub budget: u64,
    pub format: ArchiveFormat,
    pub mapping_path: PathBuf,
    /// When true, only plan; do not write archives or mapping.
    pub dry_run: bool,
    /// Log progress lines via the provided sink when true (delivery prints).
    pub verbose: bool,
    /// Parallel unit creators (1 = sequential).
    pub jobs: usize,
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
        }
    }
}

/// Result of pack including plan and optional human progress lines.
#[derive(Debug)]
pub struct PackResult {
    pub plan: PackPlan,
    pub dry_run: bool,
    /// Progress / plan summary lines for the CLI to print.
    pub messages: Vec<String>,
}

/// Use case: pack a source tree into sized, scattered archives + JSON mapping.
pub struct PackSourceTree<'a, E, S, M, F, H>
where
    E: ArchiveEngine,
    S: FileScanner,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
{
    engine: &'a E,
    scanner: &'a S,
    mapping: &'a M,
    fs: &'a F,
    hasher: &'a H,
}

impl<'a, E, S, M, F, H> PackSourceTree<'a, E, S, M, F, H>
where
    E: ArchiveEngine,
    S: FileScanner,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
{
    pub fn new(
        engine: &'a E,
        scanner: &'a S,
        mapping: &'a M,
        fs: &'a F,
        hasher: &'a H,
    ) -> Self {
        Self {
            engine,
            scanner,
            mapping,
            fs,
            hasher,
        }
    }

    pub fn execute(&self, req: &PackRequest) -> Result<PackResult, ArchiveError> {
        if req.budget == 0 {
            return Err(ArchiveError::InvalidBudget);
        }
        if !self.fs.is_dir(&req.source) {
            return Err(ArchiveError::SourceNotDirectory(
                req.source.display().to_string(),
            ));
        }

        let (files, empty_dirs) = self.scanner.scan_tree(&req.source)?;
        let plan = plan_pack(files, empty_dirs, req.budget);
        let archive_paths = assign_archive_paths(&plan.units, req.format);

        let mut messages = Vec::new();
        messages.push(format!(
            "budget={} units={} empty_dirs={}",
            req.budget,
            plan.units.len(),
            plan.empty_directories.len()
        ));

        for (unit, path) in plan.units.iter().zip(archive_paths.iter()) {
            messages.push(format!(
                "plan {} logical={} members={}",
                path,
                unit.logical_bytes(),
                unit.files.len()
            ));
        }

        if req.dry_run {
            return Ok(PackResult {
                plan,
                dry_run: true,
                messages,
            });
        }

        self.fs.create_dir_all(&req.output)?;

        let root_name = req
            .source
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "source".to_string());

        let jobs = req.jobs.max(1);
        let prepared: Vec<PreparedUnit> = plan
            .units
            .iter()
            .zip(archive_paths.iter())
            .enumerate()
            .map(|(idx, (unit, archive_rel))| {
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
                    id: format!("a{:04}", idx + 1),
                    archive_relpath: archive_rel.clone(),
                    unit: unit.clone(),
                    members,
                }
            })
            .collect();

        let progress = AtomicUsize::new(0);
        let total = prepared.len();
        let verbose_lines = Mutex::new(Vec::<String>::new());

        let build_one = |p: &PreparedUnit| -> Result<MappingArchive, ArchiveError> {
            let out_path = req.output.join(&p.archive_relpath);
            if let Some(parent) = out_path.parent() {
                self.fs.create_dir_all(parent)?;
            }

            let mut mapped_members = Vec::with_capacity(p.unit.files.len());
            for (file, member) in p.unit.files.iter().zip(p.members.iter()) {
                let digest = self.hasher.hash_file(&member.source)?;
                mapped_members.push(MappingMember {
                    source_relpath: file.rel_path.clone(),
                    size: file.size,
                    sha256: digest,
                });
            }

            self.engine
                .create_members(req.format, &out_path, &p.members)?;

            if req.verbose {
                let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
                if let Ok(mut lines) = verbose_lines.lock() {
                    lines.push(format!(
                        "created {}/{} {}",
                        n, total, p.archive_relpath
                    ));
                }
            }

            Ok(MappingArchive {
                id: p.id.clone(),
                archive_relpath: p.archive_relpath.clone(),
                logical_bytes: p.unit.logical_bytes(),
                members: mapped_members,
            })
        };

        let archives: Vec<MappingArchive> = if jobs == 1 {
            prepared
                .iter()
                .map(build_one)
                .collect::<Result<Vec<_>, _>>()?
        } else {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(jobs)
                .build()
                .map_err(|e| ArchiveError::Engine(e.to_string()))?;
            pool.install(|| {
                prepared
                    .par_iter()
                    .map(build_one)
                    .collect::<Result<Vec<_>, _>>()
            })?
        };

        // Keep archive order stable by preassigned id order (par_iter preserves order).
        let manifest = MappingManifest::new(
            root_name,
            req.budget,
            req.format,
            plan.empty_directories.clone(),
            archives,
        );

        if let Some(parent) = req.mapping_path.parent() {
            if !parent.as_os_str().is_empty() {
                self.fs.create_dir_all(parent)?;
            }
        }
        self.mapping.write(&req.mapping_path, &manifest)?;

        if let Ok(mut lines) = verbose_lines.lock() {
            messages.append(&mut lines);
        }
        messages.push(format!("mapping: {}", req.mapping_path.display()));

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
}

/// Default mapping path under the output directory.
pub fn default_mapping_path(output: &Path) -> PathBuf {
    output.join("archiver-map.json")
}
