use async_trait::async_trait;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::time::Instant;
use tokio::task;
use uuid::Uuid;

use crate::controllers::httpcontext_extensions::HttpContextExtensions;
use crate::dtos::photo_comment_dto::PhotoCommentDto;
use crate::dtos::photo_dtos::TagRef;
use crate::dtos::photo_dtos::{
    DeletePhotosPayload, UpdatePhotoTagsPayload, UploadFileResponse, UploadPhotosResponse,
};
use crate::entities::StorageLocation;
use crate::entities::photo::Photo;
use crate::entities::photo_comment::PhotoComment;
use crate::entities::tag::Tag;
use crate::models::setting_consts::SettingConsts;
use crate::models::string_id::ToUuid;
use crate::repositories::photo_repo::PhotoRepositoryExtensions;
use crate::repositories::tag_extensions::TagRepositoryExtensions;
use crate::services::file_service::FileService;
use crate::services::{ImageProcessPipeline, PhotoUploadService, PreviewExtractor, SettingService};

use nimble_web::Repository;
use nimble_web::controller::controller::Controller;
use nimble_web::data::paging::Page;
use nimble_web::data::query::{FilterOperator, Value};
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::FileResponse;
use nimble_web::result::Json;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::{DataProvider, QueryBuilder};
use nimble_web::{delete, get, post, put};

const MAX_COMMENT_LENGTH: usize = 1024;

pub struct PhotoController;

impl Controller for PhotoController {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}

struct UploadPhotosHandler;

#[async_trait]
impl HttpHandler for UploadPhotosHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let settings = context.service::<SettingService>()?;
        if !context.can_upload_photos().await? {
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
        let request_body = context.body_bytes()?;
        let files = upload_service
            .parse_multipart_files(content_type_header, request_body)
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;

        if files.is_empty() {
            return Err(PipelineError::message("No files found in upload request"));
        }

        let storage_query_id = context.id("storageId")?;
        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let storage = storage_repo
            .get(&storage_query_id)
            .await
            .map_err(|_| PipelineError::message("Storage location not found"))?
            .ok_or_else(|| PipelineError::message("Storage is not found"))?;

        let saved_files = upload_service
            .persist_to_storage_temp(Path::new(&storage.path), files)
            .await
            .map_err(|error| PipelineError::message(&error.to_string()))?;

        if !saved_files.is_empty() {
            let pipeline = context.service::<ImageProcessPipeline>()?;
            pipeline
                .enqueue_files(storage.clone(), saved_files.clone())
                .map_err(|error| {
                    log::error!("Failed to enqueue image pipeline: {:?}", error);
                    PipelineError::message("Failed to schedule image processing tasks")
                })?;
        }

        let response = UploadPhotosResponse {
            storage_id: storage.id.to_string(),
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

struct DeletePhotosHandler;

#[async_trait]
#[delete("/api/photos", policy = Policy::Authenticated)]
impl HttpHandler for DeletePhotosHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<DeletePhotosPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;

        if payload.photo_ids.is_empty() {
            return Err(PipelineError::message("photoIds cannot be empty"));
        }

        let photo_repo = context.service::<Repository<Photo>>()?;

        let mut deleted = 0u32;

        for raw_photo_id in payload.photo_ids {
            let photo_id = Uuid::parse_str(raw_photo_id.trim())
                .map_err(|e| PipelineError::message(&format!("invalid photo id: {}", e)))?;

            let Some(photo) = photo_repo
                .get(&photo_id)
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            else {
                continue;
            };

            deleted += photo_repo
                .delete_file(&photo, context)
                .await
                .map_err(|e| {
                    PipelineError::message(&format!("failed to delete photo files: {:?}", e))
                })
                .map(|_| 1u32)
                .unwrap_or(0u32);
        }

        Ok(ResponseValue::new(Json(
            serde_json::json!({ "deleted": deleted }),
        )))
    }
}

struct ThumbnailHandler;

#[async_trait]
#[get("/api/photos/thumbnail/{hash}")]
impl HttpHandler for ThumbnailHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let hash = context.hash()?;
        let photo_repo = context.service::<Repository<Photo>>()?;
        let photo = photo_repo
            .find_by_hash(&hash)
            .await?
            .ok_or_else(|| PipelineError::message("thumbnail not found"))?;

        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let storage = storage_repo
            .get(&photo.storage_id)
            .await
            .map_err(|_| PipelineError::message("Storage location not found"))?
            .ok_or_else(|| PipelineError::message("Storage is not found"))?;

        let file_service = context.service::<FileService>()?;
        let root = Path::new(&storage.path).join(SettingConsts::THUMBNAIL_FOLDER);

        let full_path = file_service.path_for_hash(root, &hash, SettingConsts::THUMBNAIL_FORMAT);

        log::debug!(
            "Thumbnail path resolved to: {}",
            full_path.to_string_lossy()
        );

        Ok(ResponseValue::new(
            FileResponse::from_path(full_path)
                .with_content_type(SettingConsts::THUMBNAIL_CONTENT_TYPE)
                .with_header(
                    "Cache-Control",
                    SettingConsts::DEFAULT_HTTP_IMAGE_CACHE_HEADER,
                ),
        ))
    }
}

struct PreviewHandler;

impl PreviewHandler {
    async fn build_preview(
        &self,
        context: &HttpContext,
        photo: &Photo,
        hash: &str,
    ) -> Result<Option<(PathBuf, &'static str)>, PipelineError> {
        let source_path = PathBuf::from(&photo.path);

        if !source_path.exists() {
            log::warn!(
                "Preview source file missing for hash {} at {}",
                hash,
                source_path.display()
            );
            return Ok(None);
        }

        let output_path = context.get_preview_path(hash).await?;
        let extractor = context.service::<PreviewExtractor>()?;
        let output_path_clone = output_path.clone();
        let source_path_clone = source_path.clone();
        let enqueue_at = Instant::now();

        let generated = task::spawn_blocking(move || {
            let started_at = Instant::now();
            let queue_wait = started_at.duration_since(enqueue_at);
            let extract_started = Instant::now();
            let result = extractor.extract_to(source_path_clone, &output_path_clone);
            let extract_elapsed = extract_started.elapsed();

            (result, queue_wait, extract_elapsed)
        })
        .await
        .ok()
        .and_then(|(result, queue_wait, extract_elapsed)| {
            log::debug!(
                "Preview blocking task timing for hash {}: queue_wait={:?}, extract={:?}",
                hash,
                queue_wait,
                extract_elapsed
            );
            result.ok()
        });

        if let Some(path) = generated {
            if path.exists() {
                return Ok(Some((path, "image/jpeg")));
            }
        }

        Ok(None)
    }
}

#[async_trait]
#[get("/api/photos/preview/{hash}")]
impl HttpHandler for PreviewHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let hash = context.hash()?;
        let photo_repo = context.service::<Repository<Photo>>()?;
        let photo = photo_repo
            .find_by_hash(&hash)
            .await?
            .ok_or_else(|| PipelineError::message("Preview not found"))?;

        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let storage = storage_repo
            .get(&photo.storage_id)
            .await
            .map_err(|_| PipelineError::message("Storage location not found"))?
            .ok_or_else(|| PipelineError::message("Storage is not found"))?;

        let file_service = context.service::<FileService>()?;
        let root = Path::new(&storage.path).join(SettingConsts::PREVIEW_FOLDER);

        let full_path = file_service.path_for_hash(root, &hash, SettingConsts::PREVIEW_FORMAT);

        Ok(ResponseValue::new(
            FileResponse::from_path(full_path)
                .with_content_type(SettingConsts::PREVIEW_CONTENT_TYPE)
                .with_header(
                    "Cache-Control",
                    SettingConsts::DEFAULT_HTTP_IMAGE_CACHE_HEADER,
                ),
        ))
    }
}

struct TimelineHandler;

#[async_trait]
#[get("/api/photos/timeline/{page}/{pageSize}")]
impl HttpHandler for TimelineHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<Photo>>()?;

        let page: u32 = context.page().unwrap_or(1);
        let page_size: u32 = context.page_size().unwrap_or(20);

        let limit = page_size;
        let offset = if page > 0 { (page - 1) * limit } else { 0 };

        let timeline = repository
            .build_timeline(limit, offset)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(timeline))
    }
}

struct TimelineYearsHandler;

#[async_trait]
#[get("/api/photos/timeline/years")]
impl HttpHandler for TimelineYearsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<Photo>>()?;

        let years = repository
            .get_years()
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(years))
    }
}

struct YearOffsetHandler;

#[async_trait]
#[get("/api/photos/timeline/year-offset/{year}")]
impl HttpHandler for YearOffsetHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<Photo>>()?;
        let year = context.param("year")?;

        let offset = repository
            .get_year_offset(&year)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(offset)))
    }
}

struct MapPhotosHandler;

#[async_trait]
#[get("/api/photos/gps/{page}/{pageSize}")]
impl HttpHandler for MapPhotosHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<Photo>>()?;

        let page: u32 = context.page().unwrap_or(1);
        let page_size: u32 = context.page_size().unwrap_or(200);

        let limit = page_size;
        let offset = if page > 0 { (page - 1) * limit } else { 0 };

        let photos = repository
            .photos_with_gps(limit, offset)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        let response = serde_json::json!({
            "page": page,
            "pageSize": page_size,
            "items": photos
        });

        Ok(ResponseValue::json(response))
    }
}

#[derive(Deserialize)]
struct CreatePhotoCommentPayload {
    comment: String,
}

struct PhotoCommentsHandler;

#[async_trait]
#[get("/api/photos/comments/{id}/{page}/{pageSize}")]
impl HttpHandler for PhotoCommentsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let photo_id = context.id("id")?;
        let page: u32 = context.page().unwrap_or(1);
        let page_size: u32 = context.page_size().unwrap_or(50);

        let repository = context.service::<Repository<PhotoComment>>()?;

        let query = QueryBuilder::<PhotoComment>::new()
            .filter("photo_id", FilterOperator::Eq, Value::Uuid(photo_id))
            .sort_desc("created_at")
            .page(page, page_size)
            .build();

        let comments = repository
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        let dtos = Page {
            items: comments
                .items
                .into_iter()
                .map(PhotoCommentDto::from)
                .collect(),
            total: comments.total,
            page: comments.page,
            page_size: comments.page_size,
        };

        Ok(ResponseValue::json(dtos))
    }
}

struct CreatePhotoCommentHandler;

#[async_trait]
#[post("/api/photos/comments/{id}")]
impl HttpHandler for CreatePhotoCommentHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let user_id = context.current_user_id()?;
        let photo_id = context.id("id")?;
        let display_name = context.current_user_display_name().await?;

        let identity = context
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("Identity context not found"))?;

        let settings = context.service::<SettingService>()?;
        let can_comment = settings
            .can_create_comments(identity.identity().claims().roles())
            .await?;
        if !can_comment {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }

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

        let comment = PhotoComment::new(
            photo_id,
            user_id,
            Some(display_name),
            Some(body.to_string()),
        );
        let repository = context.service::<Repository<PhotoComment>>()?;
        let saved = repository
            .insert(comment)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(PhotoCommentDto::from(saved)))
    }
}

struct PhotoTagsHandler;

#[async_trait]
#[get("/api/photos/tags")]
impl HttpHandler for PhotoTagsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<Tag>>()?;

        let query = QueryBuilder::<Tag>::new()
            .distinct()
            .sort_asc("name")
            .build();

        let tags = repository
            .all(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let names = tags.into_iter().map(|t| t.name).collect::<Vec<_>>();
        Ok(ResponseValue::json(names))
    }
}

struct UpdatePhotoTagsHandler;

#[async_trait]
#[put("/api/photos/tags")]
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
        let photo_repo = context.service::<Repository<Photo>>()?;
        let tag_repo = context.service::<Repository<Tag>>()?;

        let mut updated = 0u32;
        for raw_photo_id in payload.photo_ids {
            let photo_id = raw_photo_id.to_uuid().ok_or_else(|| {
                PipelineError::message(&format!("invalid photo id: {}", raw_photo_id))
            })?;

            let exists = photo_repo
                .get(&photo_id)
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
                .is_some();

            if !exists {
                continue;
            }

            tag_repo
                .set_photo_tags(photo_id, &refs)
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
            updated += 1;
        }

        Ok(ResponseValue::new(Json(
            serde_json::json!({ "updated": updated }),
        )))
    }
}
