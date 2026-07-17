use crate::core::domain::{
    estimate_transfer_unit, ArchiveError, ConnectionKind, EstimateRequest, MediaFacts,
    TransferEstimate, DEFAULT_TARGET_SECS,
};
use crate::core::ports::{MediaProbe, ThroughputMeter};
use std::path::PathBuf;

/// Request to estimate a transfer-oriented pack unit size.
#[derive(Debug, Clone)]
pub struct EstimateUnitRequest {
    pub path: PathBuf,
    pub connection_hint: Option<ConnectionKind>,
    pub throughput_override: Option<u64>,
    pub target_transfer_secs: u64,
    /// When true, run a sequential write sample on the destination volume.
    pub measure: bool,
    /// Sample size for --measure (bytes). Default applied when 0.
    pub measure_sample_bytes: u64,
}

impl Default for EstimateUnitRequest {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            connection_hint: None,
            throughput_override: None,
            target_transfer_secs: DEFAULT_TARGET_SECS,
            measure: false,
            measure_sample_bytes: 64 * 1024 * 1024,
        }
    }
}

/// Estimate plus probe metadata for CLI reporting.
#[derive(Debug, Clone)]
pub struct EstimateOutcome {
    pub estimate: TransferEstimate,
    pub detection: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub measured: bool,
}

/// Use case: probe destination + compute recommended pack unit budget.
pub struct EstimateTransferUnit<'a, P: MediaProbe, T: ThroughputMeter> {
    probe: &'a P,
    meter: &'a T,
}

impl<'a, P: MediaProbe, T: ThroughputMeter> EstimateTransferUnit<'a, P, T> {
    pub fn new(probe: &'a P, meter: &'a T) -> Self {
        Self { probe, meter }
    }

    pub fn execute(&self, req: &EstimateUnitRequest) -> Result<EstimateOutcome, ArchiveError> {
        let probed = self.probe.probe(&req.path, req.connection_hint)?;
        let mut measured = false;
        let mut throughput = req.throughput_override.or(probed.throughput_bytes_per_sec);

        if req.measure && throughput.is_none() {
            let sample = if req.measure_sample_bytes == 0 {
                64 * 1024 * 1024
            } else {
                req.measure_sample_bytes
            };
            // Cap sample to a quarter of free space when known.
            let sample = if probed.free_bytes > 0 {
                sample.min(probed.free_bytes / 4).max(1024 * 1024)
            } else {
                sample
            };
            match self.meter.measure_write_throughput(&req.path, sample) {
                Ok(bps) => {
                    throughput = Some(bps);
                    measured = true;
                }
                Err(e) => {
                    return Err(ArchiveError::MeasureFailed(format!(
                        "--measure failed (no table fallback): {e}"
                    )));
                }
            }
        }

        let facts = MediaFacts {
            free_bytes: probed.free_bytes,
            total_bytes: probed.total_bytes,
            connection: probed.connection,
            throughput_bytes_per_sec: throughput,
        };
        let secs = if req.target_transfer_secs == 0 {
            DEFAULT_TARGET_SECS
        } else {
            req.target_transfer_secs
        };
        let estimate = estimate_transfer_unit(&EstimateRequest {
            facts,
            target_transfer_secs: secs,
        });

        let detection = if measured {
            format!("{};measured_write", probed.detection)
        } else {
            probed.detection
        };

        Ok(EstimateOutcome {
            estimate,
            detection,
            free_bytes: probed.free_bytes,
            total_bytes: probed.total_bytes,
            measured,
        })
    }
}
