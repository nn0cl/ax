use super::error::ArchiveError;
use std::path::{Component, Path, PathBuf};

/// Normalize an archive member path to a relative, non-empty form using `/`.
pub fn normalize_member_path(raw: &str) -> Result<String, ArchiveError> {
    let trimmed = raw.trim().trim_start_matches('/');
    if trimmed.is_empty() || trimmed == "." {
        return Err(ArchiveError::InvalidMemberPath(raw.to_string()));
    }

    let path = Path::new(trimmed);
    let mut parts: Vec<String> = Vec::new();

    for component in path.components() {
        match component {
            Component::Normal(os) => {
                let s = os.to_string_lossy();
                if s == ".." {
                    return Err(ArchiveError::PathTraversal(raw.to_string()));
                }
                parts.push(s.into_owned());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(ArchiveError::PathTraversal(raw.to_string()));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(ArchiveError::PathTraversal(raw.to_string()));
            }
        }
    }

    if parts.is_empty() {
        return Err(ArchiveError::InvalidMemberPath(raw.to_string()));
    }

    Ok(parts.join("/"))
}

/// Returns true when the member path is safe to extract under a destination root.
pub fn is_safe_member_path(raw: &str) -> bool {
    normalize_member_path(raw).is_ok()
}

/// Join destination root with a validated member path. Rejects traversal.
pub fn safe_join(dest: &Path, member: &str) -> Result<PathBuf, ArchiveError> {
    let normalized = normalize_member_path(member)?;
    let joined = dest.join(Path::new(&normalized));

    // Extra defense: ensure the resolved path stays under dest when both exist
    // as absolute-capable paths. We compare components rather than canonicalize
    // so missing parents still work during extract.
    let dest_components: Vec<_> = dest.components().collect();
    let joined_components: Vec<_> = joined.components().collect();
    if joined_components.len() < dest_components.len()
        || joined_components[..dest_components.len()] != dest_components[..]
    {
        return Err(ArchiveError::PathTraversal(member.to_string()));
    }

    Ok(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_relative_paths() {
        assert_eq!(normalize_member_path("a.txt").unwrap(), "a.txt");
        assert_eq!(normalize_member_path("dir/b.txt").unwrap(), "dir/b.txt");
        assert_eq!(normalize_member_path("/dir/b.txt").unwrap(), "dir/b.txt");
        assert_eq!(normalize_member_path("./x").unwrap(), "x");
    }

    #[test]
    fn rejects_traversal() {
        assert!(matches!(
            normalize_member_path("../escape.txt"),
            Err(ArchiveError::PathTraversal(_))
        ));
        assert!(matches!(
            normalize_member_path("foo/../../etc/passwd"),
            Err(ArchiveError::PathTraversal(_))
        ));
        assert!(!is_safe_member_path("../x"));
    }

    #[test]
    fn safe_join_stays_under_dest() {
        let dest = PathBuf::from("/tmp/out");
        let joined = safe_join(&dest, "dir/file.txt").unwrap();
        assert_eq!(joined, PathBuf::from("/tmp/out/dir/file.txt"));
        assert!(safe_join(&dest, "../escape.txt").is_err());
    }
}
