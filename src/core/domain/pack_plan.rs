//! Pure packing planner: folder-atomic bundles, multi-folder merge up to budget.

use super::ArchiveFormat;
use std::collections::HashMap;

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
    /// For multi-folder units this is the longest common parent.
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

/// Immediate files of one parent directory (may be split if over budget).
#[derive(Debug, Clone)]
struct FolderBundle {
    dir: String,
    files: Vec<SourceFile>,
    size: u64,
}

/// Progress while planning (bundles_done, bundles_total, units_so_far).
pub type PlanProgressFn<'a> = dyn Fn(usize, usize, usize) + 'a;

/// Build a pack plan from scanned files and empty directories.
///
/// Rules:
/// - Multiple folders may be merged into one archive while under `budget`
///   (logical size; compressed archives are usually smaller).
/// - A folder **over** `budget` is split by whole files (greedy) so each unit
///   stays ≤ budget when possible.
/// - A **single file** larger than `budget` is never byte-split (one unit alone).
/// - Under-budget folders are not split (siblings stay together when they fit).
pub fn plan_pack(
    files: Vec<SourceFile>,
    empty_directories: Vec<String>,
    budget: u64,
) -> PackPlan {
    plan_pack_with_progress(files, empty_directories, budget, &|_, _, _| {})
}

/// Same as [`plan_pack`], with progress: (bundles_done, bundles_total, units_so_far).
pub fn plan_pack_with_progress(
    mut files: Vec<SourceFile>,
    empty_directories: Vec<String>,
    budget: u64,
    on_progress: &PlanProgressFn<'_>,
) -> PackPlan {
    let budget = budget.max(1);
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

    // Group by parent directory (path order).
    let mut bundles: Vec<FolderBundle> = Vec::new();
    for f in files {
        let dir = f.parent_dir().to_string();
        if let Some(last) = bundles.last_mut() {
            if last.dir == dir {
                last.size = last.size.saturating_add(f.size);
                last.files.push(f);
                continue;
            }
        }
        let size = f.size;
        bundles.push(FolderBundle {
            dir,
            files: vec![f],
            size,
        });
    }

    let bundles_total = bundles.len();
    let mut units: Vec<PackUnit> = Vec::new();
    let mut current: Vec<FolderBundle> = Vec::new();
    let mut current_size: u64 = 0;

    let flush = |current: &mut Vec<FolderBundle>,
                 current_size: &mut u64,
                 units: &mut Vec<PackUnit>| {
        if current.is_empty() {
            return;
        }
        let dirs: Vec<String> = current.iter().map(|b| b.dir.clone()).collect();
        let placement_dir = common_parent_dir(&dirs);
        let mut files = Vec::new();
        for b in current.drain(..) {
            files.extend(b.files);
        }
        *current_size = 0;
        units.push(PackUnit {
            placement_dir,
            files,
        });
    };

    for (i, bundle) in bundles.into_iter().enumerate() {
        if bundle.size > budget {
            // Over budget: flush merge queue, then split this folder by whole files.
            flush(&mut current, &mut current_size, &mut units);
            emit_split_folder_units(bundle, budget, &mut units);
        } else if !current.is_empty() && current_size.saturating_add(bundle.size) > budget {
            flush(&mut current, &mut current_size, &mut units);
            current_size = bundle.size;
            current.push(bundle);
        } else {
            current_size = current_size.saturating_add(bundle.size);
            current.push(bundle);
        }

        if i % 256 == 0 || i + 1 == bundles_total {
            on_progress(i + 1, bundles_total, units.len() + usize::from(!current.is_empty()));
        }
    }
    flush(&mut current, &mut current_size, &mut units);
    on_progress(bundles_total, bundles_total, units.len());

    let mut empty_directories = empty_directories;
    empty_directories.sort();
    empty_directories.dedup();

    PackPlan {
        units,
        empty_directories,
    }
}

/// Split one over-budget directory into whole-file units (no byte-split of a file).
fn emit_split_folder_units(bundle: FolderBundle, budget: u64, units: &mut Vec<PackUnit>) {
    let mut files = bundle.files;
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    let dir = bundle.dir;

    let mut batch: Vec<SourceFile> = Vec::new();
    let mut batch_size: u64 = 0;

    let flush_batch = |batch: &mut Vec<SourceFile>,
                       batch_size: &mut u64,
                       dir: &str,
                       units: &mut Vec<PackUnit>| {
        if batch.is_empty() {
            return;
        }
        units.push(PackUnit {
            placement_dir: dir.to_string(),
            files: std::mem::take(batch),
        });
        *batch_size = 0;
    };

    for f in files {
        if f.size > budget {
            flush_batch(&mut batch, &mut batch_size, &dir, units);
            units.push(PackUnit {
                placement_dir: dir.clone(),
                files: vec![f],
            });
        } else if !batch.is_empty() && batch_size.saturating_add(f.size) > budget {
            flush_batch(&mut batch, &mut batch_size, &dir, units);
            batch_size = f.size;
            batch.push(f);
        } else {
            batch_size = batch_size.saturating_add(f.size);
            batch.push(f);
        }
    }
    flush_batch(&mut batch, &mut batch_size, &dir, units);
}

/// Longest common parent path of directory relative paths (`/` separated).
fn common_parent_dir(dirs: &[String]) -> String {
    if dirs.is_empty() {
        return String::new();
    }
    if dirs.len() == 1 {
        return dirs[0].clone();
    }

    let split: Vec<Vec<&str>> = dirs
        .iter()
        .map(|d| {
            if d.is_empty() {
                Vec::new()
            } else {
                d.split('/').filter(|s| !s.is_empty()).collect()
            }
        })
        .collect();

    let mut prefix: Vec<&str> = Vec::new();
    let min_len = split.iter().map(|p| p.len()).min().unwrap_or(0);
    for i in 0..min_len {
        let part = split[0][i];
        if split.iter().all(|p| p[i] == part) {
            prefix.push(part);
        } else {
            break;
        }
    }
    prefix.join("/")
}

/// Assign archive relative paths for each unit (scattered hierarchy).
/// Uses HashMap counters so cost is O(units), not O(units²).
pub fn assign_archive_paths(units: &[PackUnit], format: ArchiveFormat) -> Vec<String> {
    let ext = format.file_extension();
    let mut counters: HashMap<&str, u32> = HashMap::new();
    let mut paths = Vec::with_capacity(units.len());

    for unit in units {
        let dir = unit.placement_dir.as_str();
        let n = {
            let entry = counters.entry(dir).or_insert(0);
            *entry += 1;
            *entry
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
    fn one_folder_stays_together() {
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
    fn folder_over_budget_splits_by_whole_files() {
        // Same folder, total 1200 > budget 1000 → 800 + 400 (no byte-split).
        let plan = plan_pack(
            vec![
                f("photos/a.jpg", 400),
                f("photos/b.jpg", 400),
                f("photos/c.png", 400),
            ],
            vec![],
            1000,
        );
        assert_eq!(plan.units.len(), 2);
        assert_eq!(plan.units[0].files.len(), 2);
        assert_eq!(plan.units[1].files.len(), 1);
        assert!(plan
            .units
            .iter()
            .all(|u| u.files.iter().all(|x| x.parent_dir() == "photos")));
    }

    #[test]
    fn multiple_folders_merge_under_budget() {
        // three sibling folders 300 each, budget 1000 → one archive
        let plan = plan_pack(
            vec![
                f("a/x.txt", 300),
                f("b/y.txt", 300),
                f("c/z.txt", 300),
            ],
            vec![],
            1000,
        );
        assert_eq!(plan.units.len(), 1);
        assert_eq!(plan.units[0].files.len(), 3);
        // common parent of a,b,c is root
        assert_eq!(plan.units[0].placement_dir, "");
    }

    #[test]
    fn merge_stops_before_exceeding_budget() {
        // 400+400+400 with budget 1000 → first two together (800), third alone
        let plan = plan_pack(
            vec![
                f("a/x.txt", 400),
                f("b/y.txt", 400),
                f("c/z.txt", 400),
            ],
            vec![],
            1000,
        );
        assert_eq!(plan.units.len(), 2);
        assert_eq!(plan.units[0].files.len(), 2);
        assert_eq!(plan.units[1].files.len(), 1);
    }

    #[test]
    fn nested_common_parent_placement() {
        let plan = plan_pack(
            vec![
                f("docs/a/x.txt", 100),
                f("docs/b/y.txt", 100),
            ],
            vec![],
            1000,
        );
        assert_eq!(plan.units.len(), 1);
        assert_eq!(plan.units[0].placement_dir, "docs");
    }

    #[test]
    fn huge_file_alone_small_sibling_separate() {
        let plan = plan_pack(
            vec![f("big/video.mkv", 5000), f("big/note.txt", 10)],
            vec![],
            1000,
        );
        // File > budget is never byte-split; small sibling is its own unit.
        // Path order: note.txt first (fits alone), then video.mkv alone (over budget).
        assert_eq!(plan.units.len(), 2);
        assert_eq!(plan.units[0].files[0].rel_path, "big/note.txt");
        assert_eq!(plan.units[1].files[0].rel_path, "big/video.mkv");
        assert_eq!(plan.units[1].files.len(), 1);
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

    #[test]
    fn assign_paths_many_dirs_is_linear() {
        let units: Vec<PackUnit> = (0..5_000)
            .map(|i| PackUnit {
                placement_dir: format!("d{i}"),
                files: vec![f(&format!("d{i}/f.txt"), 1)],
            })
            .collect();
        let paths = assign_archive_paths(&units, ArchiveFormat::Zip);
        assert_eq!(paths.len(), 5_000);
        assert_eq!(paths[0], "d0/part-0001.zip");
        assert_eq!(paths[4999], "d4999/part-0001.zip");
    }

    #[test]
    fn common_parent_helpers() {
        assert_eq!(
            common_parent_dir(&["docs/a".into(), "docs/b".into()]),
            "docs"
        );
        assert_eq!(common_parent_dir(&["a".into(), "b".into()]), "");
        assert_eq!(common_parent_dir(&["only".into()]), "only");
    }
}
