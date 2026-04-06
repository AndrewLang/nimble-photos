use async_trait::async_trait;
use std::path::Path;

use crate::prelude::*;

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
