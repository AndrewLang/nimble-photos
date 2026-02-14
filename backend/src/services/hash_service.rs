use anyhow::Result;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::time::SystemTime;
use xxhash_rust::xxh3::Xxh3;

pub struct HashService;

impl HashService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute(&self, data: &[u8], file_size: usize, file_date: SystemTime) -> String {
        const CHUNK: usize = 64 * 1024;
        let len = file_size;
        assert!(
            data.len() >= len,
            "data buffer is smaller than the declared file size"
        );
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
        Self::finalize_hash(hasher, len, file_date)
    }

    pub fn compute_file(&self, path: &str) -> Result<String> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        let modified = metadata.modified()?;
        let mut file = File::open(path)?;
        self.compute_from_reader(&mut file, size as usize, modified)
    }

    fn compute_from_reader<R: Read + Seek>(
        &self,
        reader: &mut R,
        len: usize,
        file_date: SystemTime,
    ) -> Result<String> {
        const CHUNK: usize = 64 * 1024;
        let mut hasher = Xxh3::new();
        let mut buffer = vec![0u8; CHUNK];

        let head_len = CHUNK.min(len);
        if head_len > 0 {
            reader.seek(SeekFrom::Start(0))?;
            reader.read_exact(&mut buffer[..head_len])?;
            hasher.update(&buffer[..head_len]);
        }

        if len > CHUNK * 2 {
            let mid = len / 2;
            let end = (mid + CHUNK).min(len);
            let mid_len = end - mid;
            reader.seek(SeekFrom::Start(mid as u64))?;
            reader.read_exact(&mut buffer[..mid_len])?;
            hasher.update(&buffer[..mid_len]);
        }

        if len > CHUNK {
            let tail_len = CHUNK.min(len);
            let start = len - tail_len;
            reader.seek(SeekFrom::Start(start as u64))?;
            reader.read_exact(&mut buffer[..tail_len])?;
            hasher.update(&buffer[..tail_len]);
        }

        Ok(Self::finalize_hash(hasher, len, file_date))
    }

    fn finalize_hash(mut hasher: Xxh3, len: usize, file_date: SystemTime) -> String {
        hasher.update(&len.to_le_bytes());
        hasher.update(
            &file_date
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_le_bytes(),
        );

        format!("{:016x}", hasher.digest())
    }
}
