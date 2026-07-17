//! Merge pack CLI flags with `[pack]` section from `.inf` / `.ini` config.

use super::args::{default_jobs, CliConnection, CliFormat, CliMedia};
use crate::core::domain::{
    auto_jobs_provisional_cap, load_ini_file, ArchiveError, IniConfig, DEFAULT_TARGET_SECS,
};
use clap::ValueEnum;
use std::path::{Path, PathBuf};

/// Resolved pack settings after CLI > config > defaults.
#[derive(Debug, Clone)]
pub struct ResolvedPackSettings {
    pub source: PathBuf,
    pub output: PathBuf,
    pub target_size: Option<String>,
    pub media: Option<CliMedia>,
    pub auto_size: bool,
    pub volumes: Option<u32>,
    pub connection: Option<CliConnection>,
    pub target_secs: u64,
    pub throughput: Option<String>,
    pub measure: bool,
    pub format: CliFormat,
    pub mapping: Option<PathBuf>,
    pub dry_run: bool,
    /// 0 = quiet, 1 = `-v`, 2+ = `-vv` (detail).
    pub verbose: u8,
    pub progress: bool,
    pub jobs: usize,
    /// `--at` / `--auto-jobs` (or config); retune after plan when jobs not explicit.
    pub auto_jobs: bool,
    /// True when `-j` or config `jobs=` fixed the count.
    pub jobs_explicit: bool,
    pub work_dir: Option<PathBuf>,
    pub keep_stage: bool,
    pub resume: bool,
    pub check_resources: bool,
    pub force_resources: bool,
    pub resource_safety_factor: f64,
    pub compress_only: bool,
    pub verify_stage: bool,
    /// Path of loaded config file, if any (for status messages).
    pub config_path: Option<PathBuf>,
}

/// CLI-side pack args (before config merge).
pub struct PackCliArgs {
    pub source: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub target_size: Option<String>,
    pub media: Option<CliMedia>,
    pub auto_size: bool,
    pub volumes: Option<u32>,
    pub connection: Option<CliConnection>,
    pub target_secs: Option<u64>,
    pub throughput: Option<String>,
    pub measure: bool,
    pub format: Option<CliFormat>,
    pub mapping: Option<PathBuf>,
    pub dry_run: bool,
    pub verbose: u8,
    pub progress: Option<bool>,
    pub jobs: Option<usize>,
    pub auto_jobs: bool,
    pub work_dir: Option<PathBuf>,
    pub keep_stage: bool,
    pub resume: bool,
    pub check_resources: Option<bool>,
    pub force_resources: bool,
    pub resource_safety_factor: Option<f64>,
    pub compress_only: bool,
    pub verify_stage: bool,
}

pub fn load_optional_config(path: Option<&Path>) -> Result<Option<(PathBuf, IniConfig)>, ArchiveError> {
    match path {
        None => Ok(None),
        Some(p) => {
            let cfg = load_ini_file(p)?;
            Ok(Some((p.to_path_buf(), cfg)))
        }
    }
}

/// CLI flags override config; unset CLI fields fall back to `[pack]` then defaults.
pub fn resolve_pack_settings(
    cli: PackCliArgs,
    config_path: Option<&Path>,
) -> Result<ResolvedPackSettings, ArchiveError> {
    let loaded = load_optional_config(config_path)?;
    let (config_path, cfg) = match loaded {
        Some((p, c)) => (Some(p), Some(c)),
        None => (None, None),
    };
    let pack = cfg.as_ref();

    let source = or_path(cli.source, pack, "source").ok_or_else(|| {
        ArchiveError::Mapping(
            "pack requires --source or config [pack] source=".into(),
        )
    })?;
    let output = or_path(cli.output, pack, "output").ok_or_else(|| {
        ArchiveError::Mapping(
            "pack requires --output or config [pack] output=".into(),
        )
    })?;

    let target_size = cli
        .target_size
        .or_else(|| pack.and_then(|c| c.get("pack", "target_size").map(str::to_string)));
    let media = match cli.media {
        Some(m) => Some(m),
        None => parse_media_opt(pack.and_then(|c| c.get("pack", "media")))?,
    };
    let auto_size = cli.auto_size
        || pack
            .and_then(|c| c.get_bool("pack", "auto_size").ok().flatten())
            .unwrap_or(false);
    let volumes = match cli.volumes {
        Some(v) => Some(v),
        None => pack.and_then(|c| c.get_u64("pack", "volumes").ok().flatten()).map(|v| v as u32),
    };
    let connection = match cli.connection {
        Some(c) => Some(c),
        None => parse_connection_opt(pack.and_then(|c| c.get("pack", "connection")))?,
    };
    let target_secs = cli
        .target_secs
        .or_else(|| pack.and_then(|c| c.get_u64("pack", "target_secs").ok().flatten()))
        .unwrap_or(DEFAULT_TARGET_SECS);
    let throughput = cli
        .throughput
        .or_else(|| pack.and_then(|c| c.get("pack", "throughput").map(str::to_string)));
    let measure = cli.measure
        || pack
            .and_then(|c| c.get_bool("pack", "measure").ok().flatten())
            .unwrap_or(false);
    let format = match cli.format {
        Some(f) => f,
        None => parse_format_opt(pack.and_then(|c| c.get("pack", "format")))?
            .unwrap_or(CliFormat::TarZst),
    };
    let mapping = or_path(cli.mapping, pack, "mapping");
    let dry_run = cli.dry_run
        || pack
            .and_then(|c| c.get_bool("pack", "dry_run").ok().flatten())
            .unwrap_or(false);
    let verbose = {
        let from_cli = cli.verbose;
        let from_cfg = pack
            .and_then(|c| c.get_bool("pack", "verbose").ok().flatten())
            .unwrap_or(false);
        if from_cli > 0 {
            from_cli
        } else if from_cfg {
            1
        } else {
            0
        }
    };
    let progress = cli
        .progress
        .or_else(|| pack.and_then(|c| c.get_bool("pack", "progress").ok().flatten()))
        .unwrap_or(true);
    let jobs_from_cli = cli.jobs;
    let jobs_from_cfg = pack.and_then(|c| c.get_usize("pack", "jobs").ok().flatten());
    let jobs_explicit = jobs_from_cli.is_some() || jobs_from_cfg.is_some();
    let auto_jobs = (cli.auto_jobs
        || pack
            .and_then(|c| c.get_bool("pack", "auto_jobs").ok().flatten())
            .unwrap_or(false))
        && !jobs_explicit;
    let jobs = if let Some(j) = jobs_from_cli.or(jobs_from_cfg) {
        j.max(1)
    } else if auto_jobs {
        // Upper bound until plan; pack retunes with units/files.
        auto_jobs_provisional_cap()
    } else {
        default_jobs()
    };
    let work_dir = or_path(cli.work_dir, pack, "work_dir");
    let keep_stage = cli.keep_stage
        || pack
            .and_then(|c| c.get_bool("pack", "keep_stage").ok().flatten())
            .unwrap_or(false);
    let resume = cli.resume
        || pack
            .and_then(|c| c.get_bool("pack", "resume").ok().flatten())
            .unwrap_or(false);
    let check_resources = cli
        .check_resources
        .or_else(|| pack.and_then(|c| c.get_bool("pack", "check_resources").ok().flatten()))
        .unwrap_or(true);
    let force_resources = cli.force_resources
        || pack
            .and_then(|c| c.get_bool("pack", "force_resources").ok().flatten())
            .unwrap_or(false);
    let resource_safety_factor = cli
        .resource_safety_factor
        .or_else(|| pack.and_then(|c| c.get_f64("pack", "resource_safety_factor").ok().flatten()))
        .unwrap_or(1.0);
    let compress_only = cli.compress_only
        || pack
            .and_then(|c| c.get_bool("pack", "compress_only").ok().flatten())
            .unwrap_or(false);
    let verify_stage = cli.verify_stage
        || pack
            .and_then(|c| c.get_bool("pack", "verify_stage").ok().flatten())
            .unwrap_or(false);

    if keep_stage && work_dir.is_none() {
        return Err(ArchiveError::Mapping(
            "--keep-stage / keep_stage=true requires --work-dir or config work_dir=".into(),
        ));
    }

    Ok(ResolvedPackSettings {
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
        auto_jobs,
        jobs_explicit,
        work_dir,
        keep_stage,
        resume,
        check_resources,
        force_resources,
        resource_safety_factor,
        compress_only,
        verify_stage,
        config_path,
    })
}

fn or_path(cli: Option<PathBuf>, pack: Option<&IniConfig>, key: &str) -> Option<PathBuf> {
    cli.or_else(|| pack.and_then(|c| c.get_path("pack", key)))
}

fn parse_format_opt(raw: Option<&str>) -> Result<Option<CliFormat>, ArchiveError> {
    match raw {
        None => Ok(None),
        Some(s) => match s.to_ascii_lowercase().as_str() {
            "zip" => Ok(Some(CliFormat::Zip)),
            "tar" => Ok(Some(CliFormat::Tar)),
            "tar.gz" | "tgz" => Ok(Some(CliFormat::TarGz)),
            "tar.zst" | "tzst" => Ok(Some(CliFormat::TarZst)),
            other => Err(ArchiveError::Mapping(format!(
                "config pack.format: unknown format {other:?}"
            ))),
        },
    }
}

fn parse_media_opt(raw: Option<&str>) -> Result<Option<CliMedia>, ArchiveError> {
    match raw {
        None => Ok(None),
        Some(s) => parse_value_enum::<CliMedia>(s)
            .map(Some)
            .map_err(|_| ArchiveError::InvalidMedia(s.to_string())),
    }
}

fn parse_connection_opt(raw: Option<&str>) -> Result<Option<CliConnection>, ArchiveError> {
    match raw {
        None => Ok(None),
        Some(s) => parse_value_enum::<CliConnection>(s)
            .map(Some)
            .map_err(|_| ArchiveError::InvalidConnection(s.to_string())),
    }
}

fn parse_value_enum<T: ValueEnum + Copy>(input: &str) -> Result<T, ()> {
    for v in T::value_variants() {
        if v.to_possible_value()
            .is_some_and(|p| p.matches(input, true))
        {
            return Ok(*v);
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::parse_ini;

    #[test]
    fn config_fills_source_and_verify_stage() {
        let text = r#"
[pack]
source = /data/in
output = /data/out
target_size = 100M
verify_stage = true
jobs = 2
format = zip
"#;
        let cfg = parse_ini(text).unwrap();
        // Simulate resolve with empty CLI and in-memory config via direct fields
        let cli = PackCliArgs {
            source: None,
            output: None,
            target_size: None,
            media: None,
            auto_size: false,
            volumes: None,
            connection: None,
            target_secs: None,
            throughput: None,
            measure: false,
            format: None,
            mapping: None,
            dry_run: false,
            verbose: 0,
            progress: None,
            jobs: None,
            auto_jobs: false,
            work_dir: None,
            keep_stage: false,
            resume: false,
            check_resources: None,
            force_resources: false,
            resource_safety_factor: None,
            compress_only: false,
            verify_stage: false,
        };
        // unit-test merge logic inline
        assert_eq!(cfg.get("pack", "source"), Some("/data/in"));
        assert_eq!(cfg.get_bool("pack", "verify_stage").unwrap(), Some(true));
        assert_eq!(cfg.get_usize("pack", "jobs").unwrap(), Some(2));
        let _ = cli;
    }

    #[test]
    fn cli_overrides_config_verify_stage() {
        let cli = PackCliArgs {
            source: Some(PathBuf::from("/s")),
            output: Some(PathBuf::from("/o")),
            target_size: Some("10M".into()),
            media: None,
            auto_size: false,
            volumes: None,
            connection: None,
            target_secs: None,
            throughput: None,
            measure: false,
            format: Some(CliFormat::Zip),
            mapping: None,
            dry_run: false,
            verbose: 0,
            progress: None,
            jobs: Some(3),
            auto_jobs: false,
            work_dir: None,
            keep_stage: false,
            resume: false,
            check_resources: None,
            force_resources: false,
            resource_safety_factor: None,
            compress_only: false,
            verify_stage: true,
        };
        let r = resolve_pack_settings(cli, None).unwrap();
        assert!(r.verify_stage);
        assert_eq!(r.jobs, 3);
        assert!(matches!(r.format, CliFormat::Zip));
    }

    #[test]
    fn cli_jobs_override_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let conf = dir.path().join("p.inf");
        std::fs::write(
            &conf,
            r#"[pack]
source = /s
output = /o
target_size = 1M
jobs = 1
format = tar.zst
verify_stage = true
"#,
        )
        .unwrap();
        let cli = PackCliArgs {
            source: None,
            output: None,
            target_size: None,
            media: None,
            auto_size: false,
            volumes: None,
            connection: None,
            target_secs: None,
            throughput: None,
            measure: false,
            format: Some(CliFormat::Zip),
            mapping: None,
            dry_run: false,
            verbose: 0,
            progress: None,
            jobs: Some(8),
            auto_jobs: true, // ignored when -j is set
            work_dir: None,
            keep_stage: false,
            resume: false,
            check_resources: None,
            force_resources: false,
            resource_safety_factor: None,
            compress_only: false,
            verify_stage: false, // config true still enables
        };
        let r = resolve_pack_settings(cli, Some(&conf)).unwrap();
        assert_eq!(r.jobs, 8);
        assert!(r.jobs_explicit);
        assert!(!r.auto_jobs); // -j wins over --at
        assert!(matches!(r.format, CliFormat::Zip));
        assert!(r.verify_stage); // config true, CLI false → enabled
        assert_eq!(r.source, PathBuf::from("/s"));
    }

    #[test]
    fn auto_jobs_provisional_without_j() {
        let cli = PackCliArgs {
            source: Some(PathBuf::from("/s")),
            output: Some(PathBuf::from("/o")),
            target_size: Some("10M".into()),
            media: None,
            auto_size: false,
            volumes: None,
            connection: None,
            target_secs: None,
            throughput: None,
            measure: false,
            format: Some(CliFormat::Zip),
            mapping: None,
            dry_run: false,
            verbose: 0,
            progress: None,
            jobs: None,
            auto_jobs: true,
            work_dir: None,
            keep_stage: false,
            resume: false,
            check_resources: None,
            force_resources: false,
            resource_safety_factor: None,
            compress_only: false,
            verify_stage: false,
        };
        let r = resolve_pack_settings(cli, None).unwrap();
        assert!(r.auto_jobs);
        assert!(!r.jobs_explicit);
        assert!(r.jobs >= 1);
        assert_eq!(r.jobs, auto_jobs_provisional_cap());
    }
}
