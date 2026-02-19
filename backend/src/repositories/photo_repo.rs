use async_trait::async_trait;

use crate::entities::photo::Photo;

use nimble_web::DataProvider;
use nimble_web::data::query::Value;
use nimble_web::data::repository::Repository;
use nimble_web::pipeline::pipeline::PipelineError;

#[async_trait]
pub trait PhotoRepositoryExtensions {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Photo>, PipelineError>;
}

#[async_trait]
impl PhotoRepositoryExtensions for Repository<Photo> {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Photo>, PipelineError> {
        self.get_by("hash", Value::String(hash.to_string()))
            .await
            .map_err(|_| PipelineError::message("failed to load photo by hash"))
    }
}
