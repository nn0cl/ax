use crate::core::domain::{normalize_member_path, safe_join, ArchiveError, ArchiveFormat};
use crate::core::ports::{ArchiveEngine, ContentHasher, FileSystem, MappingStore};
use std::path::{Path, PathBuf};

/// Input for restore-from-mapping.
#[derive(Debug, Clone)]
pub struct RestoreRequest {
    pub mapping_path: PathBuf,
    pub dest: PathBuf,
    /// Root that archive_relpath entries are resolved against (usually pack output).
    pub archives_root: PathBuf,
    /// When true, verify SHA-256 of restored members against the mapping.
    pub verify: bool,
}

/// Use case: restore original tree from JSON mapping + scattered archives.
pub struct RestoreFromMapping<'a, E, M, F, H>
where
    E: ArchiveEngine,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
{
    engine: &'a E,
    mapping: &'a M,
    fs: &'a F,
    hasher: &'a H,
}

impl<'a, E, M, F, H> RestoreFromMapping<'a, E, M, F, H>
where
    E: ArchiveEngine,
    M: MappingStore,
    F: FileSystem,
    H: ContentHasher,
{
    pub fn new(engine: &'a E, mapping: &'a M, fs: &'a F, hasher: &'a H) -> Self {
        Self {
            engine,
            mapping,
            fs,
            hasher,
        }
    }

    pub fn execute(&self, req: &RestoreRequest) -> Result<(), ArchiveError> {
        let manifest = self.mapping.read(&req.mapping_path)?;
        let format = ArchiveFormat::parse(&manifest.format)?;

        self.fs.create_dir_all(&req.dest)?;

        for empty in &manifest.empty_directories {
            let normalized = normalize_member_path(empty)?;
            let path = safe_join(&req.dest, &normalized)?;
            self.fs.create_dir_all(&path)?;
        }

        for archive in &manifest.archives {
            let archive_path = req.archives_root.join(&archive.archive_relpath);
            if !self.fs.exists(&archive_path) {
                return Err(ArchiveError::ArchiveNotFound(
                    archive_path.display().to_string(),
                ));
            }
            for member in &archive.members {
                let _ = safe_join(&req.dest, &member.source_relpath)?;
            }
            self.engine
                .extract(format, &archive_path, &req.dest)?;
        }

        if req.verify {
            for archive in &manifest.archives {
                for member in &archive.members {
                    if member.sha256.is_empty() {
                        return Err(ArchiveError::Mapping(format!(
                            "member missing sha256 (required): {}",
                            member.source_relpath
                        )));
                    }
                    let path = safe_join(&req.dest, &member.source_relpath)?;
                    let actual = self.hasher.hash_file(&path)?;
                    if actual != member.sha256 {
                        return Err(ArchiveError::HashMismatch {
                            path: member.source_relpath.clone(),
                            expected: member.sha256.clone(),
                            actual,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

/// When archives_root is omitted, use the mapping file's parent directory.
pub fn default_archives_root(mapping_path: &Path) -> PathBuf {
    mapping_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}
