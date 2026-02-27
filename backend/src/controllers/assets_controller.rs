use async_trait::async_trait;
use std::path::Path;

use nimble_web::Controller;
use nimble_web::EndpointRoute;
use nimble_web::FileResponse;
use nimble_web::HttpContext;
use nimble_web::HttpHandler;
use nimble_web::PipelineError;
use nimble_web::ResponseValue;
use nimble_web::get;

use crate::controllers::httpcontext_extensions::HttpContextExtensions;

pub struct AssetsController;

impl Controller for AssetsController {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}

struct LogoHandler;

#[async_trait]
#[get("/api/assets/logo/{filename}")]
impl HttpHandler for LogoHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let filename = context.param("filename")?;

        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(PipelineError::message("invalid filename"));
        }

        let path = Path::new("data").join("logo").join(filename);

        Ok(ResponseValue::new(FileResponse::from_path(path)))
    }
}
