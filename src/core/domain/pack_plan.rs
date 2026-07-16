//! Pure packing planner: 1 folder → 1 archive when possible; else by extension.

use super::ArchiveFormat;

/// A source file relative to the pack root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Relative path using `/` separators (e.g. `docs/a.txt`).
    pub rel_path: String,
    pub size: u64,
}

impl SourceFile {
    pub fn parent_dir(&self) -> &str {
        match self.rel_path.rfind('/') {
            Some(i) => &self.rel_path[..i],
            None => "",
        }
    }

    pub fn extension_key(&self) -> String {
        let name = self
            .rel_path
            .rsplit('/')
            .next()
            .unwrap_or(self.rel_path.as_str());
        match name.rfind('.') {
            Some(i) if i > 0 && i + 1 < name.len() => name[i + 1..].to_ascii_lowercase(),
            _ => String::new(),
        }
    }
}

/// One planned archive unit (not yet written).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackUnit {
    /// Directory under which the archive file will be placed (relative).
    pub placement_dir: String,
    pub files: Vec<SourceFile>,
}

impl PackUnit {
    pub fn logical_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

/// Full plan including empty directories to record in the mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackPlan {
    pub units: Vec<PackUnit>,
    pub empty_directories: Vec<String>,
}

/// Build a pack plan from scanned files and empty directories.
///
/// Rules (accepted spec):
/// - Prefer one unit per directory for that directory's *immediate* files.
/// - If immediate files exceed `budget`, group by extension then bin-pack.
/// - A single file may exceed `budget` and still forms one unit (no byte split).
pub fn plan_pack(
    mut files: Vec<SourceFile>,
    empty_directories: Vec<String>,
    budget: u64,
) -> PackPlan {
    let budget = budget.max(1);
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    // Group by parent directory.
    let mut by_dir: Vec<(String, Vec<SourceFile>)> = Vec::new();
    for f in files {
        let dir = f.parent_dir().to_string();
        if let Some(last) = by_dir.last_mut() {
            if last.0 == dir {
                last.1.push(f);
                continue;
            }
        }
        by_dir.push((dir, vec![f]));
    }

    let mut units = Vec::new();
    for (dir, dir_files) in by_dir {
        units.extend(plan_directory(&dir, dir_files, budget));
    }

    let mut empty_directories = empty_directories;
    empty_directories.sort();
    empty_directories.dedup();

    PackPlan {
        units,
        empty_directories,
    }
}

fn plan_directory(dir: &str, files: Vec<SourceFile>, budget: u64) -> Vec<PackUnit> {
    if files.is_empty() {
        return Vec::new();
    }

    let total: u64 = files.iter().map(|f| f.size).sum();
    if total <= budget {
        return vec![PackUnit {
            placement_dir: dir.to_string(),
            files,
        }];
    }

    // Group by extension.
    let mut by_ext: Vec<(String, Vec<SourceFile>)> = Vec::new();
    let mut sorted = files;
    sorted.sort_by(|a, b| {
        a.extension_key()
            .cmp(&b.extension_key())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    for f in sorted {
        let ext = f.extension_key();
        if let Some(last) = by_ext.last_mut() {
            if last.0 == ext {
                last.1.push(f);
                continue;
            }
        }
        by_ext.push((ext, vec![f]));
    }

    let mut units = Vec::new();
    for (_ext, group) in by_ext {
        units.extend(bin_pack(dir, group, budget));
    }
    units
}

/// First-fit packing; a single oversized file always gets its own unit.
fn bin_pack(dir: &str, files: Vec<SourceFile>, budget: u64) -> Vec<PackUnit> {
    let mut units: Vec<PackUnit> = Vec::new();
    let mut current: Vec<SourceFile> = Vec::new();
    let mut current_size: u64 = 0;

    for f in files {
        if f.size >= budget {
            if !current.is_empty() {
                units.push(PackUnit {
                    placement_dir: dir.to_string(),
                    files: std::mem::take(&mut current),
                });
                current_size = 0;
            }
            units.push(PackUnit {
                placement_dir: dir.to_string(),
                files: vec![f],
            });
            continue;
        }

        if !current.is_empty() && current_size + f.size > budget {
            units.push(PackUnit {
                placement_dir: dir.to_string(),
                files: std::mem::take(&mut current),
            });
            current_size = 0;
        }
        current_size += f.size;
        current.push(f);
    }

    if !current.is_empty() {
        units.push(PackUnit {
            placement_dir: dir.to_string(),
            files: current,
        });
    }
    units
}

/// Assign archive relative paths for each unit (scattered hierarchy).
pub fn assign_archive_paths(units: &[PackUnit], format: ArchiveFormat) -> Vec<String> {
    let ext = format.file_extension();
    let mut counters: Vec<(String, u32)> = Vec::new();
    let mut paths = Vec::with_capacity(units.len());

    for unit in units {
        let dir = unit.placement_dir.as_str();
        let n = {
            if let Some(c) = counters.iter_mut().find(|(d, _)| d == dir) {
                c.1 += 1;
                c.1
            } else {
                counters.push((dir.to_string(), 1));
                1
            }
        };
        let name = format!("part-{n:04}.{ext}");
        let path = if dir.is_empty() {
            name
        } else {
            format!("{dir}/{name}")
        };
        paths.push(path);
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(path: &str, size: u64) -> SourceFile {
        SourceFile {
            rel_path: path.to_string(),
            size,
        }
    }

    #[test]
    fn one_folder_one_archive_when_fits() {
        let plan = plan_pack(
            vec![f("docs/a.txt", 100), f("docs/b.txt", 100)],
            vec![],
            1000,
        );
        assert_eq!(plan.units.len(), 1);
        assert_eq!(plan.units[0].placement_dir, "docs");
        assert_eq!(plan.units[0].files.len(), 2);
    }

    #[test]
    fn oversized_folder_splits_by_extension() {
        let plan = plan_pack(
            vec![
                f("photos/a.jpg", 400),
                f("photos/b.jpg", 400),
                f("photos/c.png", 400),
            ],
            vec![],
            1000,
        );
        // total 1200 > 1000 → by ext: jpg(800) one unit, png(400) one unit
        assert_eq!(plan.units.len(), 2);
        let exts: Vec<Vec<String>> = plan
            .units
            .iter()
            .map(|u| u.files.iter().map(|x| x.extension_key()).collect())
            .collect();
        assert!(exts.iter().all(|e| e.windows(2).all(|w| w[0] == w[1])));
        assert!(plan
            .units
            .iter()
            .any(|u| u.files.iter().all(|x| x.extension_key() == "jpg")));
        assert!(plan
            .units
            .iter()
            .any(|u| u.files.iter().all(|x| x.extension_key() == "png")));
    }

    #[test]
    fn huge_file_alone_even_over_budget() {
        let plan = plan_pack(
            vec![f("big/video.mkv", 5000), f("big/note.txt", 10)],
            vec![],
            1000,
        );
        // total 5010 > budget → by ext: mkv alone, txt alone (or txt in own unit)
        assert!(plan
            .units
            .iter()
            .any(|u| u.files.len() == 1 && u.files[0].rel_path == "big/video.mkv"));
        assert!(plan
            .units
            .iter()
            .any(|u| u.files.iter().any(|x| x.rel_path == "big/note.txt")));
    }

    #[test]
    fn empty_dirs_preserved() {
        let plan = plan_pack(vec![f("a.txt", 1)], vec!["empty/nested".into()], 100);
        assert_eq!(plan.empty_directories, vec!["empty/nested"]);
    }

    #[test]
    fn archive_paths_scatter_with_counters() {
        let units = vec![
            PackUnit {
                placement_dir: "docs".into(),
                files: vec![f("docs/a.txt", 1)],
            },
            PackUnit {
                placement_dir: "docs".into(),
                files: vec![f("docs/b.txt", 1)],
            },
            PackUnit {
                placement_dir: String::new(),
                files: vec![f("root.txt", 1)],
            },
        ];
        let paths = assign_archive_paths(&units, ArchiveFormat::Zip);
        assert_eq!(
            paths,
            vec![
                "docs/part-0001.zip",
                "docs/part-0002.zip",
                "part-0001.zip",
            ]
        );
    }
}
