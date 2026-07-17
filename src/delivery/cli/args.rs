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

        #[arg(short, long, value_enum, default_value_t = CliFormat::TarZst)]
        format: CliFormat,

        #[arg(long)]
        mapping: Option<PathBuf>,

        /// Plan only; do not write archives or mapping
        #[arg(long)]
        dry_run: bool,

        /// Verbose: show per-compress-thread speed lines in the progress UI
        #[arg(long, short = 'v')]
        verbose: bool,

        /// Live progress UI (default: on when TTY)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        progress: bool,

        /// Parallel compress workers (default: CPU count, max 8)
        #[arg(long, default_value_t = default_jobs())]
        jobs: usize,

        /// Base dir for staging: creates a private random subdir, never reuses siblings.
        /// Stage is deleted by default after pack (success or failure).
        #[arg(long)]
        work_dir: Option<PathBuf>,

        /// Keep the random stage directory after pack (debug). Default: always delete.
        #[arg(long, requires = "work_dir")]
        keep_stage: bool,

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

        /// Live progress on TTY (default: on when TTY)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        progress: bool,
    },
}

