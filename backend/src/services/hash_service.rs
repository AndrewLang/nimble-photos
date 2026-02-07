use anyhow::Result;
use std::fs;
use std::time::SystemTime;
use xxhash_rust::xxh3::Xxh3;

pub struct HashService;

impl HashService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute(&self, data: &Vec<u8>, file_size: usize, file_date: SystemTime) -> String {
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
        hasher.update(
            &file_date
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_le_bytes(),
        );

        let hash = format!("{:016x}", hasher.digest());
        hash
    }

    pub fn compute_file(&self, path: &str) -> Result<String> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        let modified = metadata.modified()?;
        let hash = self.compute(&fs::read(path)?, size as usize, modified);

        Ok(hash)
    }
}
