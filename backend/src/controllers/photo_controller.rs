use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use std::path::Path;
use std::result::Result;
use uuid::Uuid;

use crate::dtos::photo_comment_dto::PhotoCommentDto;
use crate::entities::exif::ExifModel;
use crate::entities::photo_comment::PhotoComment;
use crate::entities::user_settings::UserSettings;
use crate::repositories::photo::PhotoRepository;

use nimble_web::DataProvider;
use nimble_web::Repository;
use nimble_web::controller::controller::Controller;
use nimble_web::data::query::{Filter, FilterOperator, Query, Sort, SortDirection, Value};
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
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
            EndpointRoute::get("/api/photos/{id}/metadata", PhotoMetadataHandler).build(),
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
            .get_timeline(limit, offset)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

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
            .get_years()
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
            .get_year_offset(year)
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
            .get_with_gps(limit, offset)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        let response = serde_json::json!({
            "page": page,
            "pageSize": page_size,
            "items": photos
        });

        Ok(ResponseValue::new(Json(response)))
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
