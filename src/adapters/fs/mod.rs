//! Local filesystem helpers for archive adapters.

mod hash;
mod local_fs;
mod measure;
mod probe;
mod resources;
mod scan;
mod space;

pub use hash::Sha256Hasher;
pub use local_fs::LocalFileSystem;
pub use measure::LocalThroughputMeter;
pub use probe::LocalMediaProbe;
pub use resources::LocalResourceProbe;
pub use scan::LocalFileScanner;

use crate::core::domain::ArchiveError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// One file to pack into an archive, with the relative member path.
#[derive(Debug, Clone)]
pub struct PackEntry {
    pub source: PathBuf,
    pub member_name: String,
}

/// Expand input paths into a flat list of file entries with member names.
///
/// - File input `./a.txt` → member `a.txt`
/// - Directory input `./src` → members `src/...` using the directory's last
///   component as the root name.
pub fn collect_pack_entries(inputs: &[PathBuf]) -> Result<Vec<PackEntry>, ArchiveError> {
    let mut entries = Vec::new();

    for input in inputs {
        if !input.exists() {
            return Err(ArchiveError::InputNotFound(input.display().to_string()));
        }

        if input.is_file() {
            let name = input
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".to_string());
            entries.push(PackEntry {
                source: input.clone(),
                member_name: name,
            });
            continue;
        }

        if input.is_dir() {
            let root_name = input
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "dir".to_string());

            for walk in WalkDir::new(input).follow_links(false) {
                let walk = walk.map_err(|e| ArchiveError::Engine(e.to_string()))?;
                let path = walk.path();
                if path.is_dir() {
                    continue;
                }
                let rel = path
                    .strip_prefix(input)
                    .map_err(|e| ArchiveError::Engine(e.to_string()))?;
                let member = Path::new(&root_name)
                    .join(rel)
                    .to_string_lossy()
                    .replace('\\', "/");
                entries.push(PackEntry {
                    source: path.to_path_buf(),
                    member_name: member,
                });
            }
        }
    }

    Ok(entries)
}

pub fn ensure_parent_dir(path: &Path) -> Result<(), ArchiveError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}
