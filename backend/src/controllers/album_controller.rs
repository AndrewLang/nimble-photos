use crate::dtos::album_comment_dto::AlbumCommentDto;
use crate::entities::album::Album;
use crate::entities::album_comment::AlbumComment;
use crate::entities::user_settings::UserSettings;
use crate::repositories::photo::{PhotoRepository, TagRef};
use crate::services::SettingService;

use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

use nimble_web::controller::controller::Controller;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::{Filter, FilterOperator, Query, Sort, SortDirection, Value};
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::Json;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct AlbumController;

const MAX_COMMENT_LENGTH: usize = 1024;

impl Controller for AlbumController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get(
                "/api/albums/{id}/photos/{page}/{pageSize}",
                AlbumPhotosHandler,
            )
            .build(),
            EndpointRoute::get("/api/albums/{page}/{pageSize}", ListAlbumsHandler).build(),
            EndpointRoute::get("/api/albums/{id}/tags", AlbumTagsHandler).build(),
            EndpointRoute::put("/api/albums/{id}/tags", ReplaceAlbumTagsHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::get("/api/album/comments/{id}", AlbumCommentsHandler).build(),
            EndpointRoute::post("/api/album/comments/{id}", CreateAlbumCommentHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::put(
                "/api/album/comments/visibility/{albumId}/{commentId}",
                UpdateAlbumCommentVisibilityHandler,
            )
            .with_policy(Policy::InRole("admin".to_string()))
            .build(),
        ]
    }
}

struct AlbumPhotosHandler;

#[derive(Deserialize)]
struct AlbumRules {
    #[serde(rename = "photoIds")]
    photo_ids: Vec<Uuid>,
}

#[async_trait]
impl HttpHandler for AlbumPhotosHandler {
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
        let route_params = context.route().map(|r| r.params());

        let id_str = route_params
            .and_then(|p| p.get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;

        let page: u32 = route_params
            .and_then(|p| p.get("page"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let page_size: u32 = route_params
            .and_then(|p| p.get("pageSize"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        let id = Uuid::parse_str(id_str).map_err(|_| PipelineError::message("invalid album id"))?;

        let album_repo = context.service::<Repository<Album>>()?;

        let album = album_repo
            .get(&id)
            .await
            .map_err(|e| PipelineError::message(&format!("Failed to fetch album: {:?}", e)))?
            .ok_or_else(|| PipelineError::message("Album not found"))?;

        let mut photos = Vec::new();
        let mut total = 0u64;

        if let Some(rules_json) = album.rules_json {
            if let Ok(rules) = serde_json::from_str::<AlbumRules>(&rules_json) {
                total = rules.photo_ids.len() as u64;

                let start = ((page - 1) * page_size) as usize;
                let mut end = start + page_size as usize;
                if end > rules.photo_ids.len() {
                    end = rules.photo_ids.len();
                }

                if start < rules.photo_ids.len() {
                    let slice = &rules.photo_ids[start..end];
                    let photo_repo = context.service::<Box<dyn PhotoRepository>>()?;
                    let can_view_admin_only = context
                        .get::<IdentityContext>()
                        .map(|ctx| ctx.identity().claims().roles().contains("admin"))
                        .unwrap_or(false);

                    photos = photo_repo
                        .get_by_ids(slice, can_view_admin_only)
                        .await
                        .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
                    let hidden_tags = AlbumController::viewer_hidden_tags(context).await?;
                    photos = AlbumController::filter_photos_for_viewer(photos, &hidden_tags);
                }
            }
        }

        let response = serde_json::json!({
            "page": page,
            "page_size": page_size,
            "total": total,
            "items": photos
        });

        Ok(ResponseValue::new(Json(response)))
    }
}

struct ListAlbumsHandler;

#[async_trait]
impl HttpHandler for ListAlbumsHandler {
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
        let pool = context.service::<PgPool>()?;
        let route_params = context.route().map(|r| r.params());

        let page: u32 = route_params
            .and_then(|p| p.get("page"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let page_size: u32 = route_params
            .and_then(|p| p.get("pageSize"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        let offset = if page > 0 { (page - 1) * page_size } else { 0 };

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM albums")
            .fetch_one(&*pool)
            .await
            .map_err(|e| PipelineError::message(&format!("Failed to count albums: {:?}", e)))?;

        let albums = sqlx::query_as::<_, Album>(
            "SELECT * FROM albums ORDER BY create_date DESC LIMIT $1 OFFSET $2",
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&*pool)
        .await
        .map_err(|e| PipelineError::message(&format!("Failed to fetch albums: {:?}", e)))?;

        let response = serde_json::json!({
            "page": page,
            "pageSize": page_size,
            "total": total,
            "items": albums
        });

        Ok(ResponseValue::new(Json(response)))
    }
}

struct AlbumCommentsHandler;

#[derive(Deserialize)]
struct ReplaceAlbumTagsPayload {
    tags: Vec<serde_json::Value>,
}

struct AlbumTagsHandler;

#[async_trait]
impl HttpHandler for AlbumTagsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let album_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let album_id = Uuid::parse_str(album_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid album id: {}", e)))?;

        let repository = context.service::<Box<dyn PhotoRepository>>()?;
        let tags = repository
            .get_album_tags(album_id, AlbumController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(tags)))
    }
}

struct ReplaceAlbumTagsHandler;

#[async_trait]
impl HttpHandler for ReplaceAlbumTagsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let album_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let album_id = Uuid::parse_str(album_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid album id: {}", e)))?;

        let payload = context
            .read_json::<ReplaceAlbumTagsPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;
        let refs = AlbumController::to_tag_refs(&payload.tags)?;
        let current_user_id = context
            .get::<IdentityContext>()
            .and_then(|ctx| Uuid::parse_str(ctx.identity().subject()).ok())
            .ok_or_else(|| PipelineError::message("invalid identity"))?;

        let repository = context.service::<Box<dyn PhotoRepository>>()?;
        repository
            .set_album_tags(album_id, &refs, Some(current_user_id))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let tags = repository
            .get_album_tags(album_id, AlbumController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(tags)))
    }
}

#[async_trait]
impl HttpHandler for AlbumCommentsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let album_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let album_id = Uuid::parse_str(album_id_param)
            .map_err(|_| PipelineError::message("invalid album id"))?;

        log::info!("Fetching comments for album {}", album_id);

        let repository = context.service::<Repository<AlbumComment>>()?;

        let mut query = Query::<AlbumComment>::new();
        query.filters.push(Filter {
            field: "album_id".to_string(),
            operator: FilterOperator::Eq,
            value: Value::Uuid(album_id),
        });
        query.sorting.push(Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Desc,
        });

        let comments_page = repository
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        let identity_context = context.get::<IdentityContext>();
        let is_admin = identity_context
            .as_ref()
            .map(|ctx| ctx.identity().claims().roles().contains("admin"))
            .unwrap_or(false);
        let current_user_id = identity_context
            .as_ref()
            .and_then(|ctx| Uuid::parse_str(ctx.identity().subject()).ok());

        let visible_comments = comments_page
            .items
            .into_iter()
            .filter(|comment| {
                if !comment.hidden {
                    return true;
                }
                if is_admin {
                    return true;
                }
                if let Some(user_id) = current_user_id {
                    return comment.user_id == Some(user_id);
                }
                false
            })
            .collect::<Vec<_>>();

        let total = visible_comments.len();
        let response = json!({
            "page": 1,
            "pageSize": total,
            "total": total,
            "items": visible_comments.into_iter().map(AlbumCommentDto::from).collect::<Vec<_>>(),
        });

        log::info!("Returning {} comments for album {}", total, album_id);

        Ok(ResponseValue::new(Json(response)))
    }
}

#[derive(Deserialize)]
struct CreateAlbumCommentPayload {
    comment: String,
}

struct CreateAlbumCommentHandler;

#[async_trait]
impl HttpHandler for CreateAlbumCommentHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<CreateAlbumCommentPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        let trimmed = payload.comment.trim();
        if trimmed.is_empty() {
            return Err(PipelineError::message("Comment cannot be empty"));
        }
        if trimmed.chars().count() > MAX_COMMENT_LENGTH {
            return Err(PipelineError::message(&format!(
                "Comment must be {} characters or fewer",
                MAX_COMMENT_LENGTH
            )));
        }

        let album_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let album_id = Uuid::parse_str(album_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid album id: {}", e)))?;

        let identity = context
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?;
        let user_id = Uuid::parse_str(identity.identity().subject())
            .map_err(|_| PipelineError::message("invalid identity"))?;
        let settings = context.service::<SettingService>()?;
        let can_comment = settings
            .can_create_comments(identity.identity().claims().roles())
            .await?;
        if !can_comment {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

        let settings_repo = context.service::<Repository<UserSettings>>()?;
        let display_name = settings_repo
            .get(&user_id.to_string())
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .map(|settings| settings.display_name)
            .unwrap_or_else(|| "Anonymous".to_string());

        let mut new_comment = AlbumComment::default();
        new_comment.id = Some(Uuid::new_v4());
        new_comment.album_id = Some(album_id);
        new_comment.user_id = Some(user_id);
        new_comment.user_display_name = Some(display_name);
        new_comment.body = Some(trimmed.to_string());
        new_comment.created_at = Some(Utc::now());
        new_comment.hidden = false;

        let repository = context.service::<Repository<AlbumComment>>()?;
        let saved = repository
            .insert(new_comment)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(AlbumCommentDto::from(saved))))
    }
}

#[derive(Deserialize)]
struct UpdateAlbumCommentVisibilityPayload {
    hidden: bool,
}

struct UpdateAlbumCommentVisibilityHandler;

#[async_trait]
impl HttpHandler for UpdateAlbumCommentVisibilityHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let route_params = context.route().map(|route| route.params());

        let album_id_param = route_params
            .as_ref()
            .and_then(|params| params.get("albumId"))
            .ok_or_else(|| PipelineError::message("albumId parameter missing"))?;
        let album_id = Uuid::parse_str(album_id_param)
            .map_err(|_| PipelineError::message("invalid album id"))?;

        let comment_id_param = route_params
            .and_then(|params| params.get("commentId"))
            .ok_or_else(|| PipelineError::message("commentId parameter missing"))?;
        let comment_id = Uuid::parse_str(comment_id_param)
            .map_err(|_| PipelineError::message("invalid comment id"))?;

        let payload = context
            .read_json::<UpdateAlbumCommentVisibilityPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        let repository = context.service::<Repository<AlbumComment>>()?;
        let mut comment = repository
            .get(&comment_id)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .ok_or_else(|| PipelineError::message("Comment not found"))?;

        if comment.album_id != Some(album_id) {
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

impl AlbumController {
    fn is_admin(context: &HttpContext) -> bool {
        context
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().contains("admin"))
            .unwrap_or(false)
    }

    fn to_tag_refs(values: &[serde_json::Value]) -> Result<Vec<TagRef>, PipelineError> {
        let mut refs = Vec::<TagRef>::new();
        for value in values {
            match value {
                serde_json::Value::Number(num) => {
                    let id = num
                        .as_i64()
                        .ok_or_else(|| PipelineError::message("invalid numeric tag id"))?;
                    refs.push(TagRef::Id(id));
                }
                serde_json::Value::String(text) => {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(id) = trimmed.parse::<i64>() {
                        refs.push(TagRef::Id(id));
                    } else {
                        refs.push(TagRef::Name(trimmed.to_string()));
                    }
                }
                _ => return Err(PipelineError::message("tags must be ids or names")),
            }
        }
        Ok(refs)
    }

    fn is_viewer(context: &HttpContext) -> bool {
        context
            .get::<IdentityContext>()
            .map(|ctx| {
                let identity = ctx.identity();
                let roles = identity.claims().roles();
                roles.contains("viewer") && !roles.contains("admin")
            })
            .unwrap_or(false)
    }

    async fn viewer_hidden_tags(context: &HttpContext) -> Result<HashSet<String>, PipelineError> {
        if !Self::is_viewer(context) {
            return Ok(HashSet::new());
        }
        let settings = context.service::<SettingService>()?;
        settings.viewer_hidden_tags().await
    }

    fn filter_photos_for_viewer(
        photos: Vec<crate::entities::photo::Photo>,
        hidden_tags: &HashSet<String>,
    ) -> Vec<crate::entities::photo::Photo> {
        if hidden_tags.is_empty() {
            return photos;
        }

        photos
            .into_iter()
            .filter(|photo| {
                !photo
                    .tags
                    .as_ref()
                    .map(|tags| {
                        tags.iter()
                            .any(|tag| hidden_tags.contains(&tag.trim().to_lowercase()))
                    })
                    .unwrap_or(false)
            })
            .collect()
    }
}
