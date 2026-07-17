use crate::core::domain::{ArchiveError, SourceFile};
use crate::core::interrupt;
use crate::core::ports::{FileScanner, ScanProgress};
use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct LocalFileScanner;

impl LocalFileScanner {
    pub fn new() -> Self {
        Self
    }
}

impl FileScanner for LocalFileScanner {
    fn scan_tree_with_progress(
        &self,
        root: &Path,
        on_progress: &dyn Fn(ScanProgress),
    ) -> Result<(Vec<SourceFile>, Vec<String>), ArchiveError> {
        let root = root.canonicalize().map_err(ArchiveError::Io)?;

        let mut files = Vec::new();
        let mut all_dirs: BTreeSet<String> = BTreeSet::new();
        let mut bytes_found: u64 = 0;
        let mut dirs_seen: usize = 0;
        let mut last_report = Instant::now() - Duration::from_secs(1);

        let report = |files_found: usize,
                      bytes_found: u64,
                      dirs_seen: usize,
                      current: &str,
                      force: bool,
                      last_report: &mut Instant| {
            let now = Instant::now();
            if force || now.duration_since(*last_report) >= Duration::from_millis(150) {
                *last_report = now;
                on_progress(ScanProgress {
                    files_found,
                    bytes_found,
                    dirs_seen,
                    current: current.to_string(),
                });
            }
        };

        report(
            0,
            0,
            0,
            root.to_string_lossy().as_ref(),
            true,
            &mut last_report,
        );

        for (walk_i, entry) in WalkDir::new(&root).follow_links(false).into_iter().enumerate()
        {
            if walk_i % 64 == 0 {
                interrupt::check()?;
            }
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
                all_dirs.insert(rel_str.clone());
                dirs_seen += 1;
                report(
                    files.len(),
                    bytes_found,
                    dirs_seen,
                    &rel_str,
                    false,
                    &mut last_report,
                );
                continue;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            let meta = std::fs::metadata(path)?;
            let size = meta.len();
            bytes_found = bytes_found.saturating_add(size);
            files.push(SourceFile {
                rel_path: rel_str.clone(),
                size,
            });
            report(
                files.len(),
                bytes_found,
                dirs_seen,
                &rel_str,
                false,
                &mut last_report,
            );
        }

        report(
            files.len(),
            bytes_found,
            dirs_seen,
            "(walk complete; indexing empty dirs…)",
            true,
            &mut last_report,
        );

        // O(files * depth + dirs): mark every ancestor directory of any file as non-empty.
        let mut non_empty: HashSet<String> = HashSet::new();
        for (i, f) in files.iter().enumerate() {
            let path = f.rel_path.as_str();
            let mut start = 0usize;
            while let Some(rel) = path[start..].find('/') {
                let end = start + rel;
                non_empty.insert(path[..end].to_string());
                start = end + 1;
            }
            if i % 50_000 == 0 {
                report(
                    files.len(),
                    bytes_found,
                    dirs_seen,
                    &format!("(indexing empty dirs… {i}/{})", files.len()),
                    true,
                    &mut last_report,
                );
            }
        }

        let mut empty: Vec<String> = all_dirs
            .into_iter()
            .filter(|d| !non_empty.contains(d))
            .collect();

        report(
            files.len(),
            bytes_found,
            dirs_seen,
            &format!("(sort {} files…)", files.len()),
            true,
            &mut last_report,
        );

        files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        empty.sort();

        report(
            files.len(),
            bytes_found,
            dirs_seen,
            "(scan complete)",
            true,
            &mut last_report,
        );

        Ok((files, empty))
    }
}
