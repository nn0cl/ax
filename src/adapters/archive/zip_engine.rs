use crate::adapters::fs::{collect_pack_entries, ensure_parent_dir};
use crate::core::domain::{safe_join, ArchiveError, ArchiveMember, MemberSpec};
use crate::core::interrupt;
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub fn create_zip(output: &Path, inputs: &[PathBuf]) -> Result<(), ArchiveError> {
    let entries = collect_pack_entries(inputs)?;
    let members: Vec<MemberSpec> = entries
        .into_iter()
        .map(|e| MemberSpec {
            source: e.source,
            member_name: e.member_name,
        })
        .collect();
    create_zip_members(output, &members)
}

pub fn create_zip_members(output: &Path, members: &[MemberSpec]) -> Result<(), ArchiveError> {
    create_zip_members_with_progress(output, members, &|_, _| {})
}

pub fn create_zip_members_with_progress(
    output: &Path,
    members: &[MemberSpec],
    on_member: &dyn Fn(usize, &MemberSpec),
) -> Result<(), ArchiveError> {
    ensure_parent_dir(output)?;
    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);
    // ZIP64 opt-in: members may exceed 4 GiB (LISS-0017).
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .large_file(true);

    for (i, member) in members.iter().enumerate() {
        interrupt::check()?;
        on_member(i, member);
        zip.start_file(&member.member_name, options)
            .map_err(|e| ArchiveError::Engine(e.to_string()))?;
        let mut src = File::open(&member.source)?;
        std::io::copy(&mut src, &mut zip)?;
    }

    zip.finish()
        .map_err(|e| ArchiveError::Engine(e.to_string()))?;
    Ok(())
}

pub fn list_zip(archive: &Path) -> Result<Vec<ArchiveMember>, ArchiveError> {
    let file = File::open(archive)?;
    let mut zip = ZipArchive::new(file).map_err(|e| ArchiveError::Engine(e.to_string()))?;
    let mut members = Vec::new();

    for i in 0..zip.len() {
        let entry = zip
            .by_index(i)
            .map_err(|e| ArchiveError::Engine(e.to_string()))?;
        let name = entry.name().trim_end_matches('/').to_string();
        if name.is_empty() {
            continue;
        }
        members.push(ArchiveMember {
            path: name,
            is_dir: entry.is_dir(),
            size: entry.size(),
        });
    }

    Ok(members)
}

pub fn extract_zip(archive: &Path, dest: &Path) -> Result<(), ArchiveError> {
    std::fs::create_dir_all(dest)?;
    let file = File::open(archive)?;
    let mut zip = ZipArchive::new(file).map_err(|e| ArchiveError::Engine(e.to_string()))?;

    for i in 0..zip.len() {
        interrupt::check()?;
        let mut entry = zip
            .by_index(i)
            .map_err(|e| ArchiveError::Engine(e.to_string()))?;
        let raw_name = entry.name().to_string();

        if entry.is_dir() {
            let out = safe_join(dest, raw_name.trim_end_matches('/'))?;
            std::fs::create_dir_all(&out)?;
            continue;
        }

        let out = safe_join(dest, &raw_name)?;
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = File::create(&out)?;
        std::io::copy(&mut entry, &mut outfile)?;
    }

    Ok(())
}
