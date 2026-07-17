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

    /// Relative compression efficiency vs zstd (1.0 = zstd baseline).
    ///
    /// Used with content-class base ratios:
    /// `ratio ≈ 1 - (1 - base_zstd) * efficiency`.
    /// Store formats return 0 (no compression).
    pub fn compression_efficiency(self) -> f64 {
        match self {
            Self::Tar => 0.0,
            Self::TarZst => 1.0,
            Self::TarGz => 0.88,
            Self::Zip => 0.82,
        }
    }

    /// Fallback prior when file list is unavailable (mixed-content assumption).
    ///
    /// Prefer [`crate::core::domain::estimate_archive_for_sources`] when paths
    /// and sizes are known. Hard free-space checks still use logical size.
    pub fn prior_archive_ratio(self) -> f64 {
        match self {
            // Store + 512-byte tar block padding / headers.
            Self::Tar => 1.02,
            // Deflate, mixed general content (moderate prior).
            Self::Zip => 0.70,
            // gzip level default-ish mixed content.
            Self::TarGz => 0.65,
            // zstd typically better ratio than gzip on mixed data.
            Self::TarZst => 0.55,
        }
    }

    /// One-line label for logs: `tar.zst prior=0.55 eff=1.00`.
    pub fn prior_ratio_label(self) -> String {
        format!(
            "{} prior={:.2} eff={:.2}",
            self.as_str(),
            self.prior_archive_ratio(),
            self.compression_efficiency()
        )
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

    #[test]
    fn prior_ratios_order_by_algorithm() {
        // store ≈ 1; deflate/gzip mid; zstd typically tightest prior among compressors
        assert!(ArchiveFormat::Tar.prior_archive_ratio() > 1.0);
        assert!(ArchiveFormat::Zip.prior_archive_ratio() < 1.0);
        assert!(ArchiveFormat::TarGz.prior_archive_ratio() < ArchiveFormat::Zip.prior_archive_ratio());
        assert!(
            ArchiveFormat::TarZst.prior_archive_ratio() < ArchiveFormat::TarGz.prior_archive_ratio()
        );
    }

    #[test]
    fn efficiency_order() {
        assert_eq!(ArchiveFormat::Tar.compression_efficiency(), 0.0);
        assert!(
            ArchiveFormat::TarZst.compression_efficiency()
                > ArchiveFormat::TarGz.compression_efficiency()
        );
        assert!(
            ArchiveFormat::TarGz.compression_efficiency()
                > ArchiveFormat::Zip.compression_efficiency()
        );
    }
}
