use crate::models::setting_consts::SettingConsts;
use crate::models::{CategoryTemplateParser, PropertyMapTemplateContext};
use crate::prelude::*;
use anyhow::{Result, anyhow};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt, stream};
use serde_json::Value as JsonValue;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub struct SyncService {
    storage_repo: Arc<Repository<StorageLocation>>,
    photo_repo: Arc<Repository<Photo>>,
    exif_repo: Arc<Repository<ExifModel>>,
    file_service: Arc<FileService>,
    max_file_size: u64,
}

impl SyncService {
    const SYNC_ITEM_FIELD_NAME: &'static str = "item";
    const SYNC_FILE_FIELD_NAME: &'static str = "file";
    const FILES_FIELD_NAME: &'static str = "files";
    const UNKNOWN_FILE_BASENAME: &'static str = "upload";
    const DEFAULT_MAX_FILE_SIZE: u64 = 64 * 1024 * 1024;

    pub fn new(services: Arc<ServiceProvider>) -> Self {
        let config = services.get::<Configuration>();
        Self {
            storage_repo: services.get::<Repository<StorageLocation>>(),
            photo_repo: services.get::<Repository<Photo>>(),
            exif_repo: services.get::<Repository<ExifModel>>(),
            file_service: services.get::<FileService>(),
            max_file_size: config
                .get("upload.max_file_size_bytes")
                .or_else(|| config.get("upload.maxFileSizeBytes"))
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(Self::DEFAULT_MAX_FILE_SIZE),
        }
    }

    pub async fn check_missing_files(&self, request: CheckFileRequest) -> Result<CheckFileResponse, PipelineError> {
        let storage_id =
            Uuid::parse_str(request.storage_id.trim()).map_err(|_| PipelineError::message("invalid storageId"))?;

        let storage = self
            .storage_repo
            .get(&storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| PipelineError::message("storage not found"))?;

        let requested_hashes = request.files.iter().map(|file| Value::String(file.hash.clone())).collect::<Vec<_>>();

        let existing_photos = if requested_hashes.is_empty() {
            Vec::new()
        } else {
            let query = QueryBuilder::<Photo>::new()
                .filter("storage_id", FilterOperator::Eq, Value::Uuid(storage.id))
                .filter("hash", FilterOperator::In, Value::List(requested_hashes))
                .build();

            self.photo_repo.all(query).await.map_err(|_| PipelineError::message("failed to load existing photos"))?
        };

        let existing = existing_photos.into_iter().fold(HashMap::<String, HashSet<u64>>::new(), |mut acc, photo| {
            if let Some(hash) = photo.hash {
                let sizes = acc.entry(hash).or_default();
                if let Some(size) = photo.size.and_then(|value| u64::try_from(value).ok()) {
                    sizes.insert(size);
                }
            }
            acc
        });

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

    pub async fn sync_metadata(&self, request: SyncMetadataRequest) -> Result<ExifModel, PipelineError> {
        let storage_id =
            Uuid::parse_str(request.storage_id.trim()).map_err(|_| PipelineError::message("invalid storageId"))?;
        let image_id =
            Uuid::parse_str(request.image_id.trim()).map_err(|_| PipelineError::message("invalid imageId"))?;
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
            .filter("id", FilterOperator::Eq, Value::Uuid(image_id))
            .build();

        let mut photos =
            self.photo_repo.all(query).await.map_err(|_| PipelineError::message("failed to load existing photos"))?;
        let mut photo = photos.drain(..).next().ok_or_else(|| PipelineError::message("photo not found"))?;

        let existing_metadata = self
            .exif_repo
            .get_by("image_id", Value::Uuid(photo.id))
            .await
            .map_err(|_| PipelineError::message("failed to load metadata"))?;

        let metadata = self.build_metadata_model(existing_metadata.clone(), &photo, hash, request.metadata)?;
        self.apply_metadata_to_photo(&mut photo, &metadata);

        self.photo_repo.update(photo).await.map_err(|_| PipelineError::message("failed to save photo metadata"))?;

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

    pub async fn sync_file(&self, content_type: &str, body_bytes: Vec<u8>) -> Result<SyncFileResponse, PipelineError> {
        let item = self
            .parse_sync_item(content_type, body_bytes.clone())
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;
        let storage = self.load_storage(&item.storage_id).await?;

        log::info!("Syncing file {:?}", item);

        match item.asset_kind {
            SyncAssetKind::Image => self.sync_image(&storage, content_type, body_bytes).await,
            SyncAssetKind::Preview | SyncAssetKind::Thumbnail => {
                self.sync_thumb(&storage, content_type, body_bytes).await
            }
        }
    }

    async fn sync_image(
        &self,
        storage: &StorageLocation,
        content_type: &str,
        body_bytes: Vec<u8>,
    ) -> Result<SyncFileResponse, PipelineError> {
        let item = self
            .parse_sync_item(content_type, body_bytes.clone())
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;
        let mut photo = self.ensure_photo(storage, &item).await?;
        let final_path = self
            .image_output_path(storage, &photo, &item)
            .map_err(|error| PipelineError::message(&error.to_string()))?;
        let saved_file = self
            .persist_sync_file_to_path(content_type, body_bytes, &final_path)
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;

        let final_relative_path = self
            .file_service
            .relative_path(&storage.normalized_path(), &final_path)
            .map_err(|error| PipelineError::message(&error.to_string()))?;
        let byte_size = i64::try_from(saved_file.byte_size)
            .map_err(|_| PipelineError::message("file size exceeds supported range"))?;
        photo.path = final_relative_path.clone();
        photo.name = final_path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_string();
        photo.format = final_path.extension().and_then(|value| value.to_str()).map(|value| value.to_string());
        photo.size = Some(byte_size);
        photo.updated_at = Some(Utc::now());

        self.photo_repo
            .update(photo.clone())
            .await
            .map_err(|_| PipelineError::message("failed to save photo file metadata"))?;

        Ok(SyncFileResponse {
            image_id: photo.id.to_string(),
            storage_id: item.storage_id.clone(),
            hash: item.hash.clone(),
            asset_kind: item.asset_kind.clone(),
            file: UploadFileResponse {
                file_name: photo.name.clone(),
                relative_path: final_relative_path,
                byte_size: saved_file.byte_size,
                content_type: saved_file.content_type,
            },
        })
    }

    async fn sync_thumb(
        &self,
        storage: &StorageLocation,
        content_type: &str,
        body_bytes: Vec<u8>,
    ) -> Result<SyncFileResponse, PipelineError> {
        let item = self
            .parse_sync_item(content_type, body_bytes.clone())
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;
        let final_path =
            self.asset_output_path(storage, &item).map_err(|error| PipelineError::message(&error.to_string()))?;
        let saved_file = self
            .persist_sync_file_to_path(content_type, body_bytes, &final_path)
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;
        let final_relative_path = self
            .file_service
            .relative_path(&storage.normalized_path(), &final_path)
            .map_err(|error| PipelineError::message(&error.to_string()))?;

        Ok(SyncFileResponse {
            image_id: "".to_string(),
            storage_id: item.storage_id.clone(),
            hash: item.hash.clone(),
            asset_kind: item.asset_kind.clone(),
            file: UploadFileResponse {
                file_name: final_path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_string(),
                relative_path: final_relative_path,
                byte_size: saved_file.byte_size,
                content_type: saved_file.content_type,
            },
        })
    }

    async fn parse_sync_item(&self, content_type: &str, body_bytes: Vec<u8>) -> Result<SyncFileItem> {
        let boundary = multer::parse_boundary(content_type)?;
        let body_stream = stream::once(async move { Ok::<Bytes, std::io::Error>(Bytes::from(body_bytes)) });
        let mut multipart = multer::Multipart::new(body_stream, boundary);

        while let Some(field) = multipart.next_field().await? {
            if field.name() == Some(Self::SYNC_ITEM_FIELD_NAME) {
                let raw = field.text().await?;
                return Ok(serde_json::from_str::<SyncFileItem>(&raw)?);
            }
        }

        Err(anyhow!("missing multipart field 'item'"))
    }

    async fn persist_sync_file_to_path(
        &self,
        content_type: &str,
        body_bytes: Vec<u8>,
        destination_path: &Path,
    ) -> Result<StoredUploadFile> {
        let boundary = multer::parse_boundary(content_type)?;
        let body_stream = stream::once(async move { Ok::<Bytes, std::io::Error>(Bytes::from(body_bytes)) });
        let mut multipart = multer::Multipart::new(body_stream, boundary);

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut item: Option<SyncFileItem> = None;
        let mut stored_file: Option<StoredUploadFile> = None;

        while let Some(field) = multipart.next_field().await? {
            match field.name() {
                Some(Self::SYNC_ITEM_FIELD_NAME) => {
                    let raw = field.text().await?;
                    item = Some(serde_json::from_str::<SyncFileItem>(&raw)?);
                }
                Some(Self::SYNC_FILE_FIELD_NAME) | Some(Self::FILES_FIELD_NAME) => {
                    let sync_item =
                        item.as_ref().ok_or_else(|| anyhow!("multipart field 'item' must be sent before file"))?;

                    if fs::try_exists(destination_path).await? {
                        fs::remove_file(destination_path).await?;
                    }

                    let bytes_written =
                        self.write_stream_to_file(field.into_stream(), destination_path).await.map_err(|error| {
                            anyhow!("failed to persist sync upload '{}': {}", destination_path.display(), error)
                        })?;

                    if bytes_written != sync_item.file_size {
                        let _ = fs::remove_file(destination_path).await;
                        return Err(anyhow!(
                            "uploaded file size {} does not match expected size {}",
                            bytes_written,
                            sync_item.file_size
                        ));
                    }

                    let final_file_name = destination_path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or(Self::UNKNOWN_FILE_BASENAME)
                        .to_string();
                    let relative_path = destination_path
                        .components()
                        .map(|value| value.as_os_str().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join("/");

                    stored_file = Some(StoredUploadFile {
                        file_name: final_file_name,
                        relative_path,
                        byte_size: bytes_written as usize,
                        content_type: sync_item.content_type.clone(),
                    });
                }
                _ => {}
            }
        }

        let _item = item.ok_or_else(|| anyhow!("missing multipart field 'item'"))?;
        let stored_file = stored_file.ok_or_else(|| anyhow!("missing multipart field 'file'"))?;
        Ok(stored_file)
    }

    async fn write_stream_to_file<S>(&self, mut stream: S, path: &Path) -> Result<u64>
    where
        S: futures_util::Stream<Item = Result<Bytes, multer::Error>> + Unpin,
    {
        let mut file = File::create_new(path).await?;
        let mut bytes_written = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            bytes_written =
                bytes_written.checked_add(chunk.len() as u64).ok_or_else(|| anyhow!("uploaded file size overflow"))?;

            if bytes_written > self.max_file_size {
                drop(file);
                let _ = fs::remove_file(path).await;
                return Err(anyhow!("uploaded file exceeds max allowed size of {} bytes", self.max_file_size));
            }

            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        Ok(bytes_written)
    }

    fn sanitize_file_name(file_name: &str) -> String {
        let base_name = Path::new(file_name)
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| Self::UNKNOWN_FILE_BASENAME.to_string());
        let sanitized = base_name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '.' || character == '-' || character == '_' {
                    character
                } else {
                    '_'
                }
            })
            .collect::<String>();
        if sanitized.is_empty() { Self::UNKNOWN_FILE_BASENAME.to_string() } else { sanitized }
    }

    async fn load_photo_by_hash(&self, storage_id: Uuid, hash: &str) -> Result<Option<Photo>, PipelineError> {
        let query = QueryBuilder::<Photo>::new()
            .filter("storage_id", FilterOperator::Eq, Value::Uuid(storage_id))
            .filter("hash", FilterOperator::Eq, Value::String(hash.trim().to_string()))
            .build();

        let mut photos =
            self.photo_repo.all(query).await.map_err(|_| PipelineError::message("failed to load existing photos"))?;
        Ok(photos.drain(..).next())
    }

    async fn ensure_photo(&self, storage: &StorageLocation, item: &SyncFileItem) -> Result<Photo, PipelineError> {
        if let Some(photo) = self.load_photo_by_hash(storage.id, &item.hash).await? {
            return Ok(photo);
        }

        let mut photo = Photo::default();
        photo.storage_id = storage.id;
        photo.name = item.file_name.clone();
        photo.format =
            Path::new(&item.file_name).extension().and_then(|value| value.to_str()).map(|value| value.to_string());
        photo.hash = Some(item.hash.clone());

        self.photo_repo.insert(photo).await.map_err(|_| PipelineError::message("failed to create photo"))?;

        self.load_photo_by_hash(storage.id, &item.hash)
            .await?
            .ok_or_else(|| PipelineError::message("failed to load created photo"))
    }

    async fn load_storage(&self, storage_id: &str) -> Result<StorageLocation, PipelineError> {
        let parsed_storage_id =
            Uuid::parse_str(storage_id.trim()).map_err(|_| PipelineError::message("invalid storageId"))?;
        let storage = self
            .storage_repo
            .get(&parsed_storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| PipelineError::message("storage not found"))?;
        if storage.is_readonly {
            return Err(PipelineError::message("storage is readonly"));
        }
        Ok(storage)
    }

    fn image_output_path(&self, storage: &StorageLocation, photo: &Photo, item: &SyncFileItem) -> Result<PathBuf> {
        let properties = self.get_photo_properties(photo, item);

        let parser = CategoryTemplateParser::new(storage.category_template.clone())?;
        let relative = parser.render(&PropertyMapTemplateContext::new(properties))?;

        Ok(storage.normalized_path().join(relative))
    }

    fn get_photo_properties(&self, photo: &Photo, item: &SyncFileItem) -> PropertyMap {
        let mut properties = PropertyMap::new();
        properties
            .insert::<DateTime<Utc>>(photo.date_taken.clone().or(photo.created_at.clone()).unwrap_or(photo.sort_date))
            .alias("effective_date");
        properties.insert::<String>(item.file_name.clone()).alias("file_name");

        let hash = item.hash.trim();
        if !hash.is_empty() {
            properties.insert::<String>(hash.to_string()).alias("hash");
        }

        if let Some(model) = photo.model.as_ref() {
            properties.insert::<String>(model.clone()).alias("camera");
        }

        if let Some(rating) = photo.rating {
            properties.insert::<i32>(rating as i32).alias("rating");
        }

        properties
    }

    fn asset_output_path(&self, storage: &StorageLocation, item: &SyncFileItem) -> Result<PathBuf> {
        let hash = item.hash.trim();
        if hash.len() < 4 {
            return Err(anyhow!("hash must be at least 4 characters"));
        }

        let (base_folder, extension) = match item.asset_kind {
            SyncAssetKind::Image => {
                return Err(anyhow!("image assets should not be resolved with hashed output path"));
            }
            SyncAssetKind::Preview => (SettingConsts::PREVIEW_FOLDER, SettingConsts::PREVIEW_FORMAT),
            SyncAssetKind::Thumbnail => (SettingConsts::THUMBNAIL_FOLDER, SettingConsts::THUMBNAIL_FORMAT),
        };

        Ok(self.file_service.path_for_hash(storage.normalized_path().join(base_folder), hash, extension))
    }

    fn build_metadata_model(
        &self,
        existing: Option<ExifModel>,
        photo: &Photo,
        hash: &str,
        metadata: JsonValue,
    ) -> Result<ExifModel, PipelineError> {
        let mut base_object = match existing {
            Some(existing) => {
                serde_json::to_value(existing).ok().and_then(|value| value.as_object().cloned()).unwrap_or_default()
            }
            None => serde_json::Map::new(),
        };
        let patch_object =
            metadata.as_object().cloned().ok_or_else(|| PipelineError::message("metadata must be a JSON object"))?;

        for (key, value) in patch_object {
            base_object.insert(key, value);
        }

        base_object.insert(
            "id".to_string(),
            json!(
                base_object
                    .get("id")
                    .and_then(|value| value.as_str())
                    .and_then(|value| Uuid::parse_str(value).ok())
                    .filter(|value| !value.is_nil())
                    .unwrap_or_else(Uuid::new_v4)
            ),
        );
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
