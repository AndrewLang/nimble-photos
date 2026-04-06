use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::prelude::*;

pub struct AlbumController;

const MAX_COMMENT_LENGTH: usize = 1024;

impl Controller for AlbumController {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}

struct AlbumPhotosHandler;

#[async_trait]
#[get("/api/albums/{id}/photos/{page}/{pageSize}")]
impl HttpHandler for AlbumPhotosHandler {
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
        let id = context.entity_id()?;
        let page: u32 = context.page().unwrap_or(1);
        let page_size: u32 = context.page_size().unwrap_or(20);
        let repository = context.service::<Repository<Photo>>()?;
        let paged_photos = repository.photos_in_album(id, page, page_size).await?;

        Ok(ResponseValue::json(paged_photos))
    }
}

struct ListAlbumsHandler;

#[async_trait]
#[get("/api/albums/{page}/{pageSize}")]
impl HttpHandler for ListAlbumsHandler {
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
        let page: u32 = context.page().unwrap_or(1);
        let page_size: u32 = context.page_size().unwrap_or(20);
        let repository = context.service::<Repository<Album>>()?;

        let query = QueryBuilder::<Album>::new().page(page, page_size).build();

        let albums = repository
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(albums))
    }
}

struct AlbumCommentsHandler;

#[derive(Deserialize)]
struct AlbumPhotoIdsPayload {
    #[serde(rename = "photoIds")]
    photo_ids: Vec<Uuid>,
}

struct AddAlbumPhotosHandler;

#[async_trait]
#[post("/api/albums/{id}/photos", policy = Policy::Authenticated)]
impl HttpHandler for AddAlbumPhotosHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let album_id = context.entity_id()?;
        let payload = context
            .read_json::<AlbumPhotoIdsPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        let photo_ids = payload.photo_ids;
        let repository = context.service::<Repository<AlbumPhoto>>()?;
        let added = repository
            .add_photos_to_album(album_id, &photo_ids)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(json!({ "updated": added }))))
    }
}

struct RemoveAlbumPhotosHandler;

#[async_trait]
#[delete("/api/albums/{id}/photos", policy = Policy::Authenticated)]
impl HttpHandler for RemoveAlbumPhotosHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let album_id = context.entity_id()?;
        let payload = context
            .read_json::<AlbumPhotoIdsPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;
        let photo_ids = payload.photo_ids;
        let repository = context.service::<Repository<AlbumPhoto>>()?;
        let removed = repository
            .remove_photos_from_album(album_id, &photo_ids)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        Ok(ResponseValue::new(Json(json!({ "updated": removed }))))
    }
}

#[async_trait]
#[get("/api/album/comments/{id}")]
impl HttpHandler for AlbumCommentsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let album_id = context.entity_id()?;
        let is_admin = context.is_admin();

        log::info!("Fetching comments for album {}", album_id);

        let repository = context.service::<Repository<AlbumComment>>()?;
        let allow_hidden = is_admin;

        let query = QueryBuilder::<AlbumComment>::new()
            .filter("album_id", FilterOperator::Eq, Value::Uuid(album_id))
            .filter("hidden", FilterOperator::Eq, Value::Bool(allow_hidden))
            .sort_desc("created_at")
            .build();
        let comments = repository
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(comments))
    }
}

#[derive(Deserialize)]
struct CreateAlbumCommentPayload {
    comment: String,
}

struct CreateAlbumCommentHandler;

impl CreateAlbumCommentHandler {
    fn validate_comment(&self, comment: &str) -> Result<String, PipelineError> {
        let trimmed = comment.trim();
        if trimmed.is_empty() {
            return Err(PipelineError::message("Comment cannot be empty"));
        }
        if trimmed.chars().count() > MAX_COMMENT_LENGTH {
            return Err(PipelineError::message(&format!(
                "Comment must be {} characters or fewer",
                MAX_COMMENT_LENGTH
            )));
        }
        Ok(trimmed.to_string())
    }
}

#[async_trait]
#[post("/api/album/comments/{id}", policy = Policy::Authenticated)]
impl HttpHandler for CreateAlbumCommentHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<CreateAlbumCommentPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        let comment = self.validate_comment(&payload.comment)?;
        let album_id = context.entity_id()?;
        let user_id = context.current_user_id()?;

        let settings_repo = context.service::<Repository<UserSettings>>()?;
        let display_name = settings_repo
            .get(&user_id)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .map(|settings| settings.display_name)
            .unwrap_or_else(|| "Anonymous".to_string());

        let new_comment = AlbumComment::new(album_id, user_id, display_name, comment);
        let repository = context.service::<Repository<AlbumComment>>()?;
        let saved = repository
            .insert(new_comment)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(AlbumCommentDto::from(saved)))
    }
}

#[derive(Deserialize)]
struct UpdateAlbumCommentVisibilityPayload {
    hidden: bool,
}

struct UpdateAlbumCommentVisibilityHandler;

#[async_trait]
#[put("/api/album/comments/visibility/{albumId}/{commentId}", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for UpdateAlbumCommentVisibilityHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let album_id = context.id("albumId")?;
        let comment_id = context.id("commentId")?;
        let payload = context
            .read_json::<UpdateAlbumCommentVisibilityPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        let repository = context.service::<Repository<AlbumComment>>()?;
        let mut comment = repository
            .get(&comment_id)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .ok_or_else(|| PipelineError::message("Comment not found"))?;

        if comment.album_id != album_id {
            return Err(PipelineError::message(
                "Comment does not belong to the supplied album",
            ));
        }

        comment.hidden = payload.hidden;

        let saved = repository
            .update(comment)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(AlbumCommentDto::from(saved))))
    }
}
