//! CLI parsing helpers and pack budget resolution.

use crate::core::application::PackFailure;
use crate::core::domain::{
    effective_budget, parse_size, ArchiveError, ArchiveFormat, ConnectionKind, MediaKind,
};
use super::args::CliConnection;
use super::args::CliMedia;

pub(crate) fn parse_format(raw: Option<String>) -> Result<Option<ArchiveFormat>, ArchiveError> {
    match raw {
        Some(s) => Ok(Some(ArchiveFormat::parse(&s)?)),
        None => Ok(None),
    }
}

pub(crate) fn parse_throughput(raw: Option<String>) -> Result<Option<u64>, ArchiveError> {
    match raw {
        Some(s) => Ok(Some(parse_size(&s)?)),
        None => Ok(None),
    }
}

/// Budget for pack: explicit flags, or 0 when only `--volumes` (filled after scan).
pub(crate) fn handle_pack_failure(fail: PackFailure) -> Result<(), ArchiveError> {
    if let Some(stage) = &fail.leftover_stage {
        eprintln!(
            "kept staging directory (--keep-stage):\n  {}",
            stage.display()
        );
    }
    Err(fail.error)
}

pub(crate) fn resolve_pack_budget(
    target_size: Option<String>,
    media: Option<CliMedia>,
    auto_size: bool,
    volumes: Option<u32>,
    connection: Option<CliConnection>,
) -> Result<Option<u64>, ArchiveError> {
    if auto_size {
        return Ok(None); // caller handles
    }
    match (target_size, media, volumes) {
        (Some(s), None, _) => {
            let conn = connection
                .map(ConnectionKind::from)
                .unwrap_or(ConnectionKind::Local);
            let bytes = parse_size(&s)?;
            Ok(Some(effective_budget(MediaKind::Custom(bytes), conn)))
        }
        (None, Some(m), _) => {
            let conn = connection
                .map(ConnectionKind::from)
                .unwrap_or(ConnectionKind::Local);
            Ok(Some(effective_budget(MediaKind::from(m), conn)))
        }
        (None, None, Some(_)) => Ok(Some(0)), // pack derives after scan
        _ => Err(ArchiveError::InvalidBudget),
    }
}
