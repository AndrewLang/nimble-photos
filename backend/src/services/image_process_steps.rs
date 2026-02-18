use super::image_process_context::ImageProcessContext;
use super::image_process_step::ImageProcessStep;
use crate::entities::{exif::ExifModel, photo::Photo};
use crate::services::exif_service::ExifService;
use crate::services::hash_service::HashService;
use crate::services::image_categorizer::{
    CategorizeRequest, ImageCategorizer, TemplateCategorizer,
};
use crate::services::image_process_constants::ImageProcessKeys;
use crate::services::{PreviewExtractor, ThumbnailExtractor};

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use nimble_web::ServiceProvider;
use nimble_web::data::repository::Repository;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

pub(super) struct ExtractExifStep {
    services: Arc<ServiceProvider>,
    exif_service: Arc<ExifService>,
}

impl ExtractExifStep {
    pub(super) fn new(services: Arc<ServiceProvider>) -> Self {
        let exif_service = services.get::<ExifService>();
        Self {
            services,
            exif_service,
        }
    }

    fn parse_exif_datetime(model: &ExifModel) -> Option<DateTime<Utc>> {
        let candidates = [
            model.datetime_original.as_deref(),
            model.datetime.as_deref(),
            model.datetime_digitized.as_deref(),
        ];

        for candidate in candidates.into_iter().flatten() {
            if let Some(parsed) = Self::parse_exif_timestamp(candidate) {
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
}

#[async_trait]
impl ImageProcessStep for ExtractExifStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        log::debug!(
            "Extracting EXIF metadata for {}",
            context.source_path().display()
        );
        let service = Arc::clone(&self.exif_service);
        let source = context.source_path().to_path_buf();
        let exif = task::spawn_blocking(move || service.extract_from_path(source))
            .await
            .context("exif extraction task join error")?;

        let date_taken = Self::parse_exif_datetime(&exif);
        context.insert::<ExifModel>(ImageProcessKeys::EXIF_METADATA, exif);
        context.insert::<Option<DateTime<Utc>>>(ImageProcessKeys::EXIF_DATE_TAKEN, date_taken);
        context.insert::<PathBuf>(
            ImageProcessKeys::WORKING_DIRECTORY,
            context.payload().working_directory(),
        );
        log::debug!("EXIF extraction complete, date taken: {:?}", date_taken);
        log::debug!(
            "Working directory: {}",
            context.payload().working_directory().display()
        );
        Ok(())
    }
}

pub(super) struct ComputeHashStep {
    services: Arc<ServiceProvider>,
    hash_service: Arc<HashService>,
}

impl ComputeHashStep {
    pub(super) fn new(services: Arc<ServiceProvider>) -> Self {
        let hash_service = services.get::<HashService>();
        Self {
            services,
            hash_service,
        }
    }
}

#[async_trait]
impl ImageProcessStep for ComputeHashStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        log::debug!("Computing hash for {}", context.source_path().display());
        let service = Arc::clone(&self.hash_service);
        let source = context
            .source_path()
            .to_str()
            .ok_or_else(|| anyhow!("source path is not valid UTF-8"))?
            .to_string();
        let hash = task::spawn_blocking(move || service.compute_file(&source))
            .await
            .context("hash compute join error")?
            .context("hash compute failed")?;

        context.insert::<String>(ImageProcessKeys::HASH, hash.clone());
        log::debug!("Hash computation complete, hash: {}", hash);
        Ok(())
    }
}

pub(super) struct GenerateThumbnailStep {
    services: Arc<ServiceProvider>,
    extractor: Arc<ThumbnailExtractor>,
}

impl GenerateThumbnailStep {
    pub(super) fn new(services: Arc<ServiceProvider>) -> Self {
        let extractor = services.get::<ThumbnailExtractor>();
        Self {
            services,
            extractor,
        }
    }

    fn output_file(&self, root: &Path, hash: &str) -> PathBuf {
        root.join(&hash[0..2]).join(&hash[2..4]).join(format!(
            "{}.{}",
            hash,
            ImageProcessKeys::THUMBNAIL_FORMAT_EXTENSION
        ))
    }
}

#[async_trait]
impl ImageProcessStep for GenerateThumbnailStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let thumbnail_root = context.payload().storage.normalized_path().join(".thumbnails");
        let hash = context
            .get_by_alias::<String>(ImageProcessKeys::HASH)
            .ok_or_else(|| anyhow!("hash not found"))?;

        let output_path = self.output_file(&thumbnail_root, hash);

        let extractor = Arc::clone(&self.extractor);
        let source = context.source_path().to_path_buf();
        let output = output_path.clone();
        task::spawn_blocking(move || {
            extractor.extract_to(source, &output)?;
            Result::<_, anyhow::Error>::Ok(())
        })
        .await
        .context("thumbnail generation join error")??;

        context.insert::<PathBuf>(ImageProcessKeys::THUMBNAIL_PATH, output_path.clone());
        log::debug!(
            "Thumbnail generation complete, output path: {}",
            output_path.display()
        );

        Ok(())
    }
}

pub(super) struct GeneratePreviewStep {
    services: Arc<ServiceProvider>,
    extractor: Arc<PreviewExtractor>,
}

impl GeneratePreviewStep {
    pub(super) fn new(services: Arc<ServiceProvider>) -> Self {
        let extractor = services.get::<PreviewExtractor>();
        Self {
            services,
            extractor,
        }
    }

    fn output_file(&self, root: &Path, hash: &str) -> PathBuf {
        root.join(&hash[0..2]).join(&hash[2..4]).join(format!(
            "{}.{}",
            hash,
            ImageProcessKeys::PREVIEW_FORMAT_EXTENSION
        ))
    }
}

#[async_trait]
impl ImageProcessStep for GeneratePreviewStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let preview_root = context.payload().storage.normalized_path().join(".previews");
        let hash = context
            .get_by_alias::<String>(ImageProcessKeys::HASH)
            .ok_or_else(|| anyhow!("hash not found"))?;

        let output_path = self.output_file(&preview_root, hash);

        let extractor = Arc::clone(&self.extractor);
        let source = context.source_path().to_path_buf();
        let output = output_path.clone();
        task::spawn_blocking(move || {
            extractor.extract_to(source, &output)?;
            Result::<_, anyhow::Error>::Ok(())
        })
        .await
        .context("preview generation join error")??;

        context.insert::<PathBuf>(ImageProcessKeys::PREVIEW_PATH, output_path.clone());
        log::debug!(
            "Preview generation complete, output path: {}",
            output_path.display()
        );

        Ok(())
    }
}

pub(super) struct CategorizeImageStep {
}

impl CategorizeImageStep {
    pub(super) fn new(_services: Arc<ServiceProvider>) -> Self {
        Self {}
    }
}

#[async_trait]
impl ImageProcessStep for CategorizeImageStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        let configured_template = context.payload().storage.category_template.trim();
        let category_template = if configured_template.is_empty() {
            "{year}/{date:%Y-%m-%d}/{fileName}"
        } else {
            configured_template
        };
        log::debug!("Categorizing image using template: {}", category_template);

        let working_directory = context
            .properties()
            .get_by_alias::<PathBuf>(ImageProcessKeys::WORKING_DIRECTORY);
        log::debug!(
            "Working directory for categorization: {}",
            working_directory
                .map(|dir| dir.display().to_string())
                .unwrap_or_else(|| "none".to_string())
        );

        let categorizer = TemplateCategorizer::new(category_template);
        let request = CategorizeRequest::new(context.source_path(), context.properties());
        let final_path = categorizer.categorize(&request)?.final_path;

        context.insert::<PathBuf>(ImageProcessKeys::FINAL_PATH, final_path.clone());

        log::debug!(
            "Image categorization complete, final path: {}",
            final_path.display()
        );

        Ok(())
    }
}

pub(super) struct PersistMetadataStep {
    services: Arc<ServiceProvider>,
    photo_repo: Arc<Repository<Photo>>,
    exif_repo: Arc<Repository<ExifModel>>,
}

impl PersistMetadataStep {
    pub(super) fn new(services: Arc<ServiceProvider>) -> Self {
        let photo_repo = services.get::<Repository<Photo>>();
        let exif_repo = services.get::<Repository<ExifModel>>();
        Self {
            services,
            photo_repo,
            exif_repo,
        }
    }
}

#[async_trait]
impl ImageProcessStep for PersistMetadataStep {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()> {
        log::debug!(
            "Persisting metadata to database for {}",
            context.source_path().display()
        );
        let final_path = context
            .get_by_alias::<PathBuf>(ImageProcessKeys::FINAL_PATH)
            .ok_or_else(|| anyhow!("final path not found in context"))?;
        let exif = context
            .get_by_alias::<ExifModel>(ImageProcessKeys::EXIF_METADATA)
            .ok_or_else(|| anyhow!("exif metadata not found in context"))?;
        let extension = final_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        let photo = Photo {
            id: Some(Uuid::new_v4()),
            storage_id: Some(context.payload().storage.id.clone()),
            path: final_path.to_string_lossy().to_string(),
            name: final_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("invalid file name"))?
                .to_string(),
            format: Some(extension.clone()),
            hash: Some(
                context
                    .get_by_alias::<String>(ImageProcessKeys::HASH)
                    .cloned()
                    .ok_or_else(|| anyhow!("hash not found in context"))?,
            ),
            size: Some(final_path.metadata()?.len() as i64),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            date_imported: Some(Utc::now()),
            date_taken: exif.get_date_taken(),
            thumbnail_path: Some(
                context
                    .get_by_alias::<PathBuf>(ImageProcessKeys::THUMBNAIL_PATH)
                    .cloned()
                    .ok_or_else(|| anyhow!("thumbnail path not found in context"))?
                    .to_str()
                    .ok_or_else(|| anyhow!("thumbnail path is not valid UTF-8"))?
                    .to_string(),
            ),
            thumbnail_optimized: Some(true),
            metadata_extracted: Some(true),
            is_raw: Some(
                ImageProcessKeys::RAW_EXTENSIONS
                    .iter()
                    .any(|candidate| candidate.eq_ignore_ascii_case(&extension)),
            ),
            width: exif.get_width(),
            height: exif.get_height(),
            thumbnail_width: context
                .get_by_alias::<u32>(ImageProcessKeys::THUMBNAIL_WIDTH)
                .cloned(),
            thumbnail_height: context
                .get_by_alias::<u32>(ImageProcessKeys::THUMBNAIL_HEIGHT)
                .cloned(),
            tags: None,
        };

        let saved_photo = self
            .photo_repo
            .insert(photo)
            .await
            .map_err(|err| anyhow!("failed to insert photo: {:?}", err))?;
        log::debug!("Photo metadata persisted with ID: {:?}", saved_photo.id);

        if let Some(mut metadata) = exif.clone().into() {
            metadata.id = Some(Uuid::new_v4());
            metadata.image_id = saved_photo.id;
            metadata.hash = saved_photo.hash.clone();

            let _ = self
                .exif_repo
                .insert(metadata)
                .await
                .map_err(|err| anyhow!("failed to insert exif metadata: {:?}", err))?;
        }

        log::debug!(
            "Processed image {} into storage {}",
            saved_photo.name,
            saved_photo.path
        );

        Ok(())
    }
}
