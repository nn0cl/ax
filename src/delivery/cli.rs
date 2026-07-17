use crate::adapters::archive::CompositeArchiveEngine;
use crate::adapters::fs::{
    LocalFileScanner, LocalFileSystem, LocalMediaProbe, LocalResourceProbe, LocalThroughputMeter,
    Sha256Hasher,
};
use crate::adapters::mapping::JsonMappingStore;
use crate::core::application::{
    default_archives_root, default_mapping_path, default_verify_archives_root, CreateArchive,
    EstimateTransferUnit, EstimateUnitRequest, ExtractArchive, ListArchive, PackRequest,
    PackSourceTree, RestoreFromMapping, RestoreRequest, VerifyMapping, VerifyRequest,
};
use crate::core::domain::{
    effective_budget, format_estimate_report_with_meta, parse_size, ArchiveError, ArchiveFormat,
    ConnectionKind, MediaKind, DEFAULT_TARGET_SECS,
};
use crate::core::ports::NullProgress;
use crate::delivery::progress::SingleLineProgress;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "archiver",
    version,
    about = "Media-fit pack/restore with transfer-size estimation"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum CliFormat {
    Zip,
    Tar,
    #[value(name = "tar.gz")]
    TarGz,
    #[value(name = "tar.zst")]
    TarZst,
}

impl From<CliFormat> for ArchiveFormat {
    fn from(v: CliFormat) -> Self {
        match v {
            CliFormat::Zip => ArchiveFormat::Zip,
            CliFormat::Tar => ArchiveFormat::Tar,
            CliFormat::TarGz => ArchiveFormat::TarGz,
            CliFormat::TarZst => ArchiveFormat::TarZst,
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum CliMedia {
    Cd,
    Dvd,
    Bd25,
}

impl From<CliMedia> for MediaKind {
    fn from(v: CliMedia) -> Self {
        match v {
            CliMedia::Cd => MediaKind::Cd,
            CliMedia::Dvd => MediaKind::Dvd,
            CliMedia::Bd25 => MediaKind::Bd25,
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum CliConnection {
    Local,
    Usb2,
    Usb3,
    Thunderbolt,
    Network,
}

impl From<CliConnection> for ConnectionKind {
    fn from(v: CliConnection) -> Self {
        match v {
            CliConnection::Local => ConnectionKind::Local,
            CliConnection::Usb2 => ConnectionKind::Usb2,
            CliConnection::Usb3 => ConnectionKind::Usb3,
            CliConnection::Thunderbolt => ConnectionKind::Thunderbolt,
            CliConnection::Network => ConnectionKind::Network,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a single archive from files and directories
    Create {
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long)]
        format: Option<String>,
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
    },

    /// List members of an archive
    List {
        archive: PathBuf,
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Extract a single archive into a destination directory
    Extract {
        #[arg(short, long)]
        dest: PathBuf,
        archive: PathBuf,
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Estimate a fast-transfer pack unit size for a destination volume
    Estimate {
        /// Destination path (USB mount, network share, or local directory)
        #[arg(short, long)]
        path: PathBuf,

        /// Connection class (default: auto-detect)
        #[arg(long, value_enum)]
        connection: Option<CliConnection>,

        /// Target seconds to transfer one archive unit (default: 45)
        #[arg(long, default_value_t = DEFAULT_TARGET_SECS)]
        target_secs: u64,

        /// Override assumed write throughput, e.g. 80M (bytes/sec)
        #[arg(long)]
        throughput: Option<String>,

        /// Run a short sequential write sample on the destination
        #[arg(long)]
        measure: bool,

        /// Sample size for --measure (default: 64M)
        #[arg(long)]
        measure_sample: Option<String>,
    },

    /// Pack a source tree into media-sized archives with a JSON mapping
    Pack {
        #[arg(short, long)]
        source: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(long, conflicts_with_all = ["media", "auto_size"])]
        target_size: Option<String>,

        #[arg(long, conflicts_with_all = ["target_size", "auto_size"])]
        media: Option<CliMedia>,

        /// Derive unit size from destination free space + connection throughput
        #[arg(long, conflicts_with_all = ["target_size", "media"])]
        auto_size: bool,

        /// Split pack into up to N volume dirs (`vol-001/` …). Capacity = budget
        /// (or ceil(total/N) if used alone without --target-size/--media/--auto-size)
        #[arg(long)]
        volumes: Option<u32>,

        #[arg(long, value_enum)]
        connection: Option<CliConnection>,

        #[arg(long, default_value_t = DEFAULT_TARGET_SECS)]
        target_secs: u64,

        #[arg(long)]
        throughput: Option<String>,

        /// Measure write throughput when using --auto-size
        #[arg(long)]
        measure: bool,

        #[arg(short, long, value_enum, default_value_t = CliFormat::Zip)]
        format: CliFormat,

        #[arg(long)]
        mapping: Option<PathBuf>,

        /// Plan only; do not write archives or mapping
        #[arg(long)]
        dry_run: bool,

        /// Verbose: show per-compress-thread speed lines in the progress UI
        #[arg(long, short = 'v')]
        verbose: bool,

        /// Two-line live speed on stderr: compress + transfer (default: on when TTY)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        progress: bool,

        /// Parallel archive workers (default: 1)
        #[arg(long, default_value_t = 1)]
        jobs: usize,

        /// Local staging directory: compress here, then copy to --output (e.g. fast SSD)
        #[arg(long)]
        work_dir: Option<PathBuf>,

        /// After each archive is copied to --output, delete staged files; remove stage tree at end
        #[arg(long, requires = "work_dir")]
        cleanup_work_dir: bool,

        /// Resume from output/archiver-plan.json when present; if missing, start a fresh pack
        #[arg(long)]
        resume: bool,

        /// Check free disk on output/work-dir and soft memory before writing (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        check_resources: bool,

        /// Continue even if resource check reports insufficient free space
        #[arg(long)]
        force_resources: bool,

        /// Safety multiplier for disk estimates (default 1.0 = need up to remaining logical on output)
        #[arg(long, default_value_t = 1.0)]
        resource_safety_factor: f64,

        /// Complete local pack under --output (prefer rename from work-dir; no USB assumption)
        #[arg(long)]
        compress_only: bool,
    },

    /// Restore a tree from a JSON mapping and scattered archives
    Restore {
        #[arg(short, long)]
        mapping: PathBuf,

        #[arg(short, long)]
        dest: PathBuf,

        #[arg(long)]
        archives_root: Option<PathBuf>,

        /// Verify SHA-256 of restored files against the mapping (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        verify: bool,
    },

    /// Verify packed archives against a mapping (no restore to a user path)
    Verify {
        #[arg(short, long)]
        mapping: PathBuf,

        /// Root for archive_relpath (default: mapping file's directory)
        #[arg(long)]
        archives_root: Option<PathBuf>,

        /// Presence + member list only (skip extract and SHA-256)
        #[arg(long)]
        quick: bool,
    },
}

fn parse_format(raw: Option<String>) -> Result<Option<ArchiveFormat>, ArchiveError> {
    match raw {
        Some(s) => Ok(Some(ArchiveFormat::parse(&s)?)),
        None => Ok(None),
    }
}

fn parse_throughput(raw: Option<String>) -> Result<Option<u64>, ArchiveError> {
    match raw {
        Some(s) => Ok(Some(parse_size(&s)?)),
        None => Ok(None),
    }
}

/// Budget for pack: explicit flags, or 0 when only `--volumes` (filled after scan).
fn resolve_pack_budget(
    target_size: Option<String>,
    media: Option<CliMedia>,
    auto_size: bool,
    volumes: Option<u32>,
    connection: Option<CliConnection>,
) -> Result<Option<u64>, ArchiveError> {
    if auto_size {
        return Ok(None); // caller handles
    }
    match (target_size, media, volumes) {
        (Some(s), None, _) => {
            let conn = connection
                .map(ConnectionKind::from)
                .unwrap_or(ConnectionKind::Local);
            let bytes = parse_size(&s)?;
            Ok(Some(effective_budget(MediaKind::Custom(bytes), conn)))
        }
        (None, Some(m), _) => {
            let conn = connection
                .map(ConnectionKind::from)
                .unwrap_or(ConnectionKind::Local);
            Ok(Some(effective_budget(MediaKind::from(m), conn)))
        }
        (None, None, Some(_)) => Ok(Some(0)), // pack derives after scan
        _ => Err(ArchiveError::InvalidBudget),
    }
}

/// Run the CLI and return a process exit code.
pub fn run(cli: Cli) -> ExitCode {
    match run_inner(cli) {
        Ok(msg) => {
            if !msg.is_empty() {
                print!("{msg}");
                if !msg.ends_with('\n') {
                    println!();
                }
            }
            ExitCode::SUCCESS
        }
        Err(ArchiveError::Interrupted) => {
            eprintln!("error: {err}", err = ArchiveError::Interrupted);
            // 128 + SIGINT(2) = 130, conventional for Ctrl+C.
            ExitCode::from(130)
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run_inner(cli: Cli) -> Result<String, ArchiveError> {
    let engine = CompositeArchiveEngine::new();
    let scanner = LocalFileScanner::new();
    let mapping_store = JsonMappingStore::new();
    let media_probe = LocalMediaProbe::new();
    let meter = LocalThroughputMeter::new();
    let fs = LocalFileSystem::new();
    let hasher = Sha256Hasher::new();

    match cli.command {
        Commands::Create {
            output,
            format,
            inputs,
        } => {
            let format = parse_format(format)?;
            let used = CreateArchive::new(&engine, &fs).execute(&output, &inputs, format)?;
            Ok(format!(
                "created {} ({})\n",
                output.display(),
                used.as_str()
            ))
        }
        Commands::List { archive, format } => {
            let format = parse_format(format)?;
            let members = ListArchive::new(&engine).execute(&archive, format)?;
            let mut out = String::new();
            for m in members {
                if m.is_dir {
                    out.push_str(&format!("{}/\n", m.path));
                } else {
                    out.push_str(&format!("{}\n", m.path));
                }
            }
            Ok(out)
        }
        Commands::Extract {
            dest,
            archive,
            format,
        } => {
            let format = parse_format(format)?;
            ExtractArchive::new(&engine).execute(&archive, &dest, format)?;
            Ok(format!("extracted to {}\n", dest.display()))
        }
        Commands::Estimate {
            path,
            connection,
            target_secs,
            throughput,
            measure,
            measure_sample,
        } => {
            let sample = match measure_sample {
                Some(s) => parse_size(&s)?,
                None => 64 * 1024 * 1024,
            };
            let req = EstimateUnitRequest {
                path: path.clone(),
                connection_hint: connection.map(ConnectionKind::from),
                throughput_override: parse_throughput(throughput)?,
                target_transfer_secs: target_secs,
                measure,
                measure_sample_bytes: sample,
            };
            let out = EstimateTransferUnit::new(&media_probe, &meter).execute(&req)?;
            if measure && !out.measured {
                eprintln!("warning: write measurement failed; using table/default throughput");
            }
            let mut report = format_estimate_report_with_meta(
                &out.estimate,
                Some(&out.detection),
                Some(out.free_bytes),
                Some(out.measured),
            );
            report.push('\n');
            report.push_str(&format!("path: {}\n", path.display()));
            Ok(report)
        }
        Commands::Pack {
            source,
            output,
            target_size,
            media,
            auto_size,
            volumes,
            connection,
            target_secs,
            throughput,
            measure,
            format,
            mapping,
            dry_run,
            verbose,
            progress,
            jobs,
            work_dir,
            cleanup_work_dir,
            resume,
            check_resources,
            force_resources,
            resource_safety_factor,
            compress_only,
        } => {
            let line_progress =
                SingleLineProgress::new(progress && !dry_run, verbose, jobs.max(1));
            let resources = LocalResourceProbe::new();
            if progress && !dry_run && !line_progress.is_enabled() {
                eprintln!(
                    "warning: progress UI disabled (no controlling terminal); \
                     try running without redirect, or check /dev/tty"
                );
            }

            let budget = if auto_size {
                if measure {
                    line_progress.status_message(
                        "measuring write speed on destination (this can take ~1 min)...",
                    );
                } else {
                    line_progress.status_message("estimating pack unit size...");
                }
                let req = EstimateUnitRequest {
                    path: output.clone(),
                    connection_hint: connection.map(ConnectionKind::from),
                    throughput_override: parse_throughput(throughput)?,
                    target_transfer_secs: target_secs,
                    measure,
                    measure_sample_bytes: 64 * 1024 * 1024,
                };
                let out = EstimateTransferUnit::new(&media_probe, &meter).execute(&req)?;
                if measure && !out.measured {
                    eprintln!("warning: write measurement failed; using table/default throughput");
                }
                line_progress.status_message(&format!(
                    "budget {} bytes ({})",
                    out.estimate.recommended_unit_bytes,
                    crate::core::domain::human_bytes(out.estimate.recommended_unit_bytes)
                ));
                out.estimate.recommended_unit_bytes
            } else {
                resolve_pack_budget(target_size, media, auto_size, volumes, connection)?
                    .unwrap_or(0)
            };

            let mapping_path = mapping.unwrap_or_else(|| default_mapping_path(&output));
            line_progress.status_message(&format!(
                "scanning source {} ...",
                source.display()
            ));

            if compress_only {
                line_progress.status_message(
                    "mode=compress-only (final pack under --output)",
                );
            }
            if let Some(n) = volumes {
                line_progress.status_message(&format!("volumes={n} (vol-001/ …)"));
            }
            if let Some(ref wd) = work_dir {
                let how = if compress_only {
                    "compress local → finalize to output (prefer rename)"
                } else {
                    "compress local → copy to output"
                };
                line_progress.status_message(&format!(
                    "work_dir={} ({how}) cleanup={}",
                    wd.display(),
                    cleanup_work_dir
                ));
            }

            let req = PackRequest {
                source,
                output: output.clone(),
                budget,
                format: format.into(),
                mapping_path: mapping_path.clone(),
                dry_run,
                verbose,
                jobs,
                work_dir,
                cleanup_work_dir,
                resume,
                check_resources,
                force_resources,
                resource_safety_factor,
                compress_only,
                volumes,
            };

            let packer =
                PackSourceTree::new(&engine, &scanner, &mapping_store, &fs, &hasher, &resources);
            let result = if line_progress.is_enabled() {
                packer.execute_with_progress(&req, &line_progress)?
            } else {
                packer.execute_with_progress(&req, &NullProgress)?
            };

            // Multi-line plan only for dry-run or explicit verbose (avoid log flood).
            if dry_run {
                for line in &result.messages {
                    println!("{line}");
                }
            } else if verbose {
                for line in &result.messages {
                    eprintln!("{line}");
                }
            }

            if dry_run {
                Ok(format!(
                    "dry-run: {} unit(s), budget {} bytes (nothing written)\n",
                    result.plan.units.len(),
                    budget
                ))
            } else {
                let mut out = format!(
                    "packed {} archive(s), {} empty dir(s), budget {} bytes\nmapping: {}\n",
                    result.plan.units.len(),
                    result.plan.empty_directories.len(),
                    budget,
                    mapping_path.display()
                );
                for line in &result.messages {
                    if line.starts_with("resume:")
                        || line.starts_with("plan_file=")
                        || line.starts_with("work_dir=")
                        || line.starts_with("stage_root=")
                        || line.starts_with("resource:")
                        || line.starts_with("pipeline:")
                        || line.starts_with("mode=")
                        || line.starts_with("volumes=")
                        || line.starts_with("budget_from_volumes:")
                    {
                        out.push_str(line);
                        out.push('\n');
                    }
                }
                Ok(out)
            }
        }
        Commands::Restore {
            mapping,
            dest,
            archives_root,
            verify,
        } => {
            let archives_root = archives_root.unwrap_or_else(|| default_archives_root(&mapping));
            let req = RestoreRequest {
                mapping_path: mapping,
                dest: dest.clone(),
                archives_root,
                verify,
            };
            RestoreFromMapping::new(&engine, &mapping_store, &fs, &hasher).execute(&req)?;
            Ok(format!(
                "restored to {} (verify={})\n",
                dest.display(),
                verify
            ))
        }
        Commands::Verify {
            mapping,
            archives_root,
            quick,
        } => {
            let archives_root =
                archives_root.unwrap_or_else(|| default_verify_archives_root(&mapping));
            let req = VerifyRequest {
                mapping_path: mapping,
                archives_root,
                quick,
            };
            let report =
                VerifyMapping::new(&engine, &mapping_store, &fs, &hasher).execute(&req)?;
            for line in &report.messages {
                eprintln!("{line}");
            }
            for issue in &report.issues {
                eprintln!("issue: {issue}");
            }
            if report.is_ok() {
                Ok(format!("{}\n", report.summary_line()))
            } else {
                Err(ArchiveError::VerifyFailed(report.summary_line()))
            }
        }
    }
}
