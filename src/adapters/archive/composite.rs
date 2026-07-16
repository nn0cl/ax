use super::{tar_engine, zip_engine};
use crate::core::domain::{ArchiveError, ArchiveFormat, ArchiveMember, MemberSpec};
use crate::core::ports::ArchiveEngine;
use std::path::{Path, PathBuf};
use tar_engine::TarCompression;

/// Routes archive operations to the ZIP or TAR engines by format.
#[derive(Debug, Default)]
pub struct CompositeArchiveEngine;

impl CompositeArchiveEngine {
    pub fn new() -> Self {
        Self
    }
}

impl ArchiveEngine for CompositeArchiveEngine {
    fn create(
        &self,
        format: ArchiveFormat,
        output: &Path,
        inputs: &[PathBuf],
    ) -> Result<(), ArchiveError> {
        match format {
            ArchiveFormat::Zip => zip_engine::create_zip(output, inputs),
            ArchiveFormat::Tar => tar_engine::create_tar(output, inputs, TarCompression::None),
            ArchiveFormat::TarGz => tar_engine::create_tar(output, inputs, TarCompression::Gzip),
            ArchiveFormat::TarZst => tar_engine::create_tar(output, inputs, TarCompression::Zstd),
        }
    }

    fn create_members(
        &self,
        format: ArchiveFormat,
        output: &Path,
        members: &[MemberSpec],
    ) -> Result<(), ArchiveError> {
        match format {
            ArchiveFormat::Zip => zip_engine::create_zip_members(output, members),
            ArchiveFormat::Tar => {
                tar_engine::create_tar_members(output, members, TarCompression::None)
            }
            ArchiveFormat::TarGz => {
                tar_engine::create_tar_members(output, members, TarCompression::Gzip)
            }
            ArchiveFormat::TarZst => {
                tar_engine::create_tar_members(output, members, TarCompression::Zstd)
            }
        }
    }

    fn list(
        &self,
        format: ArchiveFormat,
        archive: &Path,
    ) -> Result<Vec<ArchiveMember>, ArchiveError> {
        match format {
            ArchiveFormat::Zip => zip_engine::list_zip(archive),
            ArchiveFormat::Tar => tar_engine::list_tar(archive, TarCompression::None),
            ArchiveFormat::TarGz => tar_engine::list_tar(archive, TarCompression::Gzip),
            ArchiveFormat::TarZst => tar_engine::list_tar(archive, TarCompression::Zstd),
        }
    }

    fn extract(
        &self,
        format: ArchiveFormat,
        archive: &Path,
        dest: &Path,
    ) -> Result<(), ArchiveError> {
        match format {
            ArchiveFormat::Zip => zip_engine::extract_zip(archive, dest),
            ArchiveFormat::Tar => tar_engine::extract_tar(archive, dest, TarCompression::None),
            ArchiveFormat::TarGz => tar_engine::extract_tar(archive, dest, TarCompression::Gzip),
            ArchiveFormat::TarZst => tar_engine::extract_tar(archive, dest, TarCompression::Zstd),
        }
    }
}
