//! Probe free space and best-effort connection class for a destination path.

use crate::adapters::fs::space;
use crate::core::domain::{ArchiveError, ConnectionKind};
use crate::core::ports::{MediaProbe, ProbedMedia};
use std::path::Path;

#[derive(Debug, Default)]
pub struct LocalMediaProbe;

impl LocalMediaProbe {
    pub fn new() -> Self {
        Self
    }
}

impl MediaProbe for LocalMediaProbe {
    fn probe(
        &self,
        path: &Path,
        connection_hint: Option<ConnectionKind>,
    ) -> Result<ProbedMedia, ArchiveError> {
        let (free_bytes, total_bytes) = space::disk_space(path)?;
        let space_path = path.to_path_buf();

        let (connection, detection) = if let Some(hint) = connection_hint {
            (hint, "cli_hint".to_string())
        } else if let Some((conn, why)) = platform_detect(&space_path) {
            (conn, why)
        } else {
            let (conn, why) = path_heuristic(&space_path);
            (conn, why)
        };

        Ok(ProbedMedia {
            free_bytes,
            total_bytes,
            connection,
            throughput_bytes_per_sec: None,
            detection,
        })
    }
}

/// Best-effort platform detection. Prefer `diskutil` Protocol on macOS.
fn platform_detect(path: &Path) -> Option<(ConnectionKind, String)> {
    #[cfg(target_os = "macos")]
    {
        diskutil_protocol(path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        None
    }
}

#[cfg(target_os = "macos")]
fn diskutil_protocol(path: &Path) -> Option<(ConnectionKind, String)> {
    use std::process::Command;

    let output = Command::new("diskutil")
        .args(["info", &path.to_string_lossy()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let protocol = text.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("Protocol:")
            .map(|v| v.trim().to_ascii_lowercase())
    })?;

    let conn = if protocol.contains("usb") {
        // diskutil rarely distinguishes USB2 vs USB3; default to Usb3 class rates.
        ConnectionKind::Usb3
    } else if protocol.contains("thunderbolt") {
        ConnectionKind::Thunderbolt
    } else if protocol.contains("network") || protocol.contains("smb") || protocol.contains("nfs")
    {
        ConnectionKind::Network
    } else if protocol.contains("apfs") || protocol.contains("sata") || protocol.contains("pci")
    {
        ConnectionKind::Local
    } else {
        return None;
    };

    Some((conn, format!("diskutil_protocol:{protocol}")))
}

fn path_heuristic(path: &Path) -> (ConnectionKind, String) {
    let s = path.to_string_lossy().to_ascii_lowercase();

    if s.starts_with("/volumes/")
        || s.starts_with("/media/")
        || s.starts_with("/run/media/")
        || s.starts_with("/mnt/")
    {
        // Removable / external mount points — USB3-class default.
        // Network shares may also appear under /Volumes on macOS.
        return (
            ConnectionKind::Usb3,
            format!("path_heuristic:{}", mount_prefix(&s)),
        );
    }

    (ConnectionKind::Local, "path_heuristic:local".into())
}

fn mount_prefix(s: &str) -> &str {
    if s.starts_with("/volumes/") {
        "/Volumes"
    } else if s.starts_with("/run/media/") {
        "/run/media"
    } else if s.starts_with("/media/") {
        "/media"
    } else if s.starts_with("/mnt/") {
        "/mnt"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probes_current_dir_free_space() {
        let probe = LocalMediaProbe::new();
        let media = probe
            .probe(Path::new("."), Some(ConnectionKind::Local))
            .unwrap();
        assert!(media.free_bytes > 0);
        assert!(media.total_bytes >= media.free_bytes);
        assert_eq!(media.connection, ConnectionKind::Local);
        assert_eq!(media.detection, "cli_hint");
    }

    #[test]
    fn path_heuristic_volumes() {
        let (c, d) = path_heuristic(Path::new("/Volumes/USB"));
        assert_eq!(c, ConnectionKind::Usb3);
        assert!(d.contains("path_heuristic"));
    }
}
