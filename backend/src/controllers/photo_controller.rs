use async_trait::async_trait;
use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct PhotoController;

impl Controller for PhotoController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
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
