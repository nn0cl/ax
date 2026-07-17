//! Clap CLI surface (command definitions only).
//!
//! Short binary name `ax` and 2-letter subcommands; long names remain as
//! visible aliases (`pa` ≡ `pack`, etc.).

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

Binary: ax (Archive eXchange). Default build output: target/*/ax

Commands (short = long alias):
  cr=create  ex=extract  ls=list  es=estimate  pa=pack  re=restore  ve=verify

Globals:
  -c, --config PATH     settings (.inf/.ini); CLI overrides file
  -s, --size SIZE|PRESET  pack budget (also --target-size on pa)
  -o, --output PATH     pack out / create archive / mapping JSON

Size presets (-s):
  c2      10 MiB   small units, resume-friendly (alias: low-net)
  fat32   3.8 GiB  under FAT32 4 GiB file limit
  flash   1 GiB    typical USB stick balance
  max     ~no split  large single-archive oriented
  or numeric: 10M, 700M, 4G, …

Examples:
  ax pa -i ./data -o ./out -s c2 -f tar.zst
  ax pa -i ./data -o ./out -s fat32
  ax es -p /Volumes/USB
  ax -o backup.zip cr ./src
  ax re -m ./out/archiver-map.json -d ./restored
  ax --help
  ax pa --help
";

#[derive(Debug, Parser)]
#[command(
    name = "ax",
    version,
    author = "nn0cl <nn0cl@dstechnology.co.jp>",
    about = "Local-first media-fit archiver — ax (Archive eXchange)",
    long_about = "Local-first media-fit archiver (Archive eXchange).\n\n\
Pack a source tree into budget-sized archives with JSON mapping, resume, \
verify, and restore. Also single-archive create / list / extract.\n\n\
Default binary name after `cargo build`: ax\n\
Short subcommands: cr ex ls es pa re ve (long aliases still work).\n\
Globals: -c config, -s size|preset, -o output.\n\
Size presets: c2, fat32, flash, max (or 10M / 4G).\n\n\
Copyright (c) nn0cl@dstechnology.co.jp. All rights reserved.\n\n\
NO WARRANTY — provided AS IS; use at your own risk. No SLA.",
    after_help = AFTER_HELP,
    propagate_version = true,
    disable_help_subcommand = false,
)]
pub struct Cli {
    /// Settings file (`.inf` / `.ini` / key=value). Section `[pack]` for pack.
    /// CLI flags override file values when both are set.
    #[arg(short = 'c', long = "config", global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Pack budget: numeric size (`10M`, `4G`) or preset
    /// `c2` | `fat32` | `flash` | `max` (see --help).
    #[arg(short = 's', long = "size", global = true, value_name = "SIZE|PRESET")]
    pub size: Option<String>,

    /// Output directory, archive path, or mapping JSON path (command-dependent).
    #[arg(short = 'o', long = "output", global = true, value_name = "PATH")]
    pub output: Option<PathBuf>,

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
    /// Create a single archive
    #[command(name = "cr", visible_alias = "create")]
    Create {
        #[arg(short = 'f', long)]
        format: Option<String>,
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
    },

    /// Extract a single archive
    #[command(name = "ex", visible_alias = "extract")]
    Extract {
        /// Destination directory
        #[arg(short = 'd', long)]
        dest: PathBuf,
        archive: PathBuf,
        #[arg(short = 'f', long)]
        format: Option<String>,
    },

    /// List members of an archive
    #[command(name = "ls", visible_alias = "list")]
    List {
        archive: PathBuf,
        #[arg(short = 'f', long)]
        format: Option<String>,
    },

    /// Estimate fast-transfer pack unit size
    #[command(name = "es", visible_alias = "estimate")]
    Estimate {
        /// Destination path (USB mount, network share, or local directory)
        #[arg(short = 'p', long)]
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

    /// Pack a source tree into media-sized archives
    #[command(name = "pa", visible_alias = "pack")]
    Pack {
        /// Source directory (required unless set in config `[pack] source=`)
        #[arg(short = 'i', long = "source")]
        source: Option<PathBuf>,

        /// Output root (or global `-o` / config `output=`)
        #[arg(long = "out")]
        pack_output: Option<PathBuf>,

        /// Budget size or preset (or global `-s`). Same tokens as `--size`.
        #[arg(long = "target-size", conflicts_with_all = ["media", "auto_size"])]
        target_size: Option<String>,

        #[arg(long, conflicts_with_all = ["target_size", "auto_size"])]
        media: Option<CliMedia>,

        /// Derive unit size from destination free space + connection throughput
        #[arg(long, conflicts_with_all = ["target_size", "media"], action = clap::ArgAction::SetTrue)]
        auto_size: bool,

        /// Split pack into up to N volume dirs (`vol-001/` …)
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
        #[arg(short = 'f', long, value_enum)]
        format: Option<CliFormat>,

        #[arg(long)]
        mapping: Option<PathBuf>,

        /// Plan only; do not write archives or mapping
        #[arg(long, action = clap::ArgAction::SetTrue)]
        dry_run: bool,

        /// Verbosity: `-v` light notes, `-vv` worker lines + per-unit plan/transfer log
        #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
        verbose: u8,

        /// Live progress UI (default: on when TTY)
        #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true")]
        progress: Option<bool>,

        /// Parallel compress workers (default: CPU count max 8)
        #[arg(long, short = 'j')]
        jobs: Option<usize>,

        /// Base dir for staging (private random subdir)
        #[arg(long)]
        work_dir: Option<PathBuf>,

        /// Keep the random stage directory after pack (debug)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        keep_stage: bool,

        /// Resume from output/archiver-plan.json
        #[arg(long, action = clap::ArgAction::SetTrue)]
        resume: bool,

        /// Check free disk / soft memory before writing (default: true)
        #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true")]
        check_resources: Option<bool>,

        /// Continue even if resource check fails
        #[arg(long, action = clap::ArgAction::SetTrue)]
        force_resources: bool,

        /// Safety multiplier for disk estimates
        #[arg(long)]
        resource_safety_factor: Option<f64>,

        /// Complete local pack under output (no USB assumption)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        compress_only: bool,

        /// Pre/post transfer integrity (member + archive SHA)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        verify_stage: bool,
    },

    /// Restore a tree from a JSON mapping
    #[command(name = "re", visible_alias = "restore")]
    Restore {
        /// Mapping JSON (or global `-o` if omitted)
        #[arg(short = 'm', long)]
        mapping: Option<PathBuf>,

        #[arg(short = 'd', long)]
        dest: PathBuf,

        #[arg(long)]
        archives_root: Option<PathBuf>,

        /// Verify SHA-256 of restored files (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        verify: bool,
    },

    /// Verify packed archives against a mapping
    #[command(name = "ve", visible_alias = "verify")]
    Verify {
        /// Mapping JSON (or global `-o` if omitted)
        #[arg(short = 'm', long)]
        mapping: Option<PathBuf>,

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
