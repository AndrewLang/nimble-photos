use crate::entities::{exif::ExifModel, photo::Photo};
use crate::services::background_task_runner::BackgroundTaskRunner;
use crate::services::exif_service::ExifService;
use crate::services::hash_service::HashService;
use crate::services::image_categorizer::{CategorizeRequest, ImageCategorizerRegistry};
use crate::services::image_process_service::ImageProcessService;
use crate::services::photo_upload_service::StoredUploadFile;
use crate::services::task_descriptor::TaskDescriptor;
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use image::image_dimensions;
use log::{error, info, trace};
use nimble_web::config::Configuration;
use nimble_web::data::repository::Repository;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ImageStorageLocation {
    pub id: String,
    pub label: String,
    pub path: PathBuf,
    pub created_at: String,
}

impl ImageStorageLocation {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        path: impl Into<PathBuf>,
        created_at: impl Into<String>,
    ) -> Self {
        let path = normalize_path(path.into());
        Self {
            id: id.into(),
            label: label.into(),
            path,
            created_at: created_at.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageProcessRequest {
    pub storage: ImageStorageLocation,
    pub relative_path: String,
    pub file_name: String,
    pub byte_size: usize,
    pub content_type: Option<String>,
}

impl ImageProcessRequest {
    pub fn from_upload(storage: ImageStorageLocation, file: StoredUploadFile) -> Self {
        Self {
            storage,
            relative_path: file.relative_path,
            file_name: file.file_name,
            byte_size: file.byte_size,
            content_type: file.content_type,
        }
    }

    pub fn source_path(&self) -> PathBuf {
        self.storage.path.join(Path::new(&self.relative_path))
    }
}

pub struct ImageProcessContext {
    request: ImageProcessRequest,
    source_path: PathBuf,
    final_path: Option<PathBuf>,
    relative_final_path: Option<String>,
    hash: Option<String>,
    exif: Option<ExifModel>,
    date_taken: Option<DateTime<Utc>>,
    thumbnail_root: PathBuf,
    preview_root: PathBuf,
    thumbnail_path: Option<PathBuf>,
    preview_path: Option<PathBuf>,
    thumbnail_dimensions: Option<(u32, u32)>,
    preview_dimensions: Option<(u32, u32)>,
    image_dimensions: Option<(u32, u32)>,
}

impl ImageProcessContext {
    fn new(request: ImageProcessRequest, thumbnail_root: PathBuf, preview_root: PathBuf) -> Self {
        let source_path = request.source_path();
        Self {
            request,
            source_path,
            final_path: None,
            relative_final_path: None,
            hash: None,
            exif: None,
            date_taken: None,
            thumbnail_root,
            preview_root,
            thumbnail_path: None,
            preview_path: None,
            thumbnail_dimensions: None,
            preview_dimensions: None,
            image_dimensions: None,
        }
    }

    fn storage_root(&self) -> &Path {
        &self.request.storage.path
    }

    fn source_path(&self) -> &Path {
        &self.source_path
    }

    fn active_image_path(&self) -> &Path {
        self.final_path
            .as_deref()
            .unwrap_or_else(|| self.source_path())
    }

    fn set_hash(&mut self, hash: String) {
        self.hash = Some(hash);
    }

    fn hash(&self) -> Option<&str> {
        self.hash.as_deref()
    }

    fn set_exif(&mut self, exif: ExifModel) {
        self.exif = Some(exif);
    }

    fn exif(&self) -> Option<&ExifModel> {
        self.exif.as_ref()
    }

    fn set_date_taken(&mut self, date: Option<DateTime<Utc>>) {
        self.date_taken = date;
    }

    fn date_taken(&self) -> Option<DateTime<Utc>> {
        self.date_taken
    }

    fn set_thumbnail(&mut self, path: PathBuf, dimensions: (u32, u32)) {
        self.thumbnail_path = Some(path);
        self.thumbnail_dimensions = Some(dimensions);
    }

    fn thumbnail_relative_path(&self) -> Result<Option<String>> {
        match &self.thumbnail_path {
            Some(path) => Ok(Some(relative_path(&self.thumbnail_root, path)?)),
            None => Ok(None),
        }
    }

    fn set_preview(&mut self, path: PathBuf, dimensions: (u32, u32)) {
        self.preview_path = Some(path);
        self.preview_dimensions = Some(dimensions);
    }

    fn set_image_dimensions(&mut self, dimensions: (u32, u32)) {
        self.image_dimensions = Some(dimensions);
    }

    fn set_final_path(&mut self, path: PathBuf, relative: String) {
        self.final_path = Some(path);
        self.relative_final_path = Some(relative);
    }

    fn relative_final_path(&self) -> Option<&str> {
        self.relative_final_path.as_deref()
    }

    fn extension(&self) -> Option<String> {
        Path::new(&self.request.file_name)
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
    }
}

#[async_trait]
trait ImageProcessStep: Send + Sync {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()>;
}

struct ExtractExifStep {
    service: Arc<ExifService>,
}

impl ExtractExifStep {
    fn new(service: Arc<ExifService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ImageProcessStep for ExtractExifStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let service = Arc::clone(&self.service);
        let source = context.source_path().to_path_buf();
        let exif = task::spawn_blocking(move || service.extract_from_path(source))
            .await
            .context("exif extraction task join error")?;

        let date_taken = parse_exif_datetime(&exif);
        context.set_exif(exif);
        context.set_date_taken(date_taken);
        Ok(())
    }
}

struct ComputeHashStep {
    service: Arc<HashService>,
}

impl ComputeHashStep {
    fn new(service: Arc<HashService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ImageProcessStep for ComputeHashStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let service = Arc::clone(&self.service);
        let source = context
            .source_path()
            .to_str()
            .ok_or_else(|| anyhow!("source path is not valid UTF-8"))?
            .to_string();
        let hash = task::spawn_blocking(move || service.compute_file(&source))
            .await
            .context("hash compute join error")?
            .context("hash compute failed")?;

        context.set_hash(hash);
        Ok(())
    }
}

struct GenerateThumbnailStep {
    service: Arc<ImageProcessService>,
    root: PathBuf,
}

impl GenerateThumbnailStep {
    fn new(service: Arc<ImageProcessService>, root: PathBuf) -> Self {
        Self { service, root }
    }
}

#[async_trait]
impl ImageProcessStep for GenerateThumbnailStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let hash = context
            .hash()
            .ok_or_else(|| anyhow!("hash must be computed before thumbnail generation"))?
            .to_string();
        let (first, second) = hash_segments(&hash);
        let output_path = self.root.join(&first).join(&second).join(format!(
            "{}.{}",
            hash,
            self.service.output_format_extension()
        ));
        if let Some(parent) = output_path.parent() {
            task::spawn_blocking({
                let parent = parent.to_path_buf();
                move || std::fs::create_dir_all(parent)
            })
            .await
            .context("thumbnail directory creation join error")??;
        }

        let service = Arc::clone(&self.service);
        let source = context.source_path().to_path_buf();
        let output = output_path.clone();
        task::spawn_blocking(move || {
            service.generate_thumbnail_from_file(source, &output)?;
            Result::<_, anyhow::Error>::Ok(())
        })
        .await
        .context("thumbnail generation join error")??;

        let dimensions = task::spawn_blocking({
            let output = output_path.clone();
            move || image_dimensions(&output).map_err(|err| anyhow!(err))
        })
        .await
        .context("thumbnail dimension read join error")??;

        context.set_thumbnail(output_path, dimensions);
        Ok(())
    }
}

struct GeneratePreviewStep {
    service: Arc<ImageProcessService>,
    root: PathBuf,
}

impl GeneratePreviewStep {
    fn new(service: Arc<ImageProcessService>, root: PathBuf) -> Self {
        Self { service, root }
    }
}

#[async_trait]
impl ImageProcessStep for GeneratePreviewStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let hash = context
            .hash()
            .ok_or_else(|| anyhow!("hash must be computed before preview generation"))?
            .to_string();
        let (first, second) = hash_segments(&hash);
        let output_path = self.root.join(&first).join(&second).join(format!(
            "{}.{}",
            hash,
            self.service.output_format_extension()
        ));
        if let Some(parent) = output_path.parent() {
            task::spawn_blocking({
                let parent = parent.to_path_buf();
                move || std::fs::create_dir_all(parent)
            })
            .await
            .context("preview directory creation join error")??;
        }

        let service = Arc::clone(&self.service);
        let source = context.source_path().to_path_buf();
        let output = output_path.clone();
        task::spawn_blocking(move || {
            service.generate_preview_from_file(source, &output)?;
            Result::<_, anyhow::Error>::Ok(())
        })
        .await
        .context("preview generation join error")??;

        let dimensions = task::spawn_blocking({
            let output = output_path.clone();
            move || image_dimensions(&output).map_err(|err| anyhow!(err))
        })
        .await
        .context("preview dimension read join error")??;

        context.set_preview(output_path, dimensions);
        Ok(())
    }
}

struct CategorizeImageStep {
    registry: Arc<ImageCategorizerRegistry>,
    categorizer_name: String,
}

impl CategorizeImageStep {
    fn new(registry: Arc<ImageCategorizerRegistry>, categorizer_name: String) -> Self {
        Self {
            registry,
            categorizer_name: categorizer_name.to_ascii_lowercase(),
        }
    }
}

#[async_trait]
impl ImageProcessStep for CategorizeImageStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let categorizer = self.registry.get(&self.categorizer_name)?;
        let request = CategorizeRequest::new(
            context.source_path(),
            context.storage_root(),
            &context.request.file_name,
        )
        .with_known_hash(context.hash())
        .with_date_taken(context.date_taken());

        let result = categorizer.categorize(&request)?;
        if context.hash.is_none() {
            if let Some(hash) = &result.hash {
                context.set_hash(hash.clone());
            }
        }
        context.set_final_path(result.final_path, result.relative_path);
        Ok(())
    }
}

struct PersistMetadataStep {
    photo_repo: Arc<Repository<Photo>>,
    exif_repo: Arc<Repository<ExifModel>>,
}

impl PersistMetadataStep {
    fn new(photo_repo: Arc<Repository<Photo>>, exif_repo: Arc<Repository<ExifModel>>) -> Self {
        Self {
            photo_repo,
            exif_repo,
        }
    }
}

#[async_trait]
impl ImageProcessStep for PersistMetadataStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let final_path = context
            .final_path
            .clone()
            .unwrap_or_else(|| context.source_path().to_path_buf());
        let dimensions = task::spawn_blocking({
            let path = final_path.clone();
            move || image_dimensions(&path).map_err(|err| anyhow!(err))
        })
        .await
        .context("image dimension read join error")??;
        context.set_image_dimensions(dimensions);

        let photo = Photo {
            id: Some(Uuid::new_v4()),
            path: context
                .relative_final_path()
                .map(|value| value.to_string())
                .unwrap_or_else(|| context.request.relative_path.clone()),
            name: context.request.file_name.clone(),
            format: context.extension(),
            hash: context.hash().map(|value| value.to_string()),
            size: Some(context.request.byte_size as i64),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            date_imported: Some(Utc::now()),
            date_taken: context.date_taken(),
            thumbnail_path: context.thumbnail_relative_path()?,
            thumbnail_optimized: Some(true),
            metadata_extracted: Some(context.exif().is_some()),
            is_raw: context
                .extension()
                .map(|ext| ImageProcessService::is_raw_extension(&ext)),
            width: context.image_dimensions.map(|value| value.0),
            height: context.image_dimensions.map(|value| value.1),
            thumbnail_width: context.thumbnail_dimensions.map(|value| value.0),
            thumbnail_height: context.thumbnail_dimensions.map(|value| value.1),
            tags: None,
        };

        let saved_photo = self
            .photo_repo
            .insert(photo)
            .await
            .map_err(|err| anyhow!("failed to insert photo: {:?}", err))?;

        if let Some(mut metadata) = context.exif.clone() {
            metadata.id = Some(Uuid::new_v4());
            metadata.image_id = saved_photo.id;
            metadata.hash = saved_photo.hash.clone();

            let _ = self
                .exif_repo
                .insert(metadata)
                .await
                .map_err(|err| anyhow!("failed to insert exif metadata: {:?}", err))?;
        }

        info!(
            "Processed image {} into storage {}",
            saved_photo.name, saved_photo.path
        );

        Ok(())
    }
}

#[derive(Clone)]
pub struct ImageProcessPipeline {
    runner: Arc<BackgroundTaskRunner>,
    steps: Vec<Arc<dyn ImageProcessStep>>,
    thumbnail_root: PathBuf,
    preview_root: PathBuf,
}

impl ImageProcessPipeline {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        runner: Arc<BackgroundTaskRunner>,
        hash_service: Arc<HashService>,
        exif_service: Arc<ExifService>,
        image_service: Arc<ImageProcessService>,
        photo_repo: Arc<Repository<Photo>>,
        exif_repo: Arc<Repository<ExifModel>>,
        configuration: Configuration,
    ) -> Self {
        let thumbnail_root = config_path(
            &configuration,
            &["thumbnail.base.path", "thumbnail.basepath"],
            "./thumbnails",
        );
        let preview_root = config_path(
            &configuration,
            &["preview.base.path", "preview.basepath"],
            "./previews",
        );
        let categorizer_name = configuration
            .get("image.categorizer")
            .or_else(|| configuration.get("photo.categorizer"))
            .unwrap_or("hash")
            .to_ascii_lowercase();

        let registry = Arc::new(ImageCategorizerRegistry::with_defaults(
            hash_service.clone(),
        ));

        let steps: Vec<Arc<dyn ImageProcessStep>> = vec![
            Arc::new(ExtractExifStep::new(exif_service)),
            Arc::new(ComputeHashStep::new(hash_service.clone())),
            Arc::new(GenerateThumbnailStep::new(
                image_service.clone(),
                thumbnail_root.clone(),
            )),
            Arc::new(GeneratePreviewStep::new(
                image_service,
                preview_root.clone(),
            )),
            Arc::new(CategorizeImageStep::new(registry, categorizer_name.clone())),
            Arc::new(PersistMetadataStep::new(photo_repo, exif_repo)),
        ];

        Self {
            runner,
            steps,
            thumbnail_root,
            preview_root,
        }
    }

    pub fn enqueue_uploaded_files(
        &self,
        storage: ImageStorageLocation,
        files: Vec<StoredUploadFile>,
    ) -> Result<()> {
        for file in files {
            let request = ImageProcessRequest::from_upload(storage.clone(), file);
            self.enqueue_request(request)?;
        }
        Ok(())
    }

    fn enqueue_request(&self, request: ImageProcessRequest) -> Result<()> {
        let pipeline = self.clone();
        let task_name = format!("image-process-{}-{}", request.storage.id, request.file_name);
        self.runner
            .enqueue(TaskDescriptor::new(task_name, async move {
                if let Err(error) = pipeline.run_steps(request).await {
                    error!("Image process pipeline failed: {:?}", error);
                    return Err(error);
                }
                Ok(())
            }))
    }

    async fn run_steps(&self, request: ImageProcessRequest) -> Result<()> {
        trace!(
            "Starting pipeline for storage {} file {}",
            request.storage.id, request.file_name
        );
        let mut context = ImageProcessContext::new(
            request,
            self.thumbnail_root.clone(),
            self.preview_root.clone(),
        );
        for step in &self.steps {
            step.execute(&mut context).await?;
        }
        Ok(())
    }

    pub async fn process_now(&self, request: ImageProcessRequest) -> Result<()> {
        self.run_steps(request).await
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn config_path(config: &Configuration, keys: &[&str], fallback: &str) -> PathBuf {
    for key in keys {
        if let Some(value) = config.get(key) {
            return normalize_path(PathBuf::from(value));
        }
    }
    normalize_path(PathBuf::from(fallback))
}

fn relative_path(base: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(base)
        .with_context(|| format!("{} is not inside {}", path.display(), base.display()))?;
    let mut components = Vec::new();
    for component in relative.components() {
        components.push(component.as_os_str().to_string_lossy().to_string());
    }
    Ok(components.join("/"))
}

fn hash_segments(hash: &str) -> (String, String) {
    if hash.len() >= 4 {
        (hash[0..2].to_string(), hash[2..4].to_string())
    } else {
        let mut normalized = hash.to_string();
        while normalized.len() < 4 {
            normalized.push('0');
        }
        (normalized[0..2].to_string(), normalized[2..4].to_string())
    }
}

fn parse_exif_datetime(model: &ExifModel) -> Option<DateTime<Utc>> {
    let candidates = [
        model.datetime_original.as_deref(),
        model.datetime.as_deref(),
        model.datetime_digitized.as_deref(),
    ];

    for candidate in candidates.into_iter().flatten() {
        if let Some(parsed) = parse_exif_timestamp(candidate) {
            return Some(parsed);
        }
    }

    None
}

fn parse_exif_timestamp(raw: &str) -> Option<DateTime<Utc>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let formats = ["%Y:%m:%d %H:%M:%S", "%Y-%m-%d %H:%M:%S"];
    for format in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }

    DateTime::parse_from_rfc3339(trimmed)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}
