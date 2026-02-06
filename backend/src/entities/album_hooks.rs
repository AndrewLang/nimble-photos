use super::album::Album;
use async_trait::async_trait;
use chrono::Utc;
use nimble_web::entity::hooks::{EntityHooks, RequestContext};
use nimble_web::result::HttpError;
use nimble_web::result::Result as HttpResult;

use crate::repositories::photo::PhotoRepository;
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

        if entity.id.is_none() {
            entity.id = Some(generator.generate());
        }

        if entity.create_date.is_none() {
            entity.create_date = Some(Utc::now());
        }

        self.sync_metadata(context, entity).await?;
        Ok(())
    }

    async fn before_update(&self, context: &RequestContext, entity: &mut Album) -> HttpResult<()> {
        self.sync_metadata(context, entity).await?;
        Ok(())
    }
}

impl AlbumHooks {
    async fn sync_metadata(&self, context: &RequestContext, entity: &mut Album) -> HttpResult<()> {
        if let Some(rules_json) = &entity.rules_json {
            if let Ok(rules) = serde_json::from_str::<serde_json::Value>(rules_json) {
                if let Some(photo_ids) = rules.get("photoIds").and_then(|v| v.as_array()) {
                    entity.image_count = Some(photo_ids.len() as i64);

                    if !photo_ids.is_empty() {
                        if let Some(first_id_val) = photo_ids.first() {
                            if let Some(first_id_str) = first_id_val.as_str() {
                                if let Ok(first_id) = uuid::Uuid::parse_str(first_id_str) {
                                    if let Some(repo) =
                                        context.services().resolve::<Box<dyn PhotoRepository>>()
                                    {
                                        if let Ok(photos) = repo.get_by_ids(&[first_id], true).await
                                        {
                                            if let Some(photo) = photos.first() {
                                                entity.thumbnail_hash = photo.hash.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        entity.thumbnail_hash = None;
                    }
                }
            }
        }
        Ok(())
    }
}
