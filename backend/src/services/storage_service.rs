use crate::models::setting_consts::SettingConsts;
use crate::prelude::*;
use crate::services::image_pipeline::DerivativeProcessPayload;

pub struct StorageService {
    storage_repo: Arc<Repository<StorageLocation>>,
    photo_repo: Arc<Repository<Photo>>,
    file_service: Arc<FileService>,
    image_pipeline: Arc<ImageProcessPipeline>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStorageResponse {
    pub storage_id: Uuid,
    pub scanned_count: usize,
    pub generated_thumbnail_count: usize,
    pub generated_preview_count: usize,
    pub skipped_count: usize,
}

impl StorageService {
    pub fn new(services: Arc<ServiceProvider>) -> Self {
        Self {
            storage_repo: services.get::<Repository<StorageLocation>>(),
            photo_repo: services.get::<Repository<Photo>>(),
            file_service: services.get::<FileService>(),
            image_pipeline: services.get::<ImageProcessPipeline>(),
        }
    }

    pub async fn scan(&self, storage_id: Uuid) -> Result<ScanStorageResponse, PipelineError> {
        let storage = self
            .storage_repo
            .get(&storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| PipelineError::message("storage not found"))?;

        let photos = self
            .photo_repo
            .all(QueryBuilder::<Photo>::new().filter("storage_id", FilterOperator::Eq, Value::Uuid(storage_id)).build())
            .await
            .map_err(|_| PipelineError::message("failed to load photos"))?;

        log::info!("Starting scan for storage location with id: {}, {} photos found", storage_id, photos.len());

        let mut derivative_requests = Vec::<DerivativeProcessPayload>::new();
        let mut scanned_count = 0usize;
        let mut generated_thumbnail_count = 0usize;
        let mut generated_preview_count = 0usize;
        let mut skipped_count = 0usize;

        for photo in photos {
            let Some(hash) = photo.hash.as_deref().filter(|value| value.len() >= 4) else {
                skipped_count += 1;
                continue;
            };

            let source_path = self.resolve_photo_source_path(&storage, &photo);
            if !source_path.exists() {
                log::warn!("Skipping scan for photo {} because source is missing: {}", photo.id, source_path.display());
                skipped_count += 1;
                continue;
            }

            let thumbnail_path = self.file_service.path_for_hash(
                storage.normalized_path().join(SettingConsts::THUMBNAIL_FOLDER),
                hash,
                SettingConsts::THUMBNAIL_FORMAT,
            );
            let preview_path = self.file_service.path_for_hash(
                storage.normalized_path().join(SettingConsts::PREVIEW_FOLDER),
                hash,
                SettingConsts::PREVIEW_FORMAT,
            );

            let needs_thumbnail = !thumbnail_path.exists();
            let needs_preview = !preview_path.exists();

            if !needs_thumbnail && !needs_preview {
                skipped_count += 1;
                continue;
            }

            scanned_count += 1;
            derivative_requests.push(DerivativeProcessPayload {
                storage: storage.clone(),
                relative_path: photo.path.clone(),
                file_name: photo.name.clone(),
                hash: hash.to_string(),
                generate_thumbnail: needs_thumbnail,
                generate_preview: needs_preview,
            });

            if needs_thumbnail {
                generated_thumbnail_count += 1;
            }

            if needs_preview {
                generated_preview_count += 1;
            }
        }

        self.image_pipeline
            .enqueue_derivative_batch(derivative_requests)
            .map_err(|error| PipelineError::message(&format!("failed to schedule derivative processing: {}", error)))?;

        Ok(ScanStorageResponse {
            storage_id,
            scanned_count,
            generated_thumbnail_count,
            generated_preview_count,
            skipped_count,
        })
    }

    fn resolve_photo_source_path(&self, storage: &StorageLocation, photo: &Photo) -> PathBuf {
        let photo_path = PathBuf::from(&photo.path);
        if photo_path.is_absolute() { photo_path } else { storage.normalized_path().join(photo_path) }
    }
}
