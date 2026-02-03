use async_trait::async_trait;
use std::path::Path;

use crate::repositories::photo::PhotoRepository;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::FileResponse;
use nimble_web::result::Json;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct PhotoController;

impl Controller for PhotoController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/photos/thumbnail/{hash}", ThumbnailHandler).build(),
            EndpointRoute::get("/api/photos/timeline/{page}/{pageSize}", TimelineHandler).build(),
            EndpointRoute::post("/api/photos/scan", ScanPhotoHandler)
                .with_policy(Policy::Authenticated)
                .build(),
        ]
    }
}

struct ScanPhotoHandler;

#[async_trait]
impl HttpHandler for ScanPhotoHandler {
    async fn invoke(
        &self,
        _context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::empty())
    }
}

struct ThumbnailHandler;

#[async_trait]
impl HttpHandler for ThumbnailHandler {
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
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
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
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
