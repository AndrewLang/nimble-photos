use crate::prelude::*;
use serde_json::Value as JsonValue;

pub struct StorageService {
    storage_repo: Arc<Repository<StorageLocation>>,
    photo_repo: Arc<Repository<Photo>>,
    exif_repo: Arc<Repository<ExifModel>>,
}

impl StorageService {
    pub fn new(
        storage_repo: Arc<Repository<StorageLocation>>,
        photo_repo: Arc<Repository<Photo>>,
        exif_repo: Arc<Repository<ExifModel>>,
    ) -> Self {
        Self {
            storage_repo,
            photo_repo,
            exif_repo,
        }
    }

    pub async fn check_missing_files(
        &self,
        request: CheckFileRequest,
    ) -> Result<CheckFileResponse, PipelineError> {
        let storage_id = Uuid::parse_str(request.storage_id.trim())
            .map_err(|_| PipelineError::message("invalid storageId"))?;

        let storage = self
            .storage_repo
            .get(&storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| PipelineError::message("storage not found"))?;

        let requested_hashes = request
            .files
            .iter()
            .map(|file| Value::String(file.hash.clone()))
            .collect::<Vec<_>>();

        let existing_photos = if requested_hashes.is_empty() {
            Vec::new()
        } else {
            let query = QueryBuilder::<Photo>::new()
                .filter("storage_id", FilterOperator::Eq, Value::Uuid(storage.id))
                .filter("hash", FilterOperator::In, Value::List(requested_hashes))
                .build();

            self.photo_repo
                .all(query)
                .await
                .map_err(|_| PipelineError::message("failed to load existing photos"))?
        };

        let existing = existing_photos.into_iter().fold(
            HashMap::<String, HashSet<u64>>::new(),
            |mut acc, photo| {
                if let Some(hash) = photo.hash {
                    let sizes = acc.entry(hash).or_default();
                    if let Some(size) = photo.size.and_then(|value| u64::try_from(value).ok()) {
                        sizes.insert(size);
                    }
                }
                acc
            },
        );

        let missing_files = request
            .files
            .into_iter()
            .filter(|file| match existing.get(&file.hash) {
                Some(sizes) if !sizes.is_empty() => !sizes.contains(&file.file_size),
                Some(_) => false,
                None => true,
            })
            .collect::<Vec<_>>();

        Ok(CheckFileResponse { missing_files })
    }

    pub async fn sync_metadata(
        &self,
        request: SyncMetadataRequest,
    ) -> Result<ExifModel, PipelineError> {
        let storage_id = Uuid::parse_str(request.storage_id.trim())
            .map_err(|_| PipelineError::message("invalid storageId"))?;
        let hash = request.hash.trim();
        if hash.is_empty() {
            return Err(PipelineError::message("hash is required"));
        }

        self.storage_repo
            .get(&storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| PipelineError::message("storage not found"))?;

        let query = QueryBuilder::<Photo>::new()
            .filter("storage_id", FilterOperator::Eq, Value::Uuid(storage_id))
            .filter("hash", FilterOperator::Eq, Value::String(hash.to_string()))
            .build();

        let mut photos = self
            .photo_repo
            .all(query)
            .await
            .map_err(|_| PipelineError::message("failed to load existing photos"))?;
        let mut photo = photos
            .drain(..)
            .next()
            .ok_or_else(|| PipelineError::message("photo not found"))?;

        let existing_metadata = self
            .exif_repo
            .get_by("image_id", Value::Uuid(photo.id))
            .await
            .map_err(|_| PipelineError::message("failed to load metadata"))?;

        let metadata = self.build_metadata_model(existing_metadata.clone(), &photo, hash, request.metadata)?;
        self.apply_metadata_to_photo(&mut photo, &metadata);

        self.photo_repo
            .update(photo)
            .await
            .map_err(|_| PipelineError::message("failed to save photo metadata"))?;

        if existing_metadata.is_some() {
            self.exif_repo
                .update(metadata.clone())
                .await
                .map_err(|_| PipelineError::message("failed to save metadata"))?;
        } else {
            self.exif_repo
                .insert(metadata.clone())
                .await
                .map_err(|_| PipelineError::message("failed to save metadata"))?;
        }

        Ok(metadata)
    }

    fn build_metadata_model(
        &self,
        existing: Option<ExifModel>,
        photo: &Photo,
        hash: &str,
        metadata: JsonValue,
    ) -> Result<ExifModel, PipelineError> {
        let mut base_object = match existing {
            Some(existing) => serde_json::to_value(existing)
                .ok()
                .and_then(|value| value.as_object().cloned())
                .unwrap_or_default(),
            None => serde_json::Map::new(),
        };
        let patch_object = metadata
            .as_object()
            .cloned()
            .ok_or_else(|| PipelineError::message("metadata must be a JSON object"))?;

        for (key, value) in patch_object {
            base_object.insert(key, value);
        }

        base_object.insert("id".to_string(), json!(base_object
            .get("id")
            .and_then(|value| value.as_str())
            .and_then(|value| Uuid::parse_str(value).ok())
            .filter(|value| !value.is_nil())
            .unwrap_or_else(Uuid::new_v4)));
        base_object.insert("imageId".to_string(), json!(photo.id));
        base_object.insert("hash".to_string(), json!(hash));

        serde_json::from_value(JsonValue::Object(base_object))
            .map_err(|_| PipelineError::message("invalid metadata payload"))
    }

    fn apply_metadata_to_photo(&self, photo: &mut Photo, metadata: &ExifModel) {
        let now = Utc::now();
        photo.updated_at = Some(now);
        photo.metadata_extracted = Some(true);
        photo.artist = metadata.artist.clone();
        photo.make = metadata.make.clone();
        photo.model = metadata.model.clone();
        photo.lens_make = metadata.lens_make.clone();
        photo.lens_model = metadata.lens_model.clone();
        photo.exposure_time = metadata.exposure_time.clone();
        photo.iso = metadata.get_iso();
        photo.aperture = metadata.get_aperture();
        photo.focal_length = metadata.focal_length;
        photo.label = metadata.label.clone();
        photo.rating = metadata.rating;
        photo.flagged = metadata.flagged;
        photo.width = metadata.get_width();
        photo.height = metadata.get_height();
        photo.orientation = metadata.orientation;

        if let Some(date_taken) = metadata.get_date_taken() {
            photo.date_taken = Some(date_taken);
            photo.year = Some(date_taken.year());
            photo.month_day = Some(date_taken.format("%m-%d").to_string());
            photo.day_date = date_taken.date_naive();
            photo.sort_date = date_taken;
        }
    }
}
