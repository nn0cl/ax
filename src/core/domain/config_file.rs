//! Simple INI / .inf style config (no external dependency).
//!
//! ```ini
//! [pack]
//! verify_stage = true
//! jobs = 4
//! format = tar.zst
//! work_dir = /tmp/archiver-work
//! ```

use super::ArchiveError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Parsed configuration (section → key → value).
#[derive(Debug, Clone, Default)]
pub struct IniConfig {
    /// Lowercase section name → keys (lowercase) → raw value string.
    pub sections: HashMap<String, HashMap<String, String>>,
}

impl IniConfig {
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(&section.to_ascii_lowercase())
            .and_then(|m| m.get(&key.to_ascii_lowercase()))
            .map(|s| s.as_str())
    }

    pub fn get_bool(&self, section: &str, key: &str) -> Result<Option<bool>, ArchiveError> {
        match self.get(section, key) {
            None => Ok(None),
            Some(v) => parse_bool(v).map(Some),
        }
    }

    pub fn get_u64(&self, section: &str, key: &str) -> Result<Option<u64>, ArchiveError> {
        match self.get(section, key) {
            None => Ok(None),
            Some(v) => v
                .parse()
                .map(Some)
                .map_err(|_| ArchiveError::InvalidSize(format!("config {section}.{key}: {v}"))),
        }
    }

    pub fn get_usize(&self, section: &str, key: &str) -> Result<Option<usize>, ArchiveError> {
        match self.get_u64(section, key)? {
            None => Ok(None),
            Some(v) => usize::try_from(v)
                .map(Some)
                .map_err(|_| ArchiveError::InvalidSize(format!("config {section}.{key} too large"))),
        }
    }

    pub fn get_f64(&self, section: &str, key: &str) -> Result<Option<f64>, ArchiveError> {
        match self.get(section, key) {
            None => Ok(None),
            Some(v) => v.parse().map(Some).map_err(|_| {
                ArchiveError::InvalidSize(format!("config {section}.{key}: not a number: {v}"))
            }),
        }
    }

    pub fn get_path(&self, section: &str, key: &str) -> Option<PathBuf> {
        self.get(section, key).map(PathBuf::from)
    }
}

/// Load `.inf` / `.ini` / key=value config from disk.
pub fn load_ini_file(path: &Path) -> Result<IniConfig, ArchiveError> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        ArchiveError::Mapping(format!("config {}: {e}", path.display()))
    })?;
    parse_ini(&text)
}

/// Parse INI text. Supports `[section]`, `key=value`, `#` / `;` comments, blank lines.
pub fn parse_ini(text: &str) -> Result<IniConfig, ArchiveError> {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current = "default".to_string();
    sections.insert(current.clone(), HashMap::new());

    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let name = line[1..line.len() - 1].trim().to_ascii_lowercase();
            if name.is_empty() {
                return Err(ArchiveError::Mapping(format!(
                    "config line {}: empty section name",
                    lineno + 1
                )));
            }
            current = name;
            sections.entry(current.clone()).or_default();
            continue;
        }
        let (k, v) = split_kv(line).ok_or_else(|| {
            ArchiveError::Mapping(format!(
                "config line {}: expected key=value, got {raw:?}",
                lineno + 1
            ))
        })?;
        sections
            .entry(current.clone())
            .or_default()
            .insert(k.to_ascii_lowercase(), v.to_string());
    }

    Ok(IniConfig { sections })
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    let eq = line.find('=')?;
    let k = line[..eq].trim();
    let v = line[eq + 1..].trim();
    // strip optional quotes
    let v = v
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| v.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(v);
    if k.is_empty() {
        None
    } else {
        Some((k, v))
    }
}

fn parse_bool(v: &str) -> Result<bool, ArchiveError> {
    match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(ArchiveError::InvalidSize(format!(
            "expected bool, got {v:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pack_section() {
        let c = parse_ini(
            r#"
# comment
[pack]
verify_stage = true
jobs = 4
format = tar.zst
work_dir = /tmp/w
"#,
        )
        .unwrap();
        assert_eq!(c.get_bool("pack", "verify_stage").unwrap(), Some(true));
        assert_eq!(c.get_usize("pack", "jobs").unwrap(), Some(4));
        assert_eq!(c.get("pack", "format"), Some("tar.zst"));
        assert_eq!(
            c.get_path("pack", "work_dir"),
            Some(PathBuf::from("/tmp/w"))
        );
    }
}
