use super::error::ArchiveError;
use std::path::Path;

/// Supported archive formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    /// TAR + zstd (multi-threaded compression when the adapter enables it).
    TarZst,
}

impl ArchiveFormat {
    pub fn parse(name: &str) -> Result<Self, ArchiveError> {
        match name.to_ascii_lowercase().as_str() {
            "zip" => Ok(Self::Zip),
            "tar" => Ok(Self::Tar),
            "tar.gz" | "tgz" | "targz" => Ok(Self::TarGz),
            "tar.zst" | "tzst" | "tar.zstd" | "zst" => Ok(Self::TarZst),
            _ => Err(ArchiveError::UnsupportedFormat),
        }
    }

    pub fn from_path(path: &Path) -> Result<Self, ArchiveError> {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        if name.ends_with(".tar.zst") || name.ends_with(".tzst") || name.ends_with(".tar.zstd") {
            return Ok(Self::TarZst);
        }
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            return Ok(Self::TarGz);
        }
        if name.ends_with(".tar") {
            return Ok(Self::Tar);
        }
        if name.ends_with(".zip") {
            return Ok(Self::Zip);
        }
        Err(ArchiveError::FormatUndetermined)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
            Self::TarZst => "tar.zst",
        }
    }

    pub fn file_extension(self) -> &'static str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_format_names() {
        assert_eq!(ArchiveFormat::parse("zip").unwrap(), ArchiveFormat::Zip);
        assert_eq!(ArchiveFormat::parse("TAR").unwrap(), ArchiveFormat::Tar);
        assert_eq!(ArchiveFormat::parse("tgz").unwrap(), ArchiveFormat::TarGz);
        assert_eq!(ArchiveFormat::parse("tar.zst").unwrap(), ArchiveFormat::TarZst);
        assert!(ArchiveFormat::parse("7z").is_err());
    }

    #[test]
    fn infers_from_extension() {
        assert_eq!(
            ArchiveFormat::from_path(&PathBuf::from("a.ZIP")).unwrap(),
            ArchiveFormat::Zip
        );
        assert_eq!(
            ArchiveFormat::from_path(&PathBuf::from("bundle.tar.gz")).unwrap(),
            ArchiveFormat::TarGz
        );
        assert_eq!(
            ArchiveFormat::from_path(&PathBuf::from("x.tar.zst")).unwrap(),
            ArchiveFormat::TarZst
        );
        assert!(ArchiveFormat::from_path(&PathBuf::from("noext")).is_err());
    }
}
