use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::result::Result;
use std::time::Instant;
use tokio::task;
use uuid::Uuid;

use crate::controllers::httpcontext_extensions::HttpContextExtensions;
use crate::dtos::photo_comment_dto::PhotoCommentDto;
use crate::dtos::photo_dtos::{PhotoLoc, PhotoLocWithTags, PhotoWithTags};
use crate::entities::StorageLocation;
use crate::entities::exif::ExifModel;
use crate::entities::photo::Photo;
use crate::entities::photo_comment::PhotoComment;
use crate::entities::user_settings::UserSettings;
use crate::repositories::photo::{PhotoRepository, TagRef};
use crate::repositories::photo_repo::PhotoRepositoryExtensions;
use crate::services::{ImageProcessPipeline, PhotoUploadService, PreviewExtractor, SettingService};

use nimble_web::DataProvider;
use nimble_web::Repository;
use nimble_web::controller::controller::Controller;
use nimble_web::data::paging::Page;
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
            EndpointRoute::get(
                "/api/photos/thumbnail/{storage_id}/{hash}",
                ThumbnailByStorageHandler,
            )
            .build(),
            EndpointRoute::get("/api/photos/thumbnail/{hash}", ThumbnailHandler).build(),
            EndpointRoute::get(
                "/api/photos/preview/{storage_id}/{hash}",
                PreviewByStorageHandler,
            )
            .build(),
            EndpointRoute::get("/api/photos/preview/{hash}", PreviewHandler).build(),
            EndpointRoute::get("/api/photos/haspreview/{hash}", HasPreviewHandler).build(),
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
        let storage =
            PhotoController::resolve_upload_storage(context, storage_query_id.as_deref()).await?;
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

struct ThumbnailHandler;
struct ThumbnailByStorageHandler;

#[async_trait]
impl HttpHandler for ThumbnailByStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let storage_id = context.route_uuid("storage_id")?;
        let hash = context.hash()?;

        log::debug!("Serving thumbnail for storage {} hash {}", storage_id, hash);

        let thumbnail_root = context.get_thumbnail_root_by_storage(storage_id).await?;

        let webp_path = thumbnail_root
            .join(&hash[0..2])
            .join(&hash[2..4])
            .join(format!("{hash}.webp"));

        let (resolved_path, content_type) = if webp_path.exists() {
            (webp_path, "image/webp")
        } else {
            return Err(PipelineError::message("thumbnail not found"));
        };

        Ok(ResponseValue::new(
            FileResponse::from_path(resolved_path)
                .with_content_type(content_type)
                .with_header("Cache-Control", "public, max-age=31536000, immutable"),
        ))
    }
}

#[async_trait]
impl HttpHandler for ThumbnailHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let hash = context.hash()?;

        log::debug!("Serving thumbnail for hash: {}", hash);

        let thumbnail_roots = context.get_thumbnail_roots().await?;

        let mut resolved: Option<(PathBuf, &'static str)> = None;
        for root in &thumbnail_roots {
            let jpeg_path = root
                .join(&hash[0..2])
                .join(&hash[2..4])
                .join(format!("{hash}.jpg"));
            if jpeg_path.exists() {
                resolved = Some((jpeg_path, "image/jpeg"));
                break;
            }

            let webp_path = root
                .join(&hash[0..2])
                .join(&hash[2..4])
                .join(format!("{hash}.webp"));

            if webp_path.exists() {
                resolved = Some((webp_path, "image/webp"));
                break;
            }
        }

        let (resolved_path, content_type) =
            resolved.ok_or_else(|| PipelineError::message("thumbnail not found"))?;

        log::debug!(
            "Thumbnail path resolved to: {}",
            resolved_path.to_string_lossy()
        );

        Ok(ResponseValue::new(
            FileResponse::from_path(resolved_path)
                .with_content_type(content_type)
                .with_header("Cache-Control", "public, max-age=31536000, immutable"),
        ))
    }
}

struct PreviewHandler;
struct PreviewByStorageHandler;

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
impl HttpHandler for PreviewByStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let storage_id = context.route_uuid("storage_id")?;
        let hash = context.hash()?;

        log::debug!("Serving preview for storage {} hash {}", storage_id, hash);

        let preview_path = context
            .get_preview_path_by_storage(storage_id, &hash)
            .await?;
        if preview_path.exists() {
            return Ok(ResponseValue::new(
                FileResponse::from_path(preview_path)
                    .with_content_type("image/jpeg")
                    .with_header("Cache-Control", "public, max-age=31536000, immutable"),
            ));
        }

        let mut query = Query::<Photo>::new();
        query.filters.push(Filter {
            field: "storage_id".to_string(),
            operator: FilterOperator::Eq,
            value: Value::Uuid(storage_id),
        });
        query.filters.push(Filter {
            field: "hash".to_string(),
            operator: FilterOperator::Eq,
            value: Value::String(hash.clone()),
        });

        let photo = context
            .service::<Repository<Photo>>()?
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .items
            .into_iter()
            .next()
            .ok_or_else(|| PipelineError::message("preview not found"))?;

        let source_path = PathBuf::from(&photo.path);
        if !source_path.exists() {
            return Err(PipelineError::message("preview source not found"));
        }

        let output_path = context
            .get_preview_path_by_storage(storage_id, &hash)
            .await?;
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
                "Preview (storage-specific) blocking task timing for hash {}: queue_wait={:?}, extract={:?}",
                hash,
                queue_wait,
                extract_elapsed
            );
            result.ok()
        });

        let resolved_path = generated
            .filter(|path| path.exists())
            .ok_or_else(|| PipelineError::message("preview not found"))?;

        Ok(ResponseValue::new(
            FileResponse::from_path(resolved_path)
                .with_content_type("image/jpeg")
                .with_header("Cache-Control", "public, max-age=31536000, immutable"),
        ))
    }
}

#[async_trait]
impl HttpHandler for PreviewHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let hash = context.hash()?;
        log::debug!("Serving preview for hash: {}", hash);

        let photo_repo = context.service::<Repository<Photo>>()?;
        let photo = photo_repo
            .find_by_hash(&hash)
            .await?
            .ok_or_else(|| PipelineError::message("preview not found"))?;

        let mut resolved: Option<(PathBuf, &'static str)> = None;
        let jpeg_path = context.get_preview_path(&hash).await?;

        if jpeg_path.exists() {
            resolved = Some((jpeg_path, "image/jpeg"));
        }

        if resolved.is_none() {
            resolved = self.build_preview(context, &photo, &hash).await?;
        }

        let (resolved_path, content_type) =
            resolved.ok_or_else(|| PipelineError::message("preview not found"))?;

        Ok(ResponseValue::new(
            FileResponse::from_path(resolved_path)
                .with_content_type(content_type)
                .with_header("Cache-Control", "public, max-age=31536000, immutable"),
        ))
    }
}

struct HasPreviewHandler;

#[async_trait]
impl HttpHandler for HasPreviewHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let hash = context.hash()?;
        log::debug!("Checking preview existence for hash: {}", hash);

        let exists = context.is_preview_exists(&hash).await;

        Ok(ResponseValue::json(
            serde_json::json!({ "hasPreview": exists }),
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
        let _search_tags = tags_raw
            .split(',')
            .map(|v| v.trim().to_lowercase())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        let _match_all = query
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

        let _hidden_tags = PhotoController::viewer_hidden_tags(context).await?;
        // let timeline = timeline
        //     .into_iter()
        //     .map(|group| {
        //         let page = group.photos;
        //         let filtered = PhotoController::filter_photos_for_viewer(page.items, &hidden_tags)
        //             .into_iter()
        //             .filter(|photo| {
        //                 PhotoController::photo_matches_search_tags(photo, &search_tags, match_all)
        //             })
        //             .collect::<Vec<_>>();
        //         let filtered_total = filtered.len() as u64;
        //         TimelineGroup {
        //             title: group.title,
        //             photos: Page::new(filtered, filtered_total, page.page, page.page_size),
        //         }
        //     })
        //     .filter(|group| !group.photos.items.is_empty())
        //     .collect::<Vec<_>>();

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
        let is_admin = PhotoController::is_admin(context);

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
            .get_with_gps(limit, offset, is_admin)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let photo_ids = PhotoController::collect_photo_loc_ids(&photos);
        let tag_map = repository
            .get_photo_tag_name_map(&photo_ids, is_admin)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let hidden_tags = PhotoController::viewer_hidden_tags(context).await?;
        let photos = PhotoController::filter_photo_locs_for_viewer(photos, &hidden_tags, &tag_map);
        let photos = PhotoController::attach_tags_to_locs(photos, &tag_map);

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
        let is_admin = PhotoController::is_admin(context);

        if tags.is_empty() {
            let mut page_result = repository
                .get_photos_page(page, page_size, is_admin)
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
            let photo_ids = PhotoController::collect_photo_ids(&page_result.items);
            let tag_map = repository
                .get_photo_tag_name_map(&photo_ids, is_admin)
                .await
                .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
            page_result =
                PhotoController::filter_photo_page_for_viewer(page_result, &hidden_tags, &tag_map);
            let dto_page = PhotoController::attach_tags_to_page(page_result, &tag_map);
            return Ok(ResponseValue::new(Json(dto_page)));
        }

        let mut result = repository
            .filter_photos_by_tags(&tags, match_all, is_admin, page, page_size)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        let photo_ids = PhotoController::collect_photo_ids(&result.items);
        let tag_map = repository
            .get_photo_tag_name_map(&photo_ids, is_admin)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
        result = PhotoController::filter_photo_page_for_viewer(result, &hidden_tags, &tag_map);
        let dto_page = PhotoController::attach_tags_to_page(result, &tag_map);
        Ok(ResponseValue::new(Json(dto_page)))
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
            .ok_or_else(|| PipelineError::message("Identity not found"))?;
        let user_id = Uuid::parse_str(identity.identity().subject()).map_err(|_| {
            PipelineError::message("Invalid identity: user ID is not valid, photo comment")
        })?;
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
            .get(&user_id)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .map(|settings| settings.display_name)
            .unwrap_or_else(|| "Anonymous".to_string());

        let mut new_comment = PhotoComment::default();
        new_comment.id = Uuid::new_v4();
        new_comment.photo_id = photo_id;
        new_comment.user_id = user_id;
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
            .ok_or_else(|| {
                PipelineError::message(
                    "Invalid identity: user ID is not valid, replacing photo tags",
                )
            })?;

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
            .ok_or_else(|| {
                PipelineError::message(
                    "Invalid identity: user ID is not valid, updating photo tags",
                )
            })?;
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
                serde_json::Value::Number(_) => {
                    return Err(PipelineError::message("tags must be UUID strings or names"));
                }
                serde_json::Value::String(text) => {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(id) = Uuid::parse_str(trimmed) {
                        refs.push(TagRef::Id(id));
                    } else {
                        refs.push(TagRef::Name(trimmed.to_string()));
                    }
                }
                _ => return Err(PipelineError::message("tags must be UUID strings or names")),
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
        context: &HttpContext,
        storage_id: Option<&str>,
    ) -> Result<StorageLocation, PipelineError> {
        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let locations = storage_repo
            .query(Query::<StorageLocation>::new())
            .await
            .map_err(|_| PipelineError::message("Failed to load storage locations"))?
            .items;

        if locations.is_empty() {
            return Err(PipelineError::message("No storage location configured"));
        }

        if let Some(requested_storage_id) = storage_id {
            let requested_storage_id = Uuid::parse_str(requested_storage_id)
                .map_err(|_| PipelineError::message("Invalid storage location id"))?;
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
        photo_tags: &HashMap<Uuid, Vec<String>>,
    ) -> Page<Photo> {
        if hidden_tags.is_empty() {
            return page;
        }

        let items = Self::filter_photos_for_viewer(page.items, hidden_tags, photo_tags);
        Page::new(items, page.total, page.page, page.page_size)
    }

    fn filter_photos_for_viewer(
        photos: Vec<Photo>,
        hidden_tags: &HashSet<String>,
        photo_tags: &HashMap<Uuid, Vec<String>>,
    ) -> Vec<Photo> {
        if hidden_tags.is_empty() {
            return photos;
        }

        photos
            .into_iter()
            .filter(|photo| {
                let tags = photo_tags.get(&photo.id);
                !Self::photo_has_hidden_tag(tags, hidden_tags)
            })
            .collect()
    }

    fn filter_photo_locs_for_viewer(
        photos: Vec<PhotoLoc>,
        hidden_tags: &HashSet<String>,
        photo_tags: &HashMap<Uuid, Vec<String>>,
    ) -> Vec<PhotoLoc> {
        if hidden_tags.is_empty() {
            return photos;
        }

        photos
            .into_iter()
            .filter(|photo| {
                let tags = photo_tags.get(&photo.photo.id);
                !Self::photo_has_hidden_tag(tags, hidden_tags)
            })
            .collect()
    }

    fn photo_has_hidden_tag(tags: Option<&Vec<String>>, hidden_tags: &HashSet<String>) -> bool {
        tags.map(|items| {
            items
                .iter()
                .any(|tag| hidden_tags.contains(&tag.trim().to_lowercase()))
        })
        .unwrap_or(false)
    }

    fn collect_photo_ids(photos: &[Photo]) -> Vec<Uuid> {
        photos.iter().map(|photo| photo.id).collect()
    }

    fn collect_photo_loc_ids(photos: &[PhotoLoc]) -> Vec<Uuid> {
        photos.iter().map(|photo| photo.photo.id).collect()
    }

    fn attach_tags_to_page(
        page: Page<Photo>,
        photo_tags: &HashMap<Uuid, Vec<String>>,
    ) -> Page<PhotoWithTags> {
        let items = page
            .items
            .into_iter()
            .map(|photo| {
                let tags = photo_tags.get(&photo.id).cloned().unwrap_or_default();
                PhotoWithTags { photo, tags }
            })
            .collect();
        Page::new(items, page.total, page.page, page.page_size)
    }

    fn attach_tags_to_locs(
        photos: Vec<PhotoLoc>,
        photo_tags: &HashMap<Uuid, Vec<String>>,
    ) -> Vec<PhotoLocWithTags> {
        photos
            .into_iter()
            .map(|photo_loc| {
                let tags = photo_tags
                    .get(&photo_loc.photo.id)
                    .cloned()
                    .unwrap_or_default();
                PhotoLocWithTags {
                    loc: photo_loc,
                    tags,
                }
            })
            .collect()
    }
}
