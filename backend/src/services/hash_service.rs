use anyhow::Result;
use std::fs;
use xxhash_rust::xxh3::Xxh3;

pub struct HashService;

impl HashService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute(&self, data: &[u8], file_size: usize) -> String {
        const CHUNK: usize = 64 * 1024;
        let len = file_size;
        let mut hasher = Xxh3::new();

        hasher.update(&data[..CHUNK.min(len)]);
        if len > CHUNK * 2 {
            let mid = len / 2;
            let end = (mid + CHUNK).min(len);
            hasher.update(&data[mid..end]);
        }
        if len > CHUNK {
            hasher.update(&data[len - CHUNK.min(len)..]);
        }
        hasher.update(&len.to_le_bytes());

        let hash = format!("{:016x}", hasher.digest());
        hash
    }

    pub fn compute_file(&self, path: &str) -> Result<String> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        let hash = self.compute(&fs::read(path)?, size as usize);

        Ok(hash)
    }
}
