use crate::adapters::archive::CompositeArchiveEngine;
use crate::adapters::fs::{
    LocalFileScanner, LocalFileSystem, LocalMediaProbe, LocalThroughputMeter, Sha256Hasher,
};
use crate::adapters::mapping::JsonMappingStore;
use crate::core::application::{
    default_archives_root, default_mapping_path, CreateArchive, EstimateTransferUnit,
    EstimateUnitRequest, ExtractArchive, ListArchive, PackRequest, PackSourceTree,
    RestoreFromMapping, RestoreRequest,
};
use crate::core::domain::{
    effective_budget, format_estimate_report_with_meta, parse_size, ArchiveError, ArchiveFormat,
    ConnectionKind, MediaKind, DEFAULT_TARGET_SECS,
};
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

        /// Log per-archive progress to stderr
        #[arg(long, short = 'v')]
        verbose: bool,

        /// Parallel archive workers (default: 1)
        #[arg(long, default_value_t = 1)]
        jobs: usize,
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

fn resolve_budget_manual(
    target_size: Option<String>,
    media: Option<CliMedia>,
    connection: ConnectionKind,
) -> Result<u64, ArchiveError> {
    match (target_size, media) {
        (Some(s), None) => {
            let bytes = parse_size(&s)?;
            Ok(effective_budget(MediaKind::Custom(bytes), connection))
        }
        (None, Some(m)) => Ok(effective_budget(MediaKind::from(m), connection)),
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
            connection,
            target_secs,
            throughput,
            measure,
            format,
            mapping,
            dry_run,
            verbose,
            jobs,
        } => {
            let budget = if auto_size {
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
                out.estimate.recommended_unit_bytes
            } else {
                let conn = connection
                    .map(ConnectionKind::from)
                    .unwrap_or(ConnectionKind::Local);
                resolve_budget_manual(target_size, media, conn)?
            };

            let mapping_path = mapping.unwrap_or_else(|| default_mapping_path(&output));
            let req = PackRequest {
                source,
                output: output.clone(),
                budget,
                format: format.into(),
                mapping_path: mapping_path.clone(),
                dry_run,
                verbose,
                jobs,
            };
            let result =
                PackSourceTree::new(&engine, &scanner, &mapping_store, &fs, &hasher).execute(&req)?;

            if verbose || dry_run {
                for line in &result.messages {
                    if dry_run {
                        println!("{line}");
                    } else {
                        eprintln!("{line}");
                    }
                }
            }

            if dry_run {
                Ok(format!(
                    "dry-run: {} unit(s), budget {} bytes (nothing written)\n",
                    result.plan.units.len(),
                    budget
                ))
            } else {
                Ok(format!(
                    "packed {} archive(s), {} empty dir(s), budget {} bytes\nmapping: {}\n",
                    result.plan.units.len(),
                    result.plan.empty_directories.len(),
                    budget,
                    mapping_path.display()
                ))
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
    }
}
