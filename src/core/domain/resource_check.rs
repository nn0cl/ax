//! Pure estimates for disk / memory headroom before pack.

/// Inputs for resource planning after units are known.
#[derive(Debug, Clone)]
pub struct ResourcePlanInput {
    /// Sum of logical bytes for units still to build.
    pub remaining_logical_bytes: u64,
    /// Largest single pending unit (logical).
    pub max_unit_logical_bytes: u64,
    /// Compress worker count.
    pub compress_jobs: usize,
    /// Transfer queue depth (staged archives that may sit on work-dir).
    pub transfer_queue: usize,
    /// Whether archives are staged on a separate work volume.
    pub use_work_dir: bool,
    /// Multiplier for size estimates (1.0 = assume archive ≈ logical worst case).
    pub safety_factor: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskRequirement {
    /// Conservative peak bytes needed on the final output volume.
    pub output_bytes: u64,
    /// Conservative peak bytes needed on the work-dir volume (0 if unused).
    pub work_dir_bytes: u64,
}

/// Estimate peak disk needs (logical-size based; compressed is usually smaller).
pub fn estimate_disk_requirement(input: &ResourcePlanInput) -> DiskRequirement {
    let factor = if input.safety_factor.is_finite() && input.safety_factor > 0.0 {
        input.safety_factor
    } else {
        1.0
    };
    let remaining = input.remaining_logical_bytes;
    let max_unit = input.max_unit_logical_bytes.max(1);

    // Output must eventually hold all remaining archives (≈ remaining logical, worst case).
    let output_bytes = scale(remaining, factor);

    // Work-dir peak: in-flight compress (jobs) + queued for transfer, each up to max unit.
    let work_dir_bytes = if input.use_work_dir {
        let slots = input
            .compress_jobs
            .saturating_add(input.transfer_queue)
            .max(1) as u64;
        let peak = max_unit.saturating_mul(slots);
        scale(peak.min(remaining.max(max_unit)), factor)
    } else {
        // partial files on output volume: a few in-flight max units
        let slots = input.compress_jobs.max(1) as u64;
        scale(max_unit.saturating_mul(slots).min(remaining.max(max_unit)), factor)
    };

    DiskRequirement {
        output_bytes,
        work_dir_bytes,
    }
}

fn scale(bytes: u64, factor: f64) -> u64 {
    let v = (bytes as f64) * factor;
    if v.is_finite() && v > 0.0 {
        v.ceil() as u64
    } else {
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceLevel {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct ResourceFinding {
    pub level: ResourceLevel,
    pub message: String,
}

/// Compare free space / memory against requirements.
pub fn evaluate_resources(
    output_free: u64,
    work_free: Option<u64>,
    mem_available: Option<u64>,
    req: &DiskRequirement,
    use_work_dir: bool,
    compress_jobs: usize,
) -> Vec<ResourceFinding> {
    let mut out = Vec::new();

    out.push(ResourceFinding {
        level: ResourceLevel::Ok,
        message: format!(
            "output free={} need≈{} ({})",
            human(output_free),
            human(req.output_bytes),
            if output_free >= req.output_bytes {
                "ok"
            } else {
                "SHORT"
            }
        ),
    });
    if output_free < req.output_bytes {
        out.push(ResourceFinding {
            level: ResourceLevel::Fail,
            message: format!(
                "output volume free space too low: have {}, need ≈ {} (logical remaining × safety)",
                human(output_free),
                human(req.output_bytes)
            ),
        });
    } else if output_free < req.output_bytes.saturating_mul(12) / 10 {
        // < 20% headroom
        out.push(ResourceFinding {
            level: ResourceLevel::Warn,
            message: format!(
                "output free space is tight (have {}, need ≈ {})",
                human(output_free),
                human(req.output_bytes)
            ),
        });
    }

    if use_work_dir {
        let wf = work_free.unwrap_or(0);
        out.push(ResourceFinding {
            level: ResourceLevel::Ok,
            message: format!(
                "work-dir free={} peak≈{} ({})",
                human(wf),
                human(req.work_dir_bytes),
                if wf >= req.work_dir_bytes {
                    "ok"
                } else {
                    "SHORT"
                }
            ),
        });
        if wf < req.work_dir_bytes {
            out.push(ResourceFinding {
                level: ResourceLevel::Fail,
                message: format!(
                    "work-dir free space too low: have {}, peak staging ≈ {} (jobs+queue × max unit)",
                    human(wf),
                    human(req.work_dir_bytes)
                ),
            });
        } else if wf < req.work_dir_bytes.saturating_mul(12) / 10 {
            out.push(ResourceFinding {
                level: ResourceLevel::Warn,
                message: format!(
                    "work-dir free space is tight (have {}, peak ≈ {})",
                    human(wf),
                    human(req.work_dir_bytes)
                ),
            });
        }
    }

    if let Some(mem) = mem_available {
        let soft = 512 * 1024 * 1024u64; // 512 MiB
        let per_job = 256 * 1024 * 1024u64;
        let prefer = soft.max(per_job.saturating_mul(compress_jobs as u64));
        out.push(ResourceFinding {
            level: ResourceLevel::Ok,
            message: format!("memory available≈{} (soft target ≥ {})", human(mem), human(prefer)),
        });
        if mem < soft {
            out.push(ResourceFinding {
                level: ResourceLevel::Warn,
                message: format!(
                    "low system memory available≈{} (< {}); pack may thrash",
                    human(mem),
                    human(soft)
                ),
            });
        }
    }

    out
}

fn human(n: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let x = n as f64;
    if x >= GIB {
        format!("{:.2} GiB", x / GIB)
    } else if x >= MIB {
        format!("{:.2} MiB", x / MIB)
    } else if x >= KIB {
        format!("{:.2} KiB", x / KIB)
    } else {
        format!("{n} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_dir_peak_bounded_by_queue() {
        let d = estimate_disk_requirement(&ResourcePlanInput {
            remaining_logical_bytes: 100 * 1024 * 1024 * 1024,
            max_unit_logical_bytes: 4 * 1024 * 1024 * 1024,
            compress_jobs: 4,
            transfer_queue: 16,
            use_work_dir: true,
            safety_factor: 1.0,
        });
        // 4+16 = 20 * 4GiB = 80GiB peak, not full 100GiB
        assert_eq!(d.work_dir_bytes, 20 * 4 * 1024 * 1024 * 1024);
        assert_eq!(d.output_bytes, 100 * 1024 * 1024 * 1024);
    }

    #[test]
    fn fail_when_output_short() {
        let req = DiskRequirement {
            output_bytes: 10_000,
            work_dir_bytes: 0,
        };
        let f = evaluate_resources(100, None, None, &req, false, 1);
        assert!(f.iter().any(|x| x.level == ResourceLevel::Fail));
    }
}
