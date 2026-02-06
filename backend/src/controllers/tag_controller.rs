use async_trait::async_trait;
use serde::Deserialize;

use crate::repositories::photo::PhotoRepository;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::Json;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct TagController;

impl Controller for TagController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/tags", ListTagsHandler).build(),
            EndpointRoute::post("/api/tags", UpsertTagHandler)
                .with_policy(Policy::InRole("admin".to_string()))
                .build(),
        ]
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertTagPayload {
    name: String,
    visibility: Option<serde_json::Value>,
}

struct ListTagsHandler;

#[async_trait]
impl HttpHandler for ListTagsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Box<dyn PhotoRepository>>()?;
        let tags = repository
            .list_all_tags(TagController::is_admin(context))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(tags)))
    }
}

struct UpsertTagHandler;

#[async_trait]
impl HttpHandler for UpsertTagHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<UpsertTagPayload>()
            .map_err(|e| PipelineError::message(e.message()))?;
        let visibility = TagController::parse_visibility(payload.visibility)?;

        let repository = context.service::<Box<dyn PhotoRepository>>()?;
        let tag = repository
            .upsert_tag(&payload.name, Some(visibility))
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::new(Json(tag)))
    }
}

impl TagController {
    fn parse_visibility(value: Option<serde_json::Value>) -> Result<i16, PipelineError> {
        let Some(value) = value else {
            return Ok(0);
        };

        match value {
            serde_json::Value::Number(n) => n
                .as_i64()
                .ok_or_else(|| PipelineError::message("invalid visibility value"))
                .and_then(|v| match v {
                    0 | 1 => Ok(v as i16),
                    _ => Err(PipelineError::message("visibility must be 0 or 1")),
                }),
            serde_json::Value::String(s) => {
                let v = s.trim().to_ascii_lowercase();
                match v.as_str() {
                    "public" | "0" => Ok(0),
                    "admin_only" | "admin-only" | "1" => Ok(1),
                    _ => Err(PipelineError::message(
                        "visibility must be public|admin_only or 0|1",
                    )),
                }
            }
            _ => Err(PipelineError::message("invalid visibility value")),
        }
    }

    fn is_admin(context: &HttpContext) -> bool {
        context
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().contains("admin"))
            .unwrap_or(false)
    }
}
