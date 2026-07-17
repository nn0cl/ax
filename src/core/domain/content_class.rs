//! Content-class compression priors (extension → class → expected ratio).
//!
//! Combined with [`ArchiveFormat`] algorithm efficiency to estimate archive size
//! before any unit is transferred. Not a guarantee; empirical ratio wins after
//! the first finished transfer.

use super::format::ArchiveFormat;
use super::pack_plan::SourceFile;
use super::human_bytes;
use std::collections::BTreeMap;

/// Coarse content class for compressibility priors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentClass {
    /// Plain prose / logs (highly compressible).
    Text,
    /// Source code (highly compressible).
    SourceCode,
    /// JSON / XML / YAML / CSV (highly compressible).
    StructuredText,
    /// Office Open XML (already zip) / similar.
    OfficeContainer,
    /// PDF (mixed; often modest gains).
    Pdf,
    /// Lossy images (JPEG, WebP, HEIC) — near incompressible.
    ImageLossy,
    /// PNG/GIF (already compressed) / BMP (more compressible).
    ImageBitmap,
    /// Lossy/compressed audio.
    Audio,
    /// Video containers (usually already compressed).
    Video,
    /// Archives and compressed blobs.
    AlreadyArchived,
    /// Executables / libraries (moderate).
    Binary,
    /// No extension or unknown type.
    Unknown,
}

impl ContentClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::SourceCode => "source",
            Self::StructuredText => "structured",
            Self::OfficeContainer => "office",
            Self::Pdf => "pdf",
            Self::ImageLossy => "image_lossy",
            Self::ImageBitmap => "image_bitmap",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::AlreadyArchived => "archived",
            Self::Binary => "binary",
            Self::Unknown => "unknown",
        }
    }

    /// Expected `archive/logical` under a strong general compressor (zstd baseline).
    pub fn base_ratio_zstd(self) -> f64 {
        match self {
            Self::Text => 0.28,
            Self::SourceCode => 0.32,
            Self::StructuredText => 0.22,
            Self::OfficeContainer => 0.98, // already deflate/zip
            Self::Pdf => 0.90,
            Self::ImageLossy => 0.99,
            Self::ImageBitmap => 0.92, // PNG≈0.95–1.0, BMP lower in aggregate
            Self::Audio => 0.98,
            Self::Video => 0.995,
            Self::AlreadyArchived => 1.00,
            Self::Binary => 0.75,
            Self::Unknown => 0.70, // mixed prior
        }
    }
}

/// Classify by file extension (lowercase, no dot). Empty → Unknown.
pub fn classify_extension(ext: &str) -> ContentClass {
    let e = ext.trim().trim_start_matches('.').to_ascii_lowercase();
    if e.is_empty() {
        return ContentClass::Unknown;
    }
    // multi-part: tar.gz handled via full name in classify_path
    match e.as_str() {
        // text
        "txt" | "text" | "md" | "markdown" | "rst" | "log" | "csv" | "tsv" | "rtf" | "tex"
        | "ltx" | "org" | "adoc" | "asciidoc" => ContentClass::Text,

        // structured
        "json" | "jsonl" | "ndjson" | "xml" | "html" | "htm" | "xhtml" | "svg" | "css" | "scss"
        | "less" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "properties" | "plist"
        | "sql" | "graphql" | "proto" | "avsc" => ContentClass::StructuredText,

        // source
        "rs" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "m" | "mm" | "swift" | "go"
        | "py" | "pyi" | "rb" | "php" | "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "java"
        | "kt" | "kts" | "scala" | "cs" | "fs" | "vb" | "r" | "jl" | "lua" | "pl" | "pm"
        | "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" | "bat" | "cmd" | "cmake" | "mk"
        | "makefile" | "gradle" | "sbt" | "vim" | "el" | "lisp" | "clj" | "ex" | "exs"
        | "erl" | "hrl" | "hs" | "ml" | "mli" | "dart" | "nim" | "zig" | "v" | "sv" | "vhd"
        | "vhdl" | "asm" | "s" => ContentClass::SourceCode,

        // office containers (OOXML = zip)
        "docx" | "xlsx" | "pptx" | "odt" | "ods" | "odp" | "doc" | "xls" | "ppt" | "pages"
        | "numbers" | "key" => ContentClass::OfficeContainer,

        "pdf" => ContentClass::Pdf,

        // lossy / modern compressed images
        "jpg" | "jpeg" | "jpe" | "jfif" | "webp" | "heic" | "heif" | "avif" | "jxl" => {
            ContentClass::ImageLossy
        }

        // bitmap / lossless-ish containers
        "png" | "gif" | "bmp" | "dib" | "tif" | "tiff" | "ico" | "icns" | "psd" | "xcf" => {
            ContentClass::ImageBitmap
        }

        // audio
        "mp3" | "aac" | "m4a" | "ogg" | "oga" | "opus" | "wma" | "flac" | "wav" | "aiff"
        | "aif" | "alac" => ContentClass::Audio,

        // video
        // note: `.ts` is TypeScript (SourceCode), not MPEG-TS
        "mp4" | "m4v" | "mov" | "mkv" | "webm" | "avi" | "wmv" | "flv" | "mpeg" | "mpg"
        | "m2ts" | "mts" | "3gp" => ContentClass::Video,

        // already archived / compressed
        "zip" | "gz" | "tgz" | "bz2" | "xz" | "lz" | "lz4" | "zst" | "zstd" | "7z" | "rar"
        | "cab" | "iso" | "dmg" | "pkg" | "deb" | "rpm" | "apk" | "ipa" | "jar" | "war"
        | "ear" | "whl" | "nupkg" | "vsix" => ContentClass::AlreadyArchived,

        // binary
        "exe" | "dll" | "so" | "dylib" | "a" | "lib" | "o" | "obj" | "class" | "pyc" | "pyo"
        | "wasm" | "bin" | "dat" | "db" | "sqlite" | "sqlite3" => ContentClass::Binary,

        _ => ContentClass::Unknown,
    }
}

/// Classify from a relative path (uses last extension; treats `.tar.gz` etc. as archived).
pub fn classify_path(rel_path: &str) -> ContentClass {
    let name = rel_path
        .rsplit('/')
        .next()
        .unwrap_or(rel_path)
        .to_ascii_lowercase();
    if name.ends_with(".tar.gz")
        || name.ends_with(".tar.bz2")
        || name.ends_with(".tar.xz")
        || name.ends_with(".tar.zst")
        || name.ends_with(".tar.zstd")
    {
        return ContentClass::AlreadyArchived;
    }
    let ext = match name.rfind('.') {
        Some(i) if i > 0 && i + 1 < name.len() => &name[i + 1..],
        _ => "",
    };
    classify_extension(ext)
}

/// Per-class contribution in a content-aware estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassBreakdown {
    pub class: ContentClass,
    pub logical_bytes: u64,
    pub estimated_archive_bytes: u64,
    pub file_count: usize,
}

/// Result of algorithm + content-class size estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentAwareEstimate {
    pub logical_bytes: u64,
    pub estimated_archive_bytes: u64,
    /// `estimated_archive / logical` (1.0 if empty).
    pub ratio: f64,
    pub format: ArchiveFormat,
    pub by_class: Vec<ClassBreakdown>,
}

impl ContentAwareEstimate {
    /// Compact log line (no trailing newline).
    pub fn summary_line(&self) -> String {
        let mut parts = Vec::new();
        for b in &self.by_class {
            if b.logical_bytes == 0 {
                continue;
            }
            let pct = if self.logical_bytes > 0 {
                100.0 * b.logical_bytes as f64 / self.logical_bytes as f64
            } else {
                0.0
            };
            parts.push(format!(
                "{}={:.0}%:{}→{}",
                b.class.as_str(),
                pct,
                human_bytes(b.logical_bytes),
                human_bytes(b.estimated_archive_bytes),
            ));
        }
        let classes = if parts.is_empty() {
            "none".into()
        } else {
            parts.join(" ")
        };
        format!(
            "size_estimate: format={} algo_eff={:.2} logical={} ({}) archive≈{} ({}) \
             ratio={:.3} classes=[{}] (content+algo prior until first transfer; not a guarantee)",
            self.format.as_str(),
            self.format.compression_efficiency(),
            self.logical_bytes,
            human_bytes(self.logical_bytes),
            self.estimated_archive_bytes,
            human_bytes(self.estimated_archive_bytes),
            self.ratio,
            classes,
        )
    }
}

/// Estimate archive size from source files (path + size).
pub fn estimate_archive_for_sources(
    files: impl IntoIterator<Item = (impl AsRef<str>, u64)>,
    format: ArchiveFormat,
) -> ContentAwareEstimate {
    let mut logical_by: BTreeMap<ContentClass, u64> = BTreeMap::new();
    let mut count_by: BTreeMap<ContentClass, usize> = BTreeMap::new();
    let mut logical_total: u64 = 0;

    for (path, size) in files {
        let class = classify_path(path.as_ref());
        *logical_by.entry(class).or_default() += size;
        *count_by.entry(class).or_default() += 1;
        logical_total = logical_total.saturating_add(size);
    }

    finish_estimate(logical_by, count_by, logical_total, format)
}

/// Estimate from [`SourceFile`] slice.
pub fn estimate_archive_for_source_files(
    files: &[SourceFile],
    format: ArchiveFormat,
) -> ContentAwareEstimate {
    estimate_archive_for_sources(
        files.iter().map(|f| (f.rel_path.as_str(), f.size)),
        format,
    )
}

/// Combined ratio for one content class under `format`.
///
/// Model: residual compressibility under zstd scaled by algorithm efficiency:
/// `ratio = 1 - (1 - base_zstd) * efficiency` (+ small container overhead for compressed formats).
pub fn combined_ratio(class: ContentClass, format: ArchiveFormat) -> f64 {
    match format {
        ArchiveFormat::Tar => 1.02,
        f => {
            let base = class.base_ratio_zstd().clamp(0.05, 1.0);
            let eff = f.compression_efficiency().clamp(0.0, 1.0);
            let residual = (1.0 - base).max(0.0);
            let r = 1.0 - residual * eff;
            // modest container / block overhead
            (r + 0.005).clamp(0.05, 1.10)
        }
    }
}

fn finish_estimate(
    logical_by: BTreeMap<ContentClass, u64>,
    count_by: BTreeMap<ContentClass, usize>,
    logical_total: u64,
    format: ArchiveFormat,
) -> ContentAwareEstimate {
    let mut by_class = Vec::new();
    let mut est_total_f = 0.0_f64;

    for (class, logical) in logical_by {
        let r = combined_ratio(class, format);
        let est_f = logical as f64 * r;
        est_total_f += est_f;
        by_class.push(ClassBreakdown {
            class,
            logical_bytes: logical,
            estimated_archive_bytes: scale_bytes(logical, r),
            file_count: count_by.get(&class).copied().unwrap_or(0),
        });
    }

    let ratio = if logical_total > 0 {
        est_total_f / logical_total as f64
    } else {
        1.0
    };
    let est_total = if est_total_f.is_finite() && est_total_f > 0.0 {
        est_total_f.ceil() as u64
    } else {
        logical_total
    };

    ContentAwareEstimate {
        logical_bytes: logical_total,
        estimated_archive_bytes: est_total,
        ratio: sanitize_ratio(ratio),
        format,
        by_class,
    }
}

fn scale_bytes(logical: u64, ratio: f64) -> u64 {
    let v = logical as f64 * ratio;
    if v.is_finite() && v > 0.0 {
        v.ceil() as u64
    } else {
        logical
    }
}

fn sanitize_ratio(r: f64) -> f64 {
    if r.is_finite() && r > 0.0 {
        r.clamp(0.05, 2.0)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_extensions() {
        assert_eq!(classify_extension("rs"), ContentClass::SourceCode);
        assert_eq!(classify_extension("JPG"), ContentClass::ImageLossy);
        assert_eq!(classify_extension("mp4"), ContentClass::Video);
        assert_eq!(classify_extension("zip"), ContentClass::AlreadyArchived);
        assert_eq!(classify_path("a/b/photo.JPEG"), ContentClass::ImageLossy);
        assert_eq!(
            classify_path("pkg/release.tar.gz"),
            ContentClass::AlreadyArchived
        );
    }

    #[test]
    fn text_compresses_better_than_video_under_zstd() {
        let text = combined_ratio(ContentClass::Text, ArchiveFormat::TarZst);
        let video = combined_ratio(ContentClass::Video, ArchiveFormat::TarZst);
        assert!(text < 0.4);
        assert!(video > 0.95);
        assert!(text < video);
    }

    #[test]
    fn tar_store_ignores_content_class() {
        let t = combined_ratio(ContentClass::Text, ArchiveFormat::Tar);
        let v = combined_ratio(ContentClass::Video, ArchiveFormat::Tar);
        assert!((t - 1.02).abs() < 1e-9);
        assert!((v - 1.02).abs() < 1e-9);
    }

    #[test]
    fn zstd_beats_gzip_on_text_residual() {
        let z = combined_ratio(ContentClass::Text, ArchiveFormat::TarZst);
        let g = combined_ratio(ContentClass::Text, ArchiveFormat::TarGz);
        assert!(z < g);
    }

    #[test]
    fn mixed_corpus_estimate() {
        let files = [
            ("src/main.rs", 1000u64),
            ("photo.jpg", 9000u64),
            ("notes.txt", 1000u64),
        ];
        let est = estimate_archive_for_sources(files, ArchiveFormat::TarZst);
        assert_eq!(est.logical_bytes, 11_000);
        // mostly jpeg → ratio closer to 1 than pure text
        assert!(est.ratio > 0.7);
        assert!(est.ratio < 1.0);
        assert!(est.estimated_archive_bytes < est.logical_bytes);
        assert!(est.by_class.len() >= 2);
        let line = est.summary_line();
        assert!(line.contains("size_estimate:"));
        assert!(line.contains("image_lossy") || line.contains("source"));
    }

    #[test]
    fn pure_media_near_unity() {
        let files = [("a.mp4", 1_000_000u64), ("b.jpg", 500_000u64)];
        let est = estimate_archive_for_sources(files, ArchiveFormat::TarZst);
        assert!(est.ratio > 0.97);
    }
}
