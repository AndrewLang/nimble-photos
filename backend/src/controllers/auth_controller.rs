use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use chrono::Utc;

use crate::dtos::user_profile_dto::UserProfileDto;
use crate::entities::{user::User, user_settings::UserSettings};

use nimble_web::controller::controller::Controller;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::Json;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

#[derive(Deserialize)]
struct LoginRequest {
    pub id: String,
}

#[derive(Serialize)]
struct LoginResponse {
    pub token: String,
}

pub struct AuthController;

impl Controller for AuthController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::post("/api/auth/login", LoginHandler).build(),
            EndpointRoute::get("/api/auth/me", MeHandler)
                .with_policy(Policy::Authenticated)
                .build(),
        ]
    }
}

struct LoginHandler;

#[async_trait]
impl HttpHandler for LoginHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: LoginRequest = context
            .read_json()
            .map_err(|err| PipelineError::message(&err.message()))?;

        // For a simple auth flow, issue a token equal to the user id if the user exists.
        if let Some(repo) = context.services().resolve::<Repository<User>>() {
            let user = repo
                .get(&payload.id)
                .await
                .map_err(|_| PipelineError::message("data error"))?;

            if user.is_none() {
                return Err(PipelineError::message("invalid credentials"));
            }
        }

        Ok(ResponseValue::new(Json(LoginResponse {
            token: payload.id,
        })))
    }
}

struct MeHandler;

#[async_trait]
impl HttpHandler for MeHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let identity = context
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?;

        let user_id = identity.identity().subject().to_string();

        let user_repo = context
            .services()
            .resolve::<Repository<User>>()
            .ok_or_else(|| PipelineError::message("user repository not registered"))?;

        let settings_repo = context
            .services()
            .resolve::<Repository<UserSettings>>()
            .ok_or_else(|| PipelineError::message("user settings repository not registered"))?;

        let user = user_repo
            .get(&user_id)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("user not found"))?;

        let settings = settings_repo
            .get(&user_id)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .unwrap_or(UserSettings {
                user_id: user.id.clone(),
                display_name: user.email.clone(),
                avatar_url: None,
                theme: "light".to_string(),
                language: "en".to_string(),
                timezone: "UTC".to_string(),
                created_at: Utc::now(),
            });

        let dto: UserProfileDto = (user, settings).into();

        Ok(ResponseValue::new(Json(dto)))
    }
}
