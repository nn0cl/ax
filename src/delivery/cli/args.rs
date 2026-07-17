//! Clap CLI surface (command definitions only).

use crate::core::domain::{
    ArchiveFormat, ConnectionKind, MediaKind, DEFAULT_TARGET_SECS,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Copyright (c) nn0cl@dstechnology.co.jp. All rights reserved.
Author: nn0cl <nn0cl@dstechnology.co.jp>

DISCLAIMER: This software is provided AS IS, without warranty of any kind,
express or implied. Automated tests exist; they do not constitute a guarantee.
Use at your own risk. No SLA.
";

#[derive(Debug, Parser)]
#[command(
    name = "archiver",
    version,
    author = "nn0cl <nn0cl@dstechnology.co.jp>",
    about = "Media-fit pack/restore archiver (USB / optical unit sizing)",
    long_about = "Local-first media-fit archiver: pack a source tree into budget-sized \
archives with JSON mapping, resume, verify, and restore.\n\n\
Copyright (c) nn0cl@dstechnology.co.jp. All rights reserved.\n\n\
NO WARRANTY — provided AS IS; use at your own risk. No SLA.",
    after_help = AFTER_HELP,
    propagate_version = true
)]
pub struct Cli {
    /// Settings file (`.inf` / `.ini` / key=value). Section `[pack]` for pack.
    /// CLI flags override file values when both are set.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Default parallel compress jobs: min(8, available_parallelism).
pub fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().clamp(1, 8))
        .unwrap_or(1)
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
        /// Source directory (required unless set in --config `[pack] source=`)
        #[arg(short, long)]
        source: Option<PathBuf>,

        /// Output root (required unless set in --config `[pack] output=`)
        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long, conflicts_with_all = ["media", "auto_size"])]
        target_size: Option<String>,

        #[arg(long, conflicts_with_all = ["target_size", "auto_size"])]
        media: Option<CliMedia>,

        /// Derive unit size from destination free space + connection throughput
        #[arg(long, conflicts_with_all = ["target_size", "media"], action = clap::ArgAction::SetTrue)]
        auto_size: bool,

        /// Split pack into up to N volume dirs (`vol-001/` …). Capacity = budget
        /// (or ceil(total/N) if used alone without --target-size/--media/--auto-size)
        #[arg(long)]
        volumes: Option<u32>,

        #[arg(long, value_enum)]
        connection: Option<CliConnection>,

        #[arg(long)]
        target_secs: Option<u64>,

        #[arg(long)]
        throughput: Option<String>,

        /// Measure write throughput when using --auto-size
        #[arg(long, action = clap::ArgAction::SetTrue)]
        measure: bool,

        /// Archive format (default: tar.zst, or config `format=`)
        #[arg(short, long, value_enum)]
        format: Option<CliFormat>,

        #[arg(long)]
        mapping: Option<PathBuf>,

        /// Plan only; do not write archives or mapping
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,

        /// Verbose: show per-compress-thread speed lines in the progress UI
        #[arg(long, short = 'v', action = clap::ArgAction::SetTrue)]
        verbose: bool,

        /// Live progress UI (default: on when TTY). Config: `progress=true|false`
        #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true")]
        progress: Option<bool>,

        /// Parallel compress workers (default: CPU count max 8, or config `jobs=`)
        #[arg(long)]
        jobs: Option<usize>,

        /// Base dir for staging: creates a private random subdir, never reuses siblings.
        /// Stage is deleted by default after pack (success or failure).
        #[arg(long)]
        work_dir: Option<PathBuf>,

        /// Keep the random stage directory after pack (debug). Default: always delete.
        #[arg(long, action = clap::ArgAction::SetTrue)]
        keep_stage: bool,

        /// Resume from output/archiver-plan.json when present; if missing, start a fresh pack
        #[arg(long, action = clap::ArgAction::SetTrue)]
        resume: bool,

        /// Check free disk on output/work-dir and soft memory before writing (default: true)
        #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true")]
        check_resources: Option<bool>,

        /// Continue even if resource check reports insufficient free space
        #[arg(long, action = clap::ArgAction::SetTrue)]
        force_resources: bool,

        /// Safety multiplier for disk estimates (default 1.0 = need up to remaining logical on output)
        #[arg(long)]
        resource_safety_factor: Option<f64>,

        /// Complete local pack under --output (prefer rename from work-dir; no USB assumption)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        compress_only: bool,

        /// Pre-transfer: re-extract stage archive and match member SHA-256;
        /// post-transfer: dest archive SHA-256 must equal stage archive SHA-256.
        /// Config: `verify_stage=true`. Default: off (extra CPU/disk I/O).
        #[arg(long, action = clap::ArgAction::SetTrue)]
        verify_stage: bool,
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

        /// Live progress on TTY (default: on when TTY)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        progress: bool,
    },
}

