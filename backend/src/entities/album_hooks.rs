use super::album::Album;
use async_trait::async_trait;
use chrono::Utc;
use nimble_web::entity::hooks::{EntityHooks, RequestContext};
use nimble_web::result::HttpError;
use nimble_web::result::Result as HttpResult;

use crate::services::IdGenerationService;

pub struct AlbumHooks;

impl AlbumHooks {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EntityHooks<Album> for AlbumHooks {
    async fn before_insert(&self, context: &RequestContext, entity: &mut Album) -> HttpResult<()> {
        let generator = context
            .services()
            .resolve::<IdGenerationService>()
            .ok_or_else(|| HttpError::new(500, "IdGenerationService is not registered"))?;

        if entity.id.is_nil() {
            entity.id = generator.generate();
        }

        if entity.create_date.is_none() {
            entity.create_date = Some(Utc::now());
        }
        Ok(())
    }
}
