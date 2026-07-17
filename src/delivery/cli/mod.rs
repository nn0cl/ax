//! CLI delivery: parse, dispatch, adapters composition.

mod args;
mod helpers;
mod pack_config;

pub use args::{Cli, Commands};

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
    format_estimate_report_with_meta, parse_size, parse_size_with_preset, ArchiveError,
    ConnectionKind,
};
use crate::core::ports::NullProgress;
use crate::delivery::progress::SingleLineProgress;
use helpers::{handle_pack_failure, parse_format, parse_throughput, resolve_pack_budget};
use pack_config::{resolve_pack_settings, PackCliArgs};
use std::process::ExitCode;

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

/// High-level pack log lines safe to print (no per-unit spam).
fn is_pack_summary_line(line: &str) -> bool {
    line.starts_with("budget=")
        || line.starts_with("resume:")
        || line.starts_with("plan_file=")
        || line.starts_with("work_dir=")
        || line.starts_with("stage_root=")
        || line.starts_with("resource:")
        || line.starts_with("pipeline:")
        || line.starts_with("mode=")
        || line.starts_with("volumes=")
        || line.starts_with("budget_from_volumes:")
        || line.starts_with("stage_policy:")
        || line.starts_with("cleaned stage")
        || line.starts_with("kept stage")
        || line.starts_with("warning: failed to remove stage")
        || line.starts_with("summary:")
        || line.starts_with("bytes:")
        || line.starts_with("verify_stage=")
        || line.starts_with("size_estimate:")
        || line.starts_with("mapping:")
}

fn run_inner(cli: Cli) -> Result<String, ArchiveError> {
    let engine = CompositeArchiveEngine::new();
    let scanner = LocalFileScanner::new();
    let mapping_store = JsonMappingStore::new();
    let media_probe = LocalMediaProbe::new();
    let meter = LocalThroughputMeter::new();
    let fs = LocalFileSystem::new();
    let hasher = Sha256Hasher::new();
    let config_path = cli.config.clone();
    let global_size = cli.size.clone();
    let global_output = cli.output.clone();

    match cli.command {
        Commands::Create { format, inputs } => {
            let output = global_output.ok_or_else(|| {
                ArchiveError::Mapping("create requires -o/--output <archive path>".into())
            })?;
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
            pack_output,
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
            max_use,
            work_dir,
            keep_stage,
            resume,
            check_resources,
            force_resources,
            resource_safety_factor,
            compress_only,
            verify_stage,
        } => {
            // Precedence: pack-local > global -o/-s > config (config applied inside resolve).
            let output = pack_output.or(global_output.clone());
            let target_size = target_size.or(global_size.clone());
            let settings = resolve_pack_settings(
                PackCliArgs {
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
                    max_use,
                    work_dir,
                    keep_stage,
                    resume,
                    check_resources,
                    force_resources,
                    resource_safety_factor,
                    compress_only,
                    verify_stage,
                },
                config_path.as_deref(),
            )?;

            let source = settings.source;
            let output = settings.output;
            let target_size = settings.target_size;
            let media = settings.media;
            let auto_size = settings.auto_size;
            let volumes = settings.volumes;
            let connection = settings.connection;
            let target_secs = settings.target_secs;
            let throughput = settings.throughput;
            let mut throughput_bps = parse_throughput(throughput)?;
            let measure = settings.measure;
            let format = settings.format;
            let mapping = settings.mapping;
            let dry_run = settings.dry_run;
            let verbose = settings.verbose;
            let progress = settings.progress;
            let jobs = settings.jobs;
            let auto_jobs = settings.auto_jobs;
            let jobs_explicit = settings.jobs_explicit;
            let max_use = settings.max_use;
            let work_dir = settings.work_dir;
            let keep_stage = settings.keep_stage;
            let resume = settings.resume;
            let check_resources = settings.check_resources;
            let force_resources = settings.force_resources;
            let resource_safety_factor = settings.resource_safety_factor;
            let compress_only = settings.compress_only;
            let verify_stage = settings.verify_stage;

            // Fixed dashboard: total + compress0..N + transfer (in-place redraw).
            // -vv only affects text logs, not the dashboard layout.
            let line_progress =
                SingleLineProgress::new(progress && !dry_run, verbose >= 2, jobs.max(1));
            let resources = LocalResourceProbe::new();
            if progress && !dry_run && !line_progress.is_enabled() {
                eprintln!(
                    "warning: progress UI disabled (no controlling terminal); \
                     try running without redirect, or check /dev/tty"
                );
            }
            // One preflight line (avoid many status_message lines before scan).
            {
                let mut pre = format!("pack source={} output={}", source.display(), output.display());
                if let Some(ref p) = settings.config_path {
                    pre.push_str(&format!(" config={}", p.display()));
                }
                if let Some(ref wd) = work_dir {
                    pre.push_str(&format!(" work_dir={}", wd.display()));
                }
                if compress_only {
                    pre.push_str(" compress-only");
                }
                if verify_stage {
                    pre.push_str(" verify_stage");
                }
                if let Some(n) = volumes {
                    pre.push_str(&format!(" volumes={n}"));
                }
                line_progress.status_message(&pre);
            }

            let budget = if auto_size {
                line_progress.status_message(if measure {
                    "auto-size: measuring write speed…"
                } else {
                    "auto-size: estimating unit size…"
                });
                let req = EstimateUnitRequest {
                    path: output.clone(),
                    connection_hint: connection.map(ConnectionKind::from),
                    throughput_override: throughput_bps,
                    target_transfer_secs: target_secs,
                    measure,
                    measure_sample_bytes: 64 * 1024 * 1024,
                };
                let out = EstimateTransferUnit::new(&media_probe, &meter).execute(&req)?;
                line_progress.status_message(&format!(
                    "budget {} ({})",
                    out.estimate.recommended_unit_bytes,
                    crate::core::domain::human_bytes(out.estimate.recommended_unit_bytes)
                ));
                // Prefer estimate/measure throughput for I/O-aware auto-jobs.
                if out.estimate.assumed_throughput_bytes_per_sec > 0 {
                    throughput_bps = Some(out.estimate.assumed_throughput_bytes_per_sec);
                }
                out.estimate.recommended_unit_bytes
            } else {
                if let Some(ref ts) = target_size {
                    if let Ok((_, Some(p))) = parse_size_with_preset(ts) {
                        line_progress.status_message(&format!(
                            "size_preset={} ({})",
                            p.as_str(),
                            crate::core::domain::human_bytes(p.bytes())
                        ));
                    }
                }
                resolve_pack_budget(target_size, media, auto_size, volumes, connection)?
                    .unwrap_or(0)
            };

            let mapping_path = mapping.unwrap_or_else(|| default_mapping_path(&output));
            line_progress.status_message(&format!("scanning {} …", source.display()));

            let req = PackRequest {
                source,
                output: output.clone(),
                budget,
                format: format.into(),
                mapping_path: mapping_path.clone(),
                dry_run,
                verbose,
                jobs,
                auto_jobs,
                jobs_explicit,
                connection: connection.map(ConnectionKind::from),
                throughput_bps,
                max_use,
                work_dir,
                keep_stage,
                resume,
                check_resources,
                force_resources,
                resource_safety_factor,
                compress_only,
                volumes,
                verify_stage,
            };

            let packer =
                PackSourceTree::new(&engine, &scanner, &mapping_store, &fs, &hasher, &resources);
            let result = match if line_progress.is_enabled() {
                packer.execute_with_progress(&req, &line_progress)
            } else {
                packer.execute_with_progress(&req, &NullProgress)
            } {
                Ok(r) => r,
                Err(fail) => {
                    handle_pack_failure(fail)?;
                    unreachable!()
                }
            };

            // Keep stdout short: summary only; unit detail needs -vv.
            if dry_run {
                for line in result.messages.iter().filter(|l| is_pack_summary_line(l)) {
                    println!("{line}");
                }
                if verbose >= 2 {
                    for line in result.messages.iter().filter(|l| l.starts_with("plan")) {
                        println!("{line}");
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
                let mut out = format!(
                    "packed {} archive(s), {} empty dir(s), budget {} bytes\nmapping: {}\n",
                    result.plan.units.len(),
                    result.plan.empty_directories.len(),
                    budget,
                    mapping_path.display()
                );
                for line in result.messages.iter().filter(|l| is_pack_summary_line(l)) {
                    out.push_str(line);
                    out.push('\n');
                }
                if verbose >= 2 {
                    for line in result
                        .messages
                        .iter()
                        .filter(|l| l.starts_with("plan") || l.starts_with("transferred"))
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
            let mapping = mapping.or(global_output.clone()).ok_or_else(|| {
                ArchiveError::Mapping(
                    "restore requires -m/--mapping or global -o <map.json>".into(),
                )
            })?;
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
            progress,
        } => {
            let mapping = mapping.or(global_output.clone()).ok_or_else(|| {
                ArchiveError::Mapping(
                    "verify requires -m/--mapping or global -o <map.json>".into(),
                )
            })?;
            let archives_root =
                archives_root.unwrap_or_else(|| default_verify_archives_root(&mapping));
            let req = VerifyRequest {
                mapping_path: mapping,
                archives_root,
                quick,
            };
            let line_progress = SingleLineProgress::new(progress, false, 1);
            if progress && !line_progress.is_enabled() {
                eprintln!(
                    "warning: progress UI disabled (no controlling terminal); \
                     try running without redirect, or check /dev/tty"
                );
            }
            let report = if line_progress.is_enabled() {
                VerifyMapping::new(&engine, &mapping_store, &fs, &hasher)
                    .execute_with_progress(&req, &line_progress)?
            } else {
                VerifyMapping::new(&engine, &mapping_store, &fs, &hasher).execute(&req)?
            };
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
