use anyhow::Result;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct PhotoService;

impl PhotoService {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_photos(&self, _folder: &Path) -> Result<()> {
        Ok(())
    }
}
