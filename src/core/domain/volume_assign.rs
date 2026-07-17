//! Assign archive units into numbered volume directories (multi-disc layout).

use super::ArchiveError;

/// Prefix archive relative paths with `vol-001/`, `vol-002/`, … by greedy fill.
///
/// - Units are kept in order (stable layout).
/// - A volume is closed when the next unit would exceed `volume_capacity`
///   (except a single unit larger than capacity still occupies one volume alone).
/// - Fails if more than `max_volumes` volumes would be required.
pub fn assign_volume_paths(
    archive_relpaths: &[String],
    logical_sizes: &[u64],
    volume_capacity: u64,
    max_volumes: usize,
) -> Result<Vec<String>, ArchiveError> {
    if archive_relpaths.len() != logical_sizes.len() {
        return Err(ArchiveError::Engine(
            "volume assign: path/size length mismatch".into(),
        ));
    }
    if max_volumes == 0 {
        return Err(ArchiveError::Engine(
            "--volumes must be >= 1".into(),
        ));
    }
    let capacity = volume_capacity.max(1);
    if archive_relpaths.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::with_capacity(archive_relpaths.len());
    let mut vol: usize = 1;
    let mut used: u64 = 0;

    for (rel, &size) in archive_relpaths.iter().zip(logical_sizes.iter()) {
        // Open a new volume when this unit does not fit (and current is not empty).
        if used > 0 && used.saturating_add(size) > capacity {
            vol += 1;
            used = 0;
        }
        if vol > max_volumes {
            return Err(ArchiveError::Engine(format!(
                "need more than {max_volumes} volume(s) at capacity ~{capacity} bytes \
                 (raise --volumes, raise --target-size/--media, or shrink source)"
            )));
        }
        let prefix = format!("vol-{vol:03}");
        let path = if rel.is_empty() {
            prefix
        } else {
            format!("{prefix}/{rel}")
        };
        out.push(path);
        used = used.saturating_add(size);
    }

    Ok(out)
}

/// How many volumes the greedy fill would use (no max cap).
pub fn count_volumes_needed(logical_sizes: &[u64], volume_capacity: u64) -> usize {
    let capacity = volume_capacity.max(1);
    if logical_sizes.is_empty() {
        return 0;
    }
    let mut vol = 1usize;
    let mut used = 0u64;
    for &size in logical_sizes {
        if used > 0 && used.saturating_add(size) > capacity {
            vol += 1;
            used = 0;
        }
        used = used.saturating_add(size);
    }
    vol
}

/// Recommended unit/volume budget when only `--volumes N` is given: ceil(total / N).
pub fn budget_from_volume_count(total_logical: u64, volumes: u32) -> u64 {
    let n = volumes.max(1) as u64;
    total_logical.div_ceil(n).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_into_two_volumes() {
        let paths = vec![
            "a/part-0001.zip".into(),
            "b/part-0001.zip".into(),
            "c/part-0001.zip".into(),
        ];
        let sizes = vec![400u64, 400, 400];
        let out = assign_volume_paths(&paths, &sizes, 1000, 2).unwrap();
        assert_eq!(out[0], "vol-001/a/part-0001.zip");
        assert_eq!(out[1], "vol-001/b/part-0001.zip"); // 800
        assert_eq!(out[2], "vol-002/c/part-0001.zip");
    }

    #[test]
    fn fails_when_too_few_volumes() {
        let paths = vec!["a.zip".into(), "b.zip".into(), "c.zip".into()];
        let sizes = vec![400u64, 400, 400];
        let err = assign_volume_paths(&paths, &sizes, 500, 1).unwrap_err();
        assert!(err.to_string().contains("more than 1"));
    }

    #[test]
    fn oversized_unit_alone() {
        let paths = vec!["big.zip".into(), "small.zip".into()];
        let sizes = vec![2000u64, 100];
        let out = assign_volume_paths(&paths, &sizes, 1000, 2).unwrap();
        assert_eq!(out[0], "vol-001/big.zip");
        assert_eq!(out[1], "vol-002/small.zip");
    }

    #[test]
    fn budget_from_volumes_ceil() {
        assert_eq!(budget_from_volume_count(1000, 3), 334);
        assert_eq!(budget_from_volume_count(900, 3), 300);
        assert_eq!(budget_from_volume_count(0, 2), 1);
    }

    #[test]
    fn count_volumes() {
        assert_eq!(count_volumes_needed(&[400, 400, 400], 1000), 2);
        assert_eq!(count_volumes_needed(&[], 1000), 0);
    }
}
