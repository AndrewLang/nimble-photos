use async_trait::async_trait;
use std::path::Path;
use uuid::Uuid;

use crate::entities::album_photo::AlbumPhoto;
use crate::entities::exif::ExifModel;
use crate::entities::photo::Photo;
use crate::entities::photo_comment::PhotoComment;
use crate::entities::storage_location::StorageLocation;
use crate::models::setting_consts::SettingConsts;
use crate::services::FileService;

use nimble_web::data::paging::Page;
use nimble_web::data::query::{FilterOperator, Value};
use nimble_web::data::query_builder::QueryBuilder;
use nimble_web::data::repository::Repository;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::{DataProvider, HttpContext};

#[async_trait]
pub trait PhotoRepositoryExtensions {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Photo>, PipelineError>;

    async fn photos_in_album(
        &self,
        album_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Photo>, PipelineError>;

    async fn delete_photo(
        &self,
        context: &HttpContext,
        photo: &Photo,
    ) -> Result<u32, PipelineError>;

    async fn delete_file(&self, photo: &Photo, context: &HttpContext) -> Result<(), PipelineError>;

    async fn delete_records(
        &self,
        photo: &Photo,
        context: &HttpContext,
    ) -> Result<(), PipelineError>;
}

#[async_trait]
impl PhotoRepositoryExtensions for Repository<Photo> {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Photo>, PipelineError> {
        self.get_by("hash", Value::String(hash.to_string()))
            .await
            .map_err(|_| PipelineError::message("failed to load photo by hash"))
    }

    async fn photos_in_album(
        &self,
        album_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Photo>, PipelineError> {
        let query = QueryBuilder::<Photo>::new()
            .join::<AlbumPhoto>("photo_id", "id")
            .filter("album_id", FilterOperator::Eq, Value::Uuid(album_id))
            .page(page, page_size)
            .build();

        self.query(query)
            .await
            .map_err(|_| PipelineError::message("failed to load photos in album"))
    }

    async fn delete_photo(
        &self,
        context: &HttpContext,
        photo: &Photo,
    ) -> Result<u32, PipelineError> {
        self.delete_file(photo, context).await?;
        self.delete_records(photo, context).await?;

        Ok(1)
    }

    async fn delete_records(
        &self,
        photo: &Photo,
        context: &HttpContext,
    ) -> Result<(), PipelineError> {
        let photo_repo = context.service::<Repository<Photo>>()?;
        let album_photo_repo = context.service::<Repository<AlbumPhoto>>()?;
        let exif_repo = context.service::<Repository<ExifModel>>()?;
        let photo_comment_repo = context.service::<Repository<PhotoComment>>()?;

        photo_repo.delete(&photo.id).await.map_err(|e| {
            PipelineError::message(&format!("failed to delete photo record: {:?}", e))
        })?;
        exif_repo
            .delete_by("image_id", Value::Uuid(photo.id))
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to delete exif record: {:?}", e))
            })?;
        photo_comment_repo
            .delete_by("photo_id", Value::Uuid(photo.id))
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to delete photo comments: {:?}", e))
            })?;
        album_photo_repo
            .delete_by("photo_id", Value::Uuid(photo.id))
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to delete album_photo records: {:?}", e))
            })?;

        Ok(())
    }

    async fn delete_file(&self, photo: &Photo, context: &HttpContext) -> Result<(), PipelineError> {
        let file_service = context.service::<FileService>()?;
        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let hash = photo
            .hash
            .as_ref()
            .ok_or_else(|| PipelineError::message("Photo hash is missing"))?;

        let storage = storage_repo
            .get(&photo.storage_id)
            .await
            .map_err(|_| PipelineError::message("Storage location not found"))?
            .ok_or_else(|| PipelineError::message("Storage is not found"))?;

        let root = Path::new(&storage.path);

        let thumbnail_path = file_service.path_for_hash(
            root.join(SettingConsts::THUMBNAIL_FOLDER),
            &hash,
            SettingConsts::THUMBNAIL_FORMAT,
        );
        let _ = file_service.remove_file(&thumbnail_path);

        let preview_path = file_service.path_for_hash(
            root.join(SettingConsts::PREVIEW_FOLDER),
            &hash,
            SettingConsts::PREVIEW_FORMAT,
        );
        let _ = file_service.remove_file(&preview_path);

        Ok(())
    }
}
