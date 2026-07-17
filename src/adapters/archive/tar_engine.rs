use crate::adapters::fs::{collect_pack_entries, ensure_parent_dir};
use crate::core::domain::{safe_join, ArchiveError, ArchiveMember, MemberSpec};
use crate::core::interrupt;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use zstd::stream::{Decoder as ZstdDecoder, Encoder as ZstdEncoder};

#[derive(Clone, Copy)]
pub enum TarCompression {
    None,
    Gzip,
    Zstd,
}

pub fn create_tar(
    output: &Path,
    inputs: &[PathBuf],
    compression: TarCompression,
) -> Result<(), ArchiveError> {
    let entries = collect_pack_entries(inputs)?;
    let members: Vec<MemberSpec> = entries
        .into_iter()
        .map(|e| MemberSpec {
            source: e.source,
            member_name: e.member_name,
        })
        .collect();
    create_tar_members(output, &members, compression)
}

pub fn create_tar_members(
    output: &Path,
    members: &[MemberSpec],
    compression: TarCompression,
) -> Result<(), ArchiveError> {
    ensure_parent_dir(output)?;
    let file = File::create(output)?;

    match compression {
        TarCompression::None => write_tar(file, members)?,
        TarCompression::Gzip => {
            let enc = GzEncoder::new(file, Compression::default());
            write_tar(enc, members)?;
        }
        TarCompression::Zstd => {
            // 0 threads ⇒ zstd uses as many threads as CPU cores (multi-threaded).
            let mut enc = ZstdEncoder::new(file, 3)
                .map_err(|e| ArchiveError::Engine(e.to_string()))?;
            enc.multithread(0)
                .map_err(|e| ArchiveError::Engine(e.to_string()))?;
            write_tar(&mut enc, members)?;
            enc.finish()
                .map_err(|e| ArchiveError::Engine(e.to_string()))?;
        }
    }
    Ok(())
}

fn write_tar<W: Write>(writer: W, members: &[MemberSpec]) -> Result<(), ArchiveError> {
    let mut builder = Builder::new(writer);
    for member in members {
        interrupt::check()?;
        builder
            .append_path_with_name(&member.source, &member.member_name)
            .map_err(|e| ArchiveError::Engine(e.to_string()))?;
    }
    builder
        .finish()
        .map_err(|e| ArchiveError::Engine(e.to_string()))?;
    Ok(())
}

pub fn list_tar(
    archive: &Path,
    compression: TarCompression,
) -> Result<Vec<ArchiveMember>, ArchiveError> {
    let file = File::open(archive)?;
    let mut members = Vec::new();

    match compression {
        TarCompression::None => collect_tar_members(Archive::new(file), &mut members)?,
        TarCompression::Gzip => {
            collect_tar_members(Archive::new(GzDecoder::new(file)), &mut members)?
        }
        TarCompression::Zstd => {
            let dec = ZstdDecoder::new(file).map_err(|e| ArchiveError::Engine(e.to_string()))?;
            collect_tar_members(Archive::new(dec), &mut members)?;
        }
    }

    Ok(members)
}

fn collect_tar_members<R: Read>(
    mut archive: Archive<R>,
    members: &mut Vec<ArchiveMember>,
) -> Result<(), ArchiveError> {
    for entry in archive
        .entries()
        .map_err(|e| ArchiveError::Engine(e.to_string()))?
    {
        let entry = entry.map_err(|e| ArchiveError::Engine(e.to_string()))?;
        let path = entry
            .path()
            .map_err(|e| ArchiveError::Engine(e.to_string()))?
            .to_string_lossy()
            .trim_end_matches('/')
            .to_string();
        if path.is_empty() {
            continue;
        }
        members.push(ArchiveMember {
            path,
            is_dir: entry.header().entry_type().is_dir(),
            size: entry.header().size().unwrap_or(0),
        });
    }
    Ok(())
}

pub fn extract_tar(
    archive: &Path,
    dest: &Path,
    compression: TarCompression,
) -> Result<(), ArchiveError> {
    std::fs::create_dir_all(dest)?;
    let file = File::open(archive)?;

    match compression {
        TarCompression::None => unpack_tar(Archive::new(file), dest)?,
        TarCompression::Gzip => unpack_tar(Archive::new(GzDecoder::new(file)), dest)?,
        TarCompression::Zstd => {
            let dec = ZstdDecoder::new(file).map_err(|e| ArchiveError::Engine(e.to_string()))?;
            unpack_tar(Archive::new(dec), dest)?;
        }
    }
    Ok(())
}

fn unpack_tar<R: Read>(mut archive: Archive<R>, dest: &Path) -> Result<(), ArchiveError> {
    for entry in archive
        .entries()
        .map_err(|e| ArchiveError::Engine(e.to_string()))?
    {
        interrupt::check()?;
        let mut entry = entry.map_err(|e| ArchiveError::Engine(e.to_string()))?;
        let raw = entry
            .path()
            .map_err(|e| ArchiveError::Engine(e.to_string()))?
            .to_string_lossy()
            .to_string();

        let _safe = safe_join(dest, raw.trim_end_matches('/'))?;

        entry
            .unpack_in(dest)
            .map_err(|e| ArchiveError::Engine(e.to_string()))?;
    }
    Ok(())
}
