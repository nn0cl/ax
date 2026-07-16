//! Sequential write micro-benchmark for transfer estimation.

use crate::core::domain::ArchiveError;
use crate::core::ports::ThroughputMeter;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct LocalThroughputMeter;

impl LocalThroughputMeter {
    pub fn new() -> Self {
        Self
    }
}

impl ThroughputMeter for LocalThroughputMeter {
    fn measure_write_throughput(
        &self,
        path: &Path,
        sample_bytes: u64,
    ) -> Result<u64, ArchiveError> {
        let sample_bytes = sample_bytes.max(1024 * 1024); // at least 1 MiB
        let dir = nearest_writable_dir(path)?;
        let tmp = unique_temp_path(&dir);

        let result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp)
                .map_err(|e| ArchiveError::MeasureFailed(e.to_string()))?;

            let chunk = vec![0u8; 1024 * 1024];
            let mut remaining = sample_bytes;
            let start = Instant::now();
            while remaining > 0 {
                let n = remaining.min(chunk.len() as u64) as usize;
                file.write_all(&chunk[..n])
                    .map_err(|e| ArchiveError::MeasureFailed(e.to_string()))?;
                remaining -= n as u64;
            }
            file.sync_all()
                .map_err(|e| ArchiveError::MeasureFailed(e.to_string()))?;
            let elapsed = start.elapsed().as_secs_f64().max(1e-6);
            let bps = (sample_bytes as f64 / elapsed).floor() as u64;
            Ok(bps.max(1))
        })();

        let _ = fs::remove_file(&tmp);
        result
    }
}

fn nearest_writable_dir(path: &Path) -> Result<PathBuf, ArchiveError> {
    let mut cur = if path.exists() {
        if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        }
    } else {
        path.to_path_buf()
    };

    loop {
        if cur.exists() && cur.is_dir() {
            return Ok(cur);
        }
        match cur.parent() {
            Some(p) if p != cur => cur = p.to_path_buf(),
            _ => {
                return std::env::current_dir().map_err(ArchiveError::Io);
            }
        }
    }
}

fn unique_temp_path(dir: &Path) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    dir.join(format!(".archiver-measure-{nanos}.tmp"))
}
