use async_trait::async_trait;
use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use std::path::Path;

use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::FileResponse;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct PhotoController;

impl Controller for PhotoController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/photos/thumbnail/{hash}", ThumbnailHandler).build(),
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
            .get("thumbnail_base_path")
            .or_else(|| config.get("thumbnail.basepath"))
            .unwrap_or("./thumbnails");

        let path = Path::new(base)
            .join(&hash[0..2])
            .join(&hash[2..4])
            .join(format!("{hash}.webp"));

        Ok(ResponseValue::new(
            FileResponse::from_path(path).with_content_type("image/webp"),
        ))
    }
}
