use async_trait::async_trait;
use uuid::Uuid;

use crate::entities::album_photo::AlbumPhoto;
use crate::entities::photo::Photo;

use nimble_web::DataProvider;
use nimble_web::data::paging::Page;
use nimble_web::data::query::{FilterOperator, Value};
use nimble_web::data::query_builder::QueryBuilder;
use nimble_web::data::repository::Repository;
use nimble_web::pipeline::pipeline::PipelineError;

#[async_trait]
pub trait PhotoRepositoryExtensions {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Photo>, PipelineError>;

    async fn photos_in_album(
        &self,
        album_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Photo>, PipelineError>;
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
}
