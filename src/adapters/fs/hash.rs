use crate::core::domain::ArchiveError;
use crate::core::interrupt;
use crate::core::ports::ContentHasher;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Sha256Hasher;

impl Sha256Hasher {
    pub fn new() -> Self {
        Self
    }
}

impl ContentHasher for Sha256Hasher {
    fn hash_file(&self, path: &Path) -> Result<String, ArchiveError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 64 * 1024];
        let mut since_check = 0u64;
        loop {
            interrupt::check()?;
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            since_check += n as u64;
            // Re-check at least every ~1 MiB for large files.
            if since_check >= 1024 * 1024 {
                interrupt::check()?;
                since_check = 0;
            }
        }
        Ok(hex::encode(hasher.finalize()))
    }
}
