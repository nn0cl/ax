use crate::core::domain::{ArchiveError, SourceFile};
use crate::core::ports::FileScanner;
use std::collections::BTreeSet;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct LocalFileScanner;

impl LocalFileScanner {
    pub fn new() -> Self {
        Self
    }
}

impl FileScanner for LocalFileScanner {
    fn scan_tree(&self, root: &Path) -> Result<(Vec<SourceFile>, Vec<String>), ArchiveError> {
        let root = root.canonicalize().map_err(ArchiveError::Io)?;

        let mut files = Vec::new();
        let mut all_dirs: BTreeSet<String> = BTreeSet::new();

        for entry in WalkDir::new(&root).follow_links(false) {
            let entry = entry.map_err(|e| ArchiveError::Engine(e.to_string()))?;
            let path = entry.path();

            if path == root {
                continue;
            }

            let rel = path
                .strip_prefix(&root)
                .map_err(|e| ArchiveError::Engine(e.to_string()))?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");

            if entry.file_type().is_dir() {
                all_dirs.insert(rel_str);
                continue;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let meta = std::fs::metadata(path)?;
            files.push(SourceFile {
                rel_path: rel_str,
                size: meta.len(),
            });
        }

        // Empty directories: no file is under this path (immediate or nested).
        let mut empty = Vec::new();
        for d in &all_dirs {
            let has_file = files.iter().any(|f| {
                f.parent_dir() == d.as_str() || f.rel_path.starts_with(&format!("{d}/"))
            });
            if !has_file {
                empty.push(d.clone());
            }
        }

        files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        empty.sort();
        Ok((files, empty))
    }
}
