use async_trait::async_trait;
use std::path::Path;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::get;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::FileResponse;
use nimble_web::result::into_response::ResponseValue;

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
