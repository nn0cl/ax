//! Local disk and memory resource probe.

use crate::adapters::fs::space;
use crate::core::domain::ArchiveError;
use crate::core::ports::{ResourceProbe, VolumeSpace};
use std::path::Path;

#[derive(Debug, Default)]
pub struct LocalResourceProbe;

impl LocalResourceProbe {
    pub fn new() -> Self {
        Self
    }
}

impl ResourceProbe for LocalResourceProbe {
    fn disk_space(&self, path: &Path) -> Result<VolumeSpace, ArchiveError> {
        let (free_bytes, total_bytes) = space::disk_space(path)?;
        Ok(VolumeSpace {
            free_bytes,
            total_bytes,
        })
    }

    fn available_memory_bytes(&self) -> Option<u64> {
        available_memory()
    }
}

#[cfg(target_os = "macos")]
fn available_memory() -> Option<u64> {
    // page_free + page_speculative + page_inactive (approx usable under pressure)
    use std::process::Command;
    let out = Command::new("vm_stat").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let page_size = 16384u64; // Apple Silicon default; fallback parse
    let page_size = parse_page_size(&text).unwrap_or(page_size);

    let mut free = 0u64;
    let mut speculative = 0u64;
    let mut inactive = 0u64;
    for line in text.lines() {
        if let Some(v) = parse_vm_stat_pages(line, "Pages free:") {
            free = v;
        } else if let Some(v) = parse_vm_stat_pages(line, "Pages speculative:") {
            speculative = v;
        } else if let Some(v) = parse_vm_stat_pages(line, "Pages inactive:") {
            inactive = v;
        }
    }
    Some((free + speculative + inactive).saturating_mul(page_size))
}

#[cfg(target_os = "macos")]
fn parse_page_size(text: &str) -> Option<u64> {
    // "Mach Virtual Memory Statistics: (page size of 16384 bytes)"
    let key = "page size of ";
    let i = text.find(key)?;
    let rest = &text[i + key.len()..];
    let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_pages(line: &str, prefix: &str) -> Option<u64> {
    let line = line.trim();
    if !line.starts_with(prefix) {
        return None;
    }
    let rest = line[prefix.len()..].trim().trim_end_matches('.');
    rest.replace(',', "").parse().ok()
}

#[cfg(target_os = "linux")]
fn available_memory() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kb.saturating_mul(1024));
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn available_memory() -> Option<u64> {
    None
}
