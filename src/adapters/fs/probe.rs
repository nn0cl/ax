//! Probe free space and best-effort connection class for a destination path.

use crate::core::domain::{ArchiveError, ConnectionKind};
use crate::core::ports::{MediaProbe, ProbedMedia};
use std::path::{Path, PathBuf};

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
        let space_path = nearest_existing_dir(path)?;
        let (free_bytes, total_bytes) = volume_space(&space_path)?;

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

fn nearest_existing_dir(path: &Path) -> Result<PathBuf, ArchiveError> {
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
        if cur.exists() {
            return Ok(if cur.is_dir() {
                cur
            } else {
                cur.parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            });
        }
        match cur.parent() {
            Some(p) if p != cur => cur = p.to_path_buf(),
            _ => {
                return Ok(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            }
        }
    }
}

#[cfg(unix)]
fn volume_space(path: &Path) -> Result<(u64, u64), ArchiveError> {
    use std::ffi::CString;
    use std::io;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| ArchiveError::Engine(format!("path for statvfs: {e}")))?;

    // SAFETY: c_path is a valid C string; statvfs writes into a local struct.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return Err(ArchiveError::Io(io::Error::last_os_error()));
        }
        let frsize = stat.f_frsize as u64;
        let free = stat.f_bavail as u64 * frsize;
        let total = stat.f_blocks as u64 * frsize;
        Ok((free, total))
    }
}

#[cfg(not(unix))]
fn volume_space(path: &Path) -> Result<(u64, u64), ArchiveError> {
    let _ = path;
    Ok((0, 0))
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
