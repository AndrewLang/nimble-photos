use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

/// Describes the inputs an image categorizer can use to determine the final storage location.
#[derive(Debug)]
pub struct CategorizeRequest<'a> {
    source_file: &'a Path,
    destination_root: &'a Path,
    file_name: &'a str,
    known_hash: Option<&'a str>,
    date_taken: Option<DateTime<Utc>>,
}

impl<'a> CategorizeRequest<'a> {
    pub fn new(source_file: &'a Path, destination_root: &'a Path, file_name: &'a str) -> Self {
        Self {
            source_file,
            destination_root,
            file_name,
            known_hash: None,
            date_taken: None,
        }
    }

    pub fn with_known_hash(mut self, hash: Option<&'a str>) -> Self {
        self.known_hash = hash;
        self
    }

    pub fn with_date_taken(mut self, date_taken: Option<DateTime<Utc>>) -> Self {
        self.date_taken = date_taken;
        self
    }

    pub fn source_file(&self) -> &Path {
        self.source_file
    }

    pub fn destination_root(&self) -> &Path {
        self.destination_root
    }

    pub fn file_name(&self) -> &str {
        self.file_name
    }

    pub fn known_hash(&self) -> Option<&str> {
        self.known_hash
    }

    pub fn date_taken(&self) -> Option<DateTime<Utc>> {
        self.date_taken
    }
}

/// Result of a categorization pass, indicating where the file ended up.
#[derive(Debug, Clone)]
pub struct CategorizeResult {
    pub final_path: PathBuf,
    pub relative_path: String,
    pub hash: Option<String>,
}

/// Strategy interface for categorizing images by a user-defined criterion (hash, date, etc.).
pub trait ImageCategorizer: Send + Sync {
    fn name(&self) -> &'static str;
    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult>;
}
