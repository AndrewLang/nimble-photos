use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use uuid::Uuid;

pub struct HttpContextExtentions;

impl HttpContextExtentions {
    pub fn require_admin(context: &HttpContext) -> Result<(), PipelineError> {
        let is_admin = context
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().contains("admin"))
            .unwrap_or(false);
        if !is_admin {
            return Err(PipelineError::message("forbidden"));
        }
        Ok(())
    }

    pub fn route_uuid(context: &HttpContext, key: &str) -> Result<Uuid, PipelineError> {
        let raw = context
            .route()
            .and_then(|route| route.params().get(key))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        Uuid::parse_str(raw).map_err(|_| PipelineError::message("invalid uuid"))
    }

    pub fn current_user_id(context: &HttpContext) -> Result<Uuid, PipelineError> {
        let subject = context
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?
            .identity()
            .subject()
            .to_string();
        Uuid::parse_str(&subject).map_err(|_| PipelineError::message("invalid identity"))
    }
}
