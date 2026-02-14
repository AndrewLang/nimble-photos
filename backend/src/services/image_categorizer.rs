use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, hash_map::Entry};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::hash_service::HashService;

pub use crate::domain::image_categorizer::{CategorizeRequest, CategorizeResult, ImageCategorizer};

type CategorizerFactory = Box<dyn Fn() -> Arc<dyn ImageCategorizer> + Send + Sync>;

pub struct ImageCategorizerRegistry {
    factories: HashMap<String, CategorizerFactory>,
    instances: Mutex<HashMap<String, Arc<dyn ImageCategorizer>>>,
}

impl ImageCategorizerRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            instances: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_defaults(hash_service: Arc<HashService>) -> Self {
        let mut registry = Self::new();
        registry.register_factory("hash", {
            let hash_service = Arc::clone(&hash_service);
            Box::new(move || Arc::new(HashImageCategorizer::new(Arc::clone(&hash_service))))
        });
        registry.register_factory("date", Box::new(|| Arc::new(DateImageCategorizer::new())));
        registry
    }

    pub fn register_factory(&mut self, name: impl Into<String>, factory: CategorizerFactory) {
        let key = name.into().to_ascii_lowercase();
        self.factories.insert(key, factory);
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn ImageCategorizer>> {
        let key = name.to_ascii_lowercase();
        if let Some(existing) = self.try_get_cached(&key)? {
            return Ok(existing);
        }

        let factory = self
            .factories
            .get(&key)
            .ok_or_else(|| anyhow!("Image categorizer `{}` not registered", name))?;
        let instance = factory();

        let mut cache = self
            .instances
            .lock()
            .map_err(|_| anyhow!("categorizer registry poisoned"))?;

        Ok(match cache.entry(key) {
            Entry::Occupied(existing) => existing.get().clone(),
            Entry::Vacant(slot) => {
                slot.insert(instance.clone());
                instance
            }
        })
    }

    fn try_get_cached(&self, key: &str) -> Result<Option<Arc<dyn ImageCategorizer>>> {
        Ok(self
            .instances
            .lock()
            .map_err(|_| anyhow!("categorizer registry poisoned"))?
            .get(key)
            .cloned())
    }
}

struct HashImageCategorizer {
    hash_service: Arc<HashService>,
}

impl HashImageCategorizer {
    fn new(hash_service: Arc<HashService>) -> Self {
        Self { hash_service }
    }

    fn ensure_hash(&self, request: &CategorizeRequest<'_>) -> Result<String> {
        if let Some(hash) = request.known_hash() {
            return Ok(hash.to_string());
        }

        let path = request
            .source_file()
            .to_str()
            .ok_or_else(|| anyhow!("source file path is not valid UTF-8"))?;
        self.hash_service
            .compute_file(path)
            .map_err(|err| anyhow!(err))
    }

    fn hashed_subfolders(hash: &str) -> (&str, &str) {
        if hash.len() >= 4 {
            (&hash[0..2], &hash[2..4])
        } else if hash.len() >= 2 {
            (&hash[0..2], &hash[0..2])
        } else {
            ("00", "00")
        }
    }
}

impl ImageCategorizer for HashImageCategorizer {
    fn name(&self) -> &'static str {
        "hash"
    }

    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult> {
        let hash = self.ensure_hash(request)?;
        let (first, second) = Self::hashed_subfolders(&hash);
        let destination_dir = request.destination_root().join(first).join(second);
        let final_path = destination_dir.join(target_file_name(request));

        move_file(request.source_file(), &final_path)?;

        let relative_path = relative_path(request.destination_root(), &final_path)?;
        Ok(CategorizeResult {
            final_path,
            relative_path,
            hash: Some(hash),
        })
    }
}

struct DateImageCategorizer;

impl DateImageCategorizer {
    fn new() -> Self {
        Self
    }

    fn determine_bucket(&self, request: &CategorizeRequest<'_>) -> Result<String> {
        if let Some(date_taken) = request.date_taken() {
            return Ok(date_taken.format("%Y-%m-%d").to_string());
        }

        let metadata = fs::metadata(request.source_file())?;

        let system_time = metadata
            .created()
            .or_else(|_| metadata.modified())
            .unwrap_or_else(|_| std::time::SystemTime::now());

        let datetime: DateTime<Utc> = system_time.into();
        Ok(datetime.format("%Y-%m-%d").to_string())
    }
}

impl ImageCategorizer for DateImageCategorizer {
    fn name(&self) -> &'static str {
        "date"
    }

    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult> {
        let folder = self.determine_bucket(request)?;
        let destination_dir = request.destination_root().join(folder);
        let final_path = destination_dir.join(target_file_name(request));
        move_file(request.source_file(), &final_path)?;
        let relative_path = relative_path(request.destination_root(), &final_path)?;
        Ok(CategorizeResult {
            final_path,
            relative_path,
            hash: None,
        })
    }
}

fn target_file_name(request: &CategorizeRequest<'_>) -> PathBuf {
    let provided = request.file_name();
    let fallback = request
        .source_file()
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("image");
    PathBuf::from(if provided.trim().is_empty() {
        fallback
    } else {
        provided
    })
}

fn move_file(source: &Path, destination: &Path) -> Result<()> {
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

fn relative_path(base: &Path, full: &Path) -> Result<String> {
    let relative = full
        .strip_prefix(base)
        .with_context(|| format!("{} is not inside {}", full.display(), base.display()))?;
    let mut segments = Vec::new();
    for component in relative.components() {
        segments.push(component.as_os_str().to_string_lossy().to_string());
    }
    Ok(segments.join("/"))
}
