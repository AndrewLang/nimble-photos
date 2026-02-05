use crate::services::SettingService;
use async_trait::async_trait;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::middleware::Middleware;
use nimble_web::pipeline::next::Next;
use nimble_web::pipeline::pipeline::PipelineError;

pub struct PublicAccessMiddleware;

impl PublicAccessMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for PublicAccessMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        const PHOTOS_PREFIX: &str = "/api/photos";

        let path = context.request().path();

        if path.starts_with(PHOTOS_PREFIX) {
            let settings = context.service::<SettingService>()?;
            let authenticated = context
                .get::<IdentityContext>()
                .map(|ctx| ctx.is_authenticated())
                .unwrap_or(false);
            let method = context.request().method();

            if method == "GET" {
                let site_public = settings.is_site_public().await?;
                if !site_public && !authenticated {
                    context.response_mut().set_status(401);
                    return Ok(());
                }
            }

            if method == "POST" && (path == "/api/photos" || path == "/api/photos/scan") {
                let uploads_enabled = settings.is_photo_upload_enabled().await?;
                if !uploads_enabled {
                    context.response_mut().set_status(403);
                    return Ok(());
                }
            }
        }

        next.run(context).await
    }
}
