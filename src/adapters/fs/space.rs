//! Disk free-space helpers (statvfs).

use crate::core::domain::ArchiveError;
use std::path::{Path, PathBuf};

/// Free and total bytes for the volume containing `path` (nearest existing ancestor).
pub fn disk_space(path: &Path) -> Result<(u64, u64), ArchiveError> {
    let p = nearest_existing_dir(path)?;
    volume_space(&p)
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
            _ => return std::env::current_dir().map_err(ArchiveError::Io),
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
