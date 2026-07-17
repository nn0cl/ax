//! Disk/memory preflight before pack writes.

use crate::core::domain::{
    estimate_archive_for_sources, estimate_disk_requirement, evaluate_resources, ArchiveError,
    ArchiveFormat, PlanUnitRecord, PlanUnitStatus, ResourceLevel, ResourcePlanInput,
};
use crate::core::ports::ResourceProbe;
use std::path::Path;

pub fn transfer_queue_depth(jobs: usize, use_work_dir: bool) -> usize {
    let jobs = jobs.max(1);
    if use_work_dir {
        (jobs * 4).max(8)
    } else {
        jobs.max(1)
    }
}

/// Append `resource:` messages; hard-fail unless `force`.
#[allow(clippy::too_many_arguments)]
pub fn check_resources(
    resources: &dyn ResourceProbe,
    output: &Path,
    work_dir: Option<&Path>,
    units: &[PlanUnitRecord],
    jobs: usize,
    transfer_queue: usize,
    safety_factor: f64,
    force: bool,
    format: ArchiveFormat,
    messages: &mut Vec<String>,
) -> Result<(), ArchiveError> {
    let pending: Vec<&PlanUnitRecord> = units
        .iter()
        .filter(|u| u.status != PlanUnitStatus::Done)
        .collect();
    if pending.is_empty() {
        return Ok(());
    }

    let remaining_logical: u64 = pending.iter().map(|u| u.logical_bytes).sum();
    let max_unit = pending.iter().map(|u| u.logical_bytes).max().unwrap_or(0);
    let use_work_dir = work_dir.is_some();
    // Hard free-space check stays logical × safety (worst case; already-compressed data).
    let disk_need = estimate_disk_requirement(&ResourcePlanInput {
        remaining_logical_bytes: remaining_logical,
        max_unit_logical_bytes: max_unit,
        compress_jobs: jobs.max(1),
        transfer_queue,
        use_work_dir,
        safety_factor,
    });
    let content_est = estimate_archive_for_sources(
        pending.iter().flat_map(|u| {
            u.members
                .iter()
                .map(|m| (m.source_relpath.as_str(), m.size))
        }),
        format,
    );

    let out_sp = resources.disk_space(output)?;
    let work_sp = if let Some(wd) = work_dir {
        Some(resources.disk_space(wd)?)
    } else {
        None
    };
    let mem = resources.available_memory_bytes();
    messages.push(format!(
        "resource: need≈{} free_out={} gate=logical×safety; compressed_est≈{} (prior {:.2})",
        crate::core::domain::human_bytes(disk_need.output_bytes),
        crate::core::domain::human_bytes(out_sp.free_bytes),
        crate::core::domain::human_bytes(content_est.estimated_archive_bytes),
        content_est.ratio,
    ));
    let findings = evaluate_resources(
        out_sp.free_bytes,
        work_sp.map(|s| s.free_bytes),
        mem,
        &disk_need,
        use_work_dir,
        jobs.max(1),
    );
    // Skip routine Ok chatter; keep Warn/Fail only.
    for f in &findings {
        if f.level != ResourceLevel::Ok {
            messages.push(format!("resource: {}", f.message));
        }
    }
    let hard = findings
        .iter()
        .filter(|f| f.level == ResourceLevel::Fail)
        .map(|f| f.message.clone())
        .collect::<Vec<_>>();
    if !hard.is_empty() && !force {
        return Err(ArchiveError::InsufficientResources(hard.join("; ")));
    }
    if !hard.is_empty() && force {
        messages.push(
            "resource: FORCED continue despite insufficient free space/memory".into(),
        );
    }
    Ok(())
}
