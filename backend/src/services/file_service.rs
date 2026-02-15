use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileService;

impl FileService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn target_file_name(&self, requested_name: &str, source_file: &Path) -> PathBuf {
        let fallback = source_file
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("image");
        let trimmed = requested_name.trim();
        let resolved = if trimmed.is_empty() {
            fallback
        } else {
            trimmed
        };
        PathBuf::from(resolved)
    }

    pub fn move_file(&self, source: &Path, destination: &Path) -> Result<()> {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        match fs::rename(source, destination) {
            Ok(_) => Ok(()),
            Err(_) => {
                fs::copy(source, destination)?;
                fs::remove_file(source)?;
                Ok(())
            }
        }
    }

    pub fn relative_path(&self, base: &Path, full: &Path) -> Result<String> {
        let relative = full
            .strip_prefix(base)
            .with_context(|| format!("{} is not inside {}", full.display(), base.display()))?;
        let mut segments = Vec::new();
        for component in relative.components() {
            segments.push(component.as_os_str().to_string_lossy().to_string());
        }
        Ok(segments.join("/"))
    }
}
