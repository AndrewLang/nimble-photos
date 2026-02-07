use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::result::Result;
use std::collections::HashSet;
use uuid::Uuid;

use crate::controllers::storage_controller::StorageLocation;
use crate::dtos::photo_comment_dto::PhotoCommentDto;
use crate::dtos::photo_dtos::{PhotoLoc, TimelineGroup};
use crate::entities::exif::ExifModel;
use crate::entities::photo::Photo;
use crate::entities::photo_comment::PhotoComment;
use crate::entities::user_settings::UserSettings;
use crate::repositories::photo::{PhotoRepository, TagRef};
use crate::services::{PhotoUploadService, SettingService};

use nimble_web::DataProvider;
use nimble_web::data::paging::Page;
use nimble_web::Repository;
use nimble_web::controller::controller::Controller;
use nimble_web::data::query::{Filter, FilterOperator, Query, Sort, SortDirection, Value};
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::http::request_body::RequestBody;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::FileResponse;
use nimble_web::result::Json;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

const MAX_COMMENT_LENGTH: usize = 1024;

pub struct PhotoController;

impl Controller for PhotoController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/photos/thumbnail/{hash}", ThumbnailHandler).build(),
            EndpointRoute::get("/api/photos/timeline/{page}/{pageSize}", TimelineHandler).build(),
            EndpointRoute::get("/api/photos/timeline/years", TimelineYearsHandler).build(),
            EndpointRoute::get("/api/photos/timeline/year-offset/{year}", YearOffsetHandler)
                .build(),
            EndpointRoute::get("/api/photos/with-gps/{page}/{pageSize}", MapPhotosHandler).build(),
            EndpointRoute::post("/api/photos", UploadPhotosHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::get("/api/photos", PhotosQueryHandler).build(),
            EndpointRoute::get("/api/photos/{id}/metadata", PhotoMetadataHandler).build(),
            EndpointRoute::get("/api/photos/{id}/tags", PhotoTagListHandler).build(),
            EndpointRoute::put("/api/photos/{id}/tags", ReplacePhotoTagsHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::get("/api/photos/tags", PhotoTagsHandler).build(),
            EndpointRoute::put("/api/photos/tags", UpdatePhotoTagsHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::get("/api/photos/comments/{id}", PhotoCommentsHandler).build(),
            EndpointRoute::post("/api/photos/comments/{id}", CreatePhotoCommentHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::post("/api/photos/scan", ScanPhotoHandler)
                .with_policy(Policy::Authenticated)
                .build(),
        ]
    }
}

struct ScanPhotoHandler;

#[async_trait]
impl HttpHandler for ScanPhotoHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::empty())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadFileResponse {
    file_name: String,
    relative_path: String,
    byte_size: usize,
    content_type: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadPhotosResponse {
    storage_id: String,
    storage_path: String,
    uploaded_count: usize,
    files: Vec<UploadFileResponse>,
}

struct UploadPhotosHandler;

#[async_trait]
impl HttpHandler for UploadPhotosHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let settings = context.service::<SettingService>()?;
        if !PhotoController::can_upload_photos(context, &settings).await? {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

        let uploads_enabled = settings.is_photo_upload_enabled().await?;
        if !uploads_enabled {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

        let upload_service = context.service::<PhotoUploadService>()?;
        let content_type_header = upload_service
            .require_content_type(context.request().headers().get("content-type"))
            .map_err(|error| PipelineError::message(&error.to_string()))?;
        let request_body = PhotoController::read_request_body_bytes(context)?;
        let files = upload_service
            .parse_multipart_files(content_type_header, request_body)
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;

        if files.is_empty() {
            return Err(PipelineError::message("No files found in upload request"));
        }

        let storage_query_id = context.request().query_param("storageId");
        let storage = PhotoController::resolve_upload_storage(&settings, storage_query_id.as_deref()).await?;
        let saved_files = upload_service
            .persist_to_storage_temp(Path::new(&storage.path), files)
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;

        let response = UploadPhotosResponse {
            storage_id: storage.id,
            storage_path: storage.path,
            uploaded_count: saved_files.len(),
            files: saved_files
                .into_iter()
                .map(|item| UploadFileResponse {
                    file_name: item.file_name,
                    relative_path: item.relative_path,
                    byte_size: item.byte_size,
                    content_type: item.content_type,
                })
                .collect(),
        };

        Ok(ResponseValue::json(response))
    }
}

struct ThumbnailHandler;

#[async_trait]
impl HttpHandler for ThumbnailHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let hash = context
            .route()
            .and_then(|route| route.params().get("hash"))
            .ok_or_else(|| PipelineError::message("hash parameter missing"))?;

        log::debug!("Serving thumbnail for hash: {}", hash);
        if hash.len() < 4 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PipelineError::message("invalid thumbnail hash"));
        }

        let config = context.config();
        let base = config
            .get("thumbnail.base.path")
            .or_else(|| config.get("thumbnail.basepath"))
            .unwrap_or("./thumbnails");

        let path = Path::new(base)
            .join(&hash[0..2])
            .join(&hash[2..4])
            .join(format!("{hash}.webp"));

        log::debug!("Thumbnail path resolved to: {}", path.to_string_lossy());

        Ok(ResponseValue::new(
            FileResponse::from_path(path)
                .with_content_type("image/webp")
                .with_header("Cache-Control", "public, max-age=31536000, immutable"),
        ))
    }
}

struct TimelineHandler;

#[async_trait]
impl HttpHandler for TimelineHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context
            .services()
            .resolve::<Box<dyn PhotoRepository>>()
            .ok_or_else(|| PipelineError::message("PhotoRepository not found"))?;
        let query = context.request().query_params();
        let tags_raw = query.get("tags").cloned().unwrap_or_default();
        let search_tags = tags_raw
            .split(',')
            .map(|v| v.trim().to_lowercase())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        let match_all = query
            .get("match")
            .map(|v| v.eq_ignore_ascii_case("all"))
            .unwrap_or(false);

        let route_params = context.route().map(|r| r.params());

        let page: u32 = route_params
            .and_then(|p| p.get("page"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let page_size: u32 = route_params
            .and_then(|p| p.get("pageSize"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let limit = page_size;
        let offset = if page > 0 { (page - 1) * limit } else { 0 };

        let timeline = repository
            .get_timeline(limit, offset, PhotoController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        let hidden_tags = PhotoController::viewer_hidden_tags(context).await?;
        let timeline = timeline
            .into_iter()
            .map(|group| {
                let page = group.photos;
                let filtered = PhotoController::filter_photos_for_viewer(page.items, &hidden_tags)
                    .into_iter()
                    .filter(|photo| PhotoController::photo_matches_search_tags(photo, &search_tags, match_all))
                    .collect::<Vec<_>>();
                let filtered_total = filtered.len() as u64;
                TimelineGroup {
                    title: group.title,
                    photos: Page::new(filtered, filtered_total, page.page, page.page_size),
                }
            })
            .filter(|group| !group.photos.items.is_empty())
            .collect::<Vec<_>>();

        Ok(ResponseValue::new(Json(timeline)))
    }
}

struct TimelineYearsHandler;

#[async_trait]
impl HttpHandler for TimelineYearsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context
            .services()
            .resolve::<Box<dyn PhotoRepository>>()
            .ok_or_else(|| PipelineError::message("PhotoRepository not found"))?;

        let years = repository
            .get_years(PhotoController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(years)))
    }
}

struct YearOffsetHandler;

#[async_trait]
impl HttpHandler for YearOffsetHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context
            .services()
            .resolve::<Box<dyn PhotoRepository>>()
            .ok_or_else(|| PipelineError::message("PhotoRepository not found"))?;

        let year = context
            .route()
            .and_then(|route| route.params().get("year"))
            .ok_or_else(|| PipelineError::message("year parameter missing"))?;

        let offset = repository
            .get_year_offset(year, PhotoController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(offset)))
    }
}

struct MapPhotosHandler;

#[async_trait]
impl HttpHandler for MapPhotosHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context
            .services()
            .resolve::<Box<dyn PhotoRepository>>()
            .ok_or_else(|| PipelineError::message("PhotoRepository not found"))?;

        let route_params = context.route().map(|r| r.params());

        let page: u32 = route_params
            .and_then(|p| p.get("page"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let page_size: u32 = route_params
            .and_then(|p| p.get("pageSize"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let limit = page_size;
        let offset = if page > 0 { (page - 1) * limit } else { 0 };

        let photos = repository
            .get_with_gps(limit, offset, PhotoController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let hidden_tags = PhotoController::viewer_hidden_tags(context).await?;
        let photos = PhotoController::filter_photo_locs_for_viewer(photos, &hidden_tags);

        let response = serde_json::json!({
            "page": page,
            "pageSize": page_size,
            "items": photos
        });

        Ok(ResponseValue::new(Json(response)))
    }
}

struct PhotosQueryHandler;

#[async_trait]
impl HttpHandler for PhotosQueryHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context
            .services()
            .resolve::<Box<dyn PhotoRepository>>()
            .ok_or_else(|| PipelineError::message("PhotoRepository not found"))?;
        let hidden_tags = PhotoController::viewer_hidden_tags(context).await?;

        let query = context.request().query_params();
        let tags_raw = query.get("tags").cloned().unwrap_or_default();
        let tags = tags_raw
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        let match_all = query
            .get("match")
            .map(|v| v.eq_ignore_ascii_case("all"))
            .unwrap_or(false);
        let page = query
            .get("page")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        let page_size = query
            .get("pageSize")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(56);

        if tags.is_empty() {
            let page_result = repository
                .get_photos_page(page, page_size, PhotoController::is_admin(context))
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
            let page_result = PhotoController::filter_photo_page_for_viewer(page_result, &hidden_tags);
            return Ok(ResponseValue::new(Json(page_result)));
        }

        let result = repository
            .filter_photos_by_tags(
                &tags,
                match_all,
                PhotoController::is_admin(context),
                page,
                page_size,
            )
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let result = PhotoController::filter_photo_page_for_viewer(result, &hidden_tags);
        Ok(ResponseValue::new(Json(result)))
    }
}

#[derive(Deserialize)]
struct CreatePhotoCommentPayload {
    comment: String,
}

struct PhotoCommentsHandler;

#[async_trait]
impl HttpHandler for PhotoCommentsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let photo_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let photo_id = Uuid::parse_str(photo_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid photo id: {}", e)))?;

        let repository = context
            .service::<Repository<PhotoComment>>()
            .map_err(|_| PipelineError::message("PhotoComment repository not found"))?;

        let mut query = Query::<PhotoComment>::new();
        query.filters.push(Filter {
            field: "photo_id".to_string(),
            operator: FilterOperator::Eq,
            value: Value::Uuid(photo_id),
        });
        query.sorting.push(Sort {
            field: "created_at".to_string(),
            direction: SortDirection::Desc,
        });

        let comments_page = repository
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let comments = comments_page
            .items
            .into_iter()
            .map(PhotoCommentDto::from)
            .collect::<Vec<_>>();

        Ok(ResponseValue::new(Json(comments)))
    }
}

struct CreatePhotoCommentHandler;

#[async_trait]
impl HttpHandler for CreatePhotoCommentHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<CreatePhotoCommentPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        let body = payload.comment.trim();
        if body.is_empty() {
            return Err(PipelineError::message("Comment cannot be empty"));
        }
        if body.chars().count() > MAX_COMMENT_LENGTH {
            return Err(PipelineError::message(&format!(
                "Comment must be {} characters or fewer",
                MAX_COMMENT_LENGTH
            )));
        }

        let photo_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let photo_id = Uuid::parse_str(photo_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid photo id: {}", e)))?;

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

        let mut new_comment = PhotoComment::default();
        new_comment.id = Some(Uuid::new_v4());
        new_comment.photo_id = Some(photo_id);
        new_comment.user_id = Some(user_id);
        new_comment.user_display_name = Some(display_name);
        new_comment.body = Some(body.to_string());
        new_comment.created_at = Some(Utc::now());

        let repository = context.service::<Repository<PhotoComment>>()?;
        let saved = repository
            .insert(new_comment)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(PhotoCommentDto::from(saved))))
    }
}

struct PhotoMetadataHandler;

#[async_trait]
impl HttpHandler for PhotoMetadataHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context
            .services()
            .resolve::<Repository<ExifModel>>()
            .ok_or_else(|| PipelineError::message("Exif repository not found"))?;

        let photo_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let photo_id = Uuid::parse_str(photo_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid photo id: {}", e)))?;

        let metadata = repository
            .get_by("image_id", Value::Uuid(photo_id))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(metadata)))
    }
}

#[derive(Deserialize)]
struct ReplaceTagsPayload {
    tags: Vec<serde_json::Value>,
}

struct PhotoTagListHandler;

#[async_trait]
impl HttpHandler for PhotoTagListHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let photo_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let photo_id = Uuid::parse_str(photo_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid photo id: {}", e)))?;

        let repository = context.service::<Box<dyn PhotoRepository>>()?;
        let tags = repository
            .get_photo_tags(photo_id, PhotoController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(tags)))
    }
}

struct ReplacePhotoTagsHandler;

#[async_trait]
impl HttpHandler for ReplacePhotoTagsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let photo_id_param = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        let photo_id = Uuid::parse_str(photo_id_param)
            .map_err(|e| PipelineError::message(&format!("invalid photo id: {}", e)))?;

        let payload = context
            .read_json::<ReplaceTagsPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;
        let refs = PhotoController::to_tag_refs(&payload.tags)?;
        let current_user_id = context
            .get::<IdentityContext>()
            .and_then(|ctx| Uuid::parse_str(ctx.identity().subject()).ok())
            .ok_or_else(|| PipelineError::message("invalid identity"))?;

        let repository = context.service::<Box<dyn PhotoRepository>>()?;
        repository
            .set_photo_tags(photo_id, &refs, Some(current_user_id))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        let tags = repository
            .get_photo_tags(photo_id, PhotoController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(tags)))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePhotoTagsPayload {
    photo_ids: Vec<String>,
    tags: Vec<String>,
}

struct PhotoTagsHandler;

#[async_trait]
impl HttpHandler for PhotoTagsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Box<dyn PhotoRepository>>()?;
        let tags = repository
            .list_all_tags(PhotoController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let names = tags.into_iter().map(|t| t.name).collect::<Vec<_>>();
        Ok(ResponseValue::new(Json(names)))
    }
}

struct UpdatePhotoTagsHandler;

#[async_trait]
impl HttpHandler for UpdatePhotoTagsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<UpdatePhotoTagsPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        if payload.photo_ids.is_empty() {
            return Err(PipelineError::message("photoIds cannot be empty"));
        }

        let refs = payload
            .tags
            .iter()
            .map(|name| TagRef::Name(name.clone()))
            .collect::<Vec<_>>();
        let current_user_id = context
            .get::<IdentityContext>()
            .and_then(|ctx| Uuid::parse_str(ctx.identity().subject()).ok())
            .ok_or_else(|| PipelineError::message("invalid identity"))?;
        let repository = context.service::<Box<dyn PhotoRepository>>()?;

        let mut updated = 0u32;
        for raw_photo_id in payload.photo_ids {
            let photo_id = Uuid::parse_str(raw_photo_id.trim())
                .map_err(|e| PipelineError::message(&format!("invalid photo id: {}", e)))?;

            let exists = context
                .service::<Repository<Photo>>()?
                .get(&photo_id)
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
                .is_some();

            if !exists {
                continue;
            }

            repository
                .set_photo_tags(photo_id, &refs, Some(current_user_id))
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
            updated += 1;
        }

        Ok(ResponseValue::new(Json(
            serde_json::json!({ "updated": updated }),
        )))
    }
}

impl PhotoController {
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

    async fn can_upload_photos(
        context: &HttpContext,
        service: &SettingService,
    ) -> Result<bool, PipelineError> {
        let roles = context
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().clone())
            .unwrap_or_default();
        service.can_upload_photos(&roles).await
    }

    fn read_request_body_bytes(context: &HttpContext) -> Result<Vec<u8>, PipelineError> {
        match context.request().body() {
            RequestBody::Empty => Ok(Vec::new()),
            RequestBody::Text(text) => Ok(text.as_bytes().to_vec()),
            RequestBody::Bytes(bytes) => Ok(bytes.clone()),
            RequestBody::Stream(stream) => {
                let mut guard = stream
                    .lock()
                    .map_err(|_| PipelineError::message("request body stream lock error"))?;
                let mut collected = Vec::<u8>::new();
                loop {
                    let next_chunk = guard
                        .read_chunk()
                        .map_err(|error| PipelineError::message(&error.to_string()))?;
                    match next_chunk {
                        Some(chunk) => collected.extend_from_slice(&chunk),
                        None => break,
                    }
                }
                Ok(collected)
            }
        }
    }

    async fn resolve_upload_storage(
        service: &SettingService,
        storage_id: Option<&str>,
    ) -> Result<StorageLocation, PipelineError> {
        let storage_setting = service.get("storage.locations").await?;
        let locations: Vec<StorageLocation> = serde_json::from_value(storage_setting.value)
            .map_err(|_| PipelineError::message("Invalid storage settings"))?;

        if locations.is_empty() {
            return Err(PipelineError::message("No storage location configured"));
        }

        if let Some(requested_storage_id) = storage_id {
            let selected = locations
                .into_iter()
                .find(|location| location.id == requested_storage_id)
                .ok_or_else(|| PipelineError::message("Requested storage location not found"))?;
            return Ok(selected);
        }

        locations
            .iter()
            .find(|location| location.is_default)
            .cloned()
            .or_else(|| locations.into_iter().next())
            .ok_or_else(|| PipelineError::message("No storage location configured"))
    }

    fn filter_photo_page_for_viewer(
        page: Page<Photo>,
        hidden_tags: &HashSet<String>,
    ) -> Page<Photo> {
        if hidden_tags.is_empty() {
            return page;
        }

        let items = Self::filter_photos_for_viewer(page.items, hidden_tags);
        Page::new(items, page.total, page.page, page.page_size)
    }

    fn filter_photos_for_viewer(photos: Vec<Photo>, hidden_tags: &HashSet<String>) -> Vec<Photo> {
        if hidden_tags.is_empty() {
            return photos;
        }

        photos
            .into_iter()
            .filter(|photo| !Self::photo_has_hidden_tag(photo.tags.as_ref(), hidden_tags))
            .collect()
    }

    fn filter_photo_locs_for_viewer(
        photos: Vec<PhotoLoc>,
        hidden_tags: &HashSet<String>,
    ) -> Vec<PhotoLoc> {
        if hidden_tags.is_empty() {
            return photos;
        }

        photos
            .into_iter()
            .filter(|photo| !Self::photo_has_hidden_tag(photo.photo.tags.as_ref(), hidden_tags))
            .collect()
    }

    fn photo_has_hidden_tag(
        tags: Option<&Vec<String>>,
        hidden_tags: &HashSet<String>,
    ) -> bool {
        tags.map(|items| {
            items
                .iter()
                .any(|tag| hidden_tags.contains(&tag.trim().to_lowercase()))
        })
        .unwrap_or(false)
    }

    fn photo_matches_search_tags(photo: &Photo, search_tags: &[String], match_all: bool) -> bool {
        if search_tags.is_empty() {
            return true;
        }

        let photo_tags = photo
            .tags
            .as_ref()
            .map(|tags| tags.iter().map(|tag| tag.trim().to_lowercase()).collect::<HashSet<_>>())
            .unwrap_or_default();

        if match_all {
            search_tags.iter().all(|tag| photo_tags.contains(tag))
        } else {
            search_tags.iter().any(|tag| photo_tags.contains(tag))
        }
    }
}
