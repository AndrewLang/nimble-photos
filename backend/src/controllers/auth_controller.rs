use async_trait::async_trait;
use serde::Deserialize;
use std::default::Default;

use chrono::Utc;

use crate::dtos::user_profile_dto::UserProfileDto;
use crate::entities::{user::User, user_settings::UserSettings};
use crate::services::AuthService;

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
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub display_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl Default for LoginRequest {
    fn default() -> Self {
        Self {
            email: "".to_string(),
            password: "".to_string(),
        }
    }
}

pub struct AuthController;

impl Controller for AuthController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::post("/api/auth/register", RegisterHandler).build(),
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
        let payload: LoginRequest = context.json()?;

        let auth_service = context.service::<AuthService>()?;
        let response = auth_service
            .login(&payload.email, &payload.password)
            .await?;

        Ok(ResponseValue::json(response))
    }
}

struct RegisterHandler;

#[async_trait]
impl HttpHandler for RegisterHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: RegisterRequest = context.json()?;

        if payload.password != payload.confirm_password {
            return Err(PipelineError::message("Passwords do not match"));
        }

        let auth_service = context.service::<AuthService>()?;
        let response = auth_service
            .register(&payload.email, &payload.password)
            .await?;

        Ok(ResponseValue::json(response))
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

        let user_repo = context.service::<Repository<User>>()?;
        let settings_repo = context.service::<Repository<UserSettings>>()?;

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
