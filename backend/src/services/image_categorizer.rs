use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use nimble_web::ServiceProvider;
use std::collections::{HashMap, hash_map::Entry};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::file_service::FileService;
use super::hash_service::HashService;
use crate::models::property_map::PropertyMap;
use crate::services::image_process_constants::ImageProcessKeys;

#[derive(Debug)]
pub struct CategorizeRequest<'a> {
    source_file: &'a Path,
    properties: &'a PropertyMap,
}

impl<'a> CategorizeRequest<'a> {
    pub fn new(source_file: &'a Path, properties: &'a PropertyMap) -> Self {
        Self {
            source_file,
            properties,
        }
    }

    pub fn source_file(&self) -> &Path {
        self.source_file
    }

    pub fn properties(&self) -> &'a PropertyMap {
        self.properties
    }
}

#[derive(Debug, Clone)]
pub struct CategorizeResult {
    pub final_path: PathBuf,
    pub hash: Option<String>,
}

pub trait ImageCategorizer: Send + Sync {
    fn name(&self) -> &'static str;
    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult>;
}

type CategorizerFactory = Box<dyn Fn() -> Arc<dyn ImageCategorizer> + Send + Sync>;

pub struct ImageCategorizerRegistry {
    services: Arc<ServiceProvider>,
    factories: HashMap<String, CategorizerFactory>,
    instances: Mutex<HashMap<String, Arc<dyn ImageCategorizer>>>,
}

impl ImageCategorizerRegistry {
    pub fn new(services: Arc<ServiceProvider>) -> Self {
        Self {
            services: services.clone(),
            factories: HashMap::new(),
            instances: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_defaults(services: Arc<ServiceProvider>) -> Self {
        let mut registry = Self::new(services.clone());

        let services_for_hash = services.clone();
        registry.register_factory("hash", {
            Box::new(move || Arc::new(HashImageCategorizer::new(services_for_hash.clone())))
        });

        let services_for_date = services.clone();
        registry.register_factory("date", {
            Box::new(move || Arc::new(DateImageCategorizer::new(services_for_date.clone())))
        });

        registry
    }

    pub fn register_categorizer(&mut self, categorizer: Arc<dyn ImageCategorizer>) {
        let name = categorizer.name().to_ascii_lowercase();
        self.factories
            .insert(name.clone(), Box::new(move || categorizer.clone()));
        log::info!("Registered image categorizer: {}", name);
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

pub(crate) struct HashImageCategorizer {
    services: Arc<ServiceProvider>,
    hash_service: Arc<HashService>,
    file_service: Arc<FileService>,
}

impl HashImageCategorizer {
    pub(crate) fn new(services: Arc<ServiceProvider>) -> Self {
        log::debug!("Initializing HashImageCategorizer...");
        let hash_service = services.get::<HashService>();
        let file_service = services.get::<FileService>();
        Self {
            services,
            hash_service,
            file_service,
        }
    }

    fn output_file(&self, root: &PathBuf, hash: &str, file_name: &str) -> PathBuf {
        root.join(&hash[0..2]).join(&hash[2..4]).join(file_name)
    }
}

impl ImageCategorizer for HashImageCategorizer {
    fn name(&self) -> &'static str {
        "hash"
    }

    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult> {
        let hash = request
            .properties
            .get_by_alias(ImageProcessKeys::HASH)
            .map(|value: &String| value.as_str())
            .map(|value| value.to_string())
            .ok_or_else(|| anyhow!("failed to determine hash for categorization"))?;

        let file_name = request
            .source_file()
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid file name"))?;

        let working_dir = request
            .properties
            .get_by_alias::<PathBuf>(ImageProcessKeys::WORKING_DIRECTORY)
            .ok_or_else(|| {
                anyhow!("working directory not found in properties for categorization")
            })?;

        let output_path = self.output_file(working_dir, &hash, file_name);
        log::debug!(
            "Categorizing image by hash, hash: {}, file name: {}, working directory: {}, output path: {}",
            hash,
            file_name,
            working_dir.clone().display(),
            output_path.clone().display()
        );

        self.file_service
            .move_file(request.source_file(), &output_path)?;

        Ok(CategorizeResult {
            final_path: output_path,
            hash: Some(hash.to_string()),
        })
    }
}

pub(crate) struct DateImageCategorizer {
    services: Arc<ServiceProvider>,
    file_service: Arc<FileService>,
}

impl DateImageCategorizer {
    pub(crate) fn new(services: Arc<ServiceProvider>) -> Self {
        let file_service = services.get::<FileService>();
        Self {
            services,
            file_service,
        }
    }

    fn output_file(
        &self,
        root: &PathBuf,
        date: &DateTime<Utc>,
        format: &str,
        file_name: &str,
    ) -> PathBuf {
        let date_name = self.format_date(date, format);
        root.join(date_name).join(file_name)
    }

    fn format_date(&self, dt: &DateTime<Utc>, format: &str) -> String {
        dt.format(format).to_string()
    }
}

impl ImageCategorizer for DateImageCategorizer {
    fn name(&self) -> &'static str {
        "date"
    }

    fn categorize(&self, request: &CategorizeRequest<'_>) -> Result<CategorizeResult> {
        let date = request
            .properties
            .get_by_alias::<Option<DateTime<Utc>>>(ImageProcessKeys::EXIF_DATE_TAKEN)
            .and_then(|value| value.as_ref())
            .ok_or_else(|| anyhow!("failed to determine date taken for categorization"))?;
        let date_format = request
            .properties
            .get_by_alias::<String>(ImageProcessKeys::CATEGORIZE_DATE_FORMAT)
            .map(String::as_str)
            .unwrap_or("%Y-%m-%d");
        let file_name = request
            .source_file()
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid file name"))?;
        let working_dir = request
            .properties
            .get_by_alias::<PathBuf>(ImageProcessKeys::WORKING_DIRECTORY)
            .ok_or_else(|| {
                anyhow!("working directory not found in properties for categorization")
            })?;

        let output_file = self.output_file(working_dir, date, date_format, file_name);

        log::debug!(
            "Categorizing image by date, date: {}, file name: {}, working directory: {}, output path: {}",
            date,
            file_name,
            working_dir.display(),
            output_file.display()
        );

        self.file_service
            .move_file(request.source_file(), &output_file)?;

        Ok(CategorizeResult {
            final_path: output_file,
            hash: None,
        })
    }
}
