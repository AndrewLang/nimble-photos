use async_trait::async_trait;
use std::path::Path;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::FileResponse;
use nimble_web::result::into_response::ResponseValue;

pub struct AssetsController;

impl Controller for AssetsController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/assets/logo/{filename}", LogoHandler).build(),
            EndpointRoute::get("/api/assets/{folder}/{filename}", FolderAssetHandler).build(),
            EndpointRoute::get("/api/assets/{filename}", RootAssetHandler).build(),
        ]
    }
}

struct LogoHandler;

#[async_trait]
impl HttpHandler for LogoHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let filename = context
            .route()
            .and_then(|route| route.params().get("filename"))
            .ok_or_else(|| PipelineError::message("filename parameter missing"))?;

        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(PipelineError::message("invalid filename"));
        }

        let path = Path::new("data").join("logo").join(filename);

        Ok(ResponseValue::new(FileResponse::from_path(path)))
    }
}

struct FolderAssetHandler;

#[async_trait]
impl HttpHandler for FolderAssetHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let folder = context
            .route()
            .and_then(|route| route.params().get("folder"))
            .ok_or_else(|| PipelineError::message("folder parameter missing"))?;
        let filename = context
            .route()
            .and_then(|route| route.params().get("filename"))
            .ok_or_else(|| PipelineError::message("filename parameter missing"))?;

        validate_path_segment(folder)?;
        validate_path_segment(filename)?;

        let path = Path::new("data").join(folder).join(filename);
        Ok(ResponseValue::new(FileResponse::from_path(path)))
    }
}

struct RootAssetHandler;

#[async_trait]
impl HttpHandler for RootAssetHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let filename = context
            .route()
            .and_then(|route| route.params().get("filename"))
            .ok_or_else(|| PipelineError::message("filename parameter missing"))?;

        validate_path_segment(filename)?;

        let path = Path::new("data").join(filename);
        Ok(ResponseValue::new(FileResponse::from_path(path)))
    }
}

fn validate_path_segment(segment: &str) -> Result<(), PipelineError> {
    if segment.contains("..") || segment.contains('/') || segment.contains('\\') {
        return Err(PipelineError::message("invalid path"));
    }
    Ok(())
}
