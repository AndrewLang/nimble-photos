use crate::prelude::*;

pub struct StorageService {
    storage_repo: Arc<Repository<StorageLocation>>,
    photo_repo: Arc<Repository<Photo>>,
}

impl StorageService {
    pub fn new(
        storage_repo: Arc<Repository<StorageLocation>>,
        photo_repo: Arc<Repository<Photo>>,
    ) -> Self {
        Self {
            storage_repo,
            photo_repo,
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
}
