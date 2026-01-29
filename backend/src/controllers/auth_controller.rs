use std::default::Default;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use chrono::Utc;

use crate::dtos::user_profile_dto::UserProfileDto;
use crate::entities::{user::User, user_settings::UserSettings};
use crate::services::{EncryptService, IdGenerationService};

use nimble_web::controller::controller::Controller;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::Value;
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    pub token: String,
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
        let payload: LoginRequest = context
            .read_json()
            .map_err(|err| PipelineError::message(&err.message()))?;

        let repo = context
            .services()
            .resolve::<Repository<User>>()
            .ok_or_else(|| PipelineError::message("user repository not registered"))?;

        let encrypt_service = context
            .services()
            .resolve::<EncryptService>()
            .ok_or_else(|| PipelineError::message("encrypt service not registered"))?;

        let user = repo
            .get_by("email", Value::String(payload.email.clone()))
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("invalid credentials"))?;

        log::debug!("User {} logging in", user.id);

        let decrypted_password = encrypt_service
            .decrypt(&user.password_hash)
            .map_err(|_| PipelineError::message("invalid credentials"))?;

        if payload.password != decrypted_password {
            return Err(PipelineError::message("invalid credentials"));
        }

        Ok(ResponseValue::new(Json(LoginResponse { token: user.id })))
    }
}

struct RegisterHandler;

#[async_trait]
impl HttpHandler for RegisterHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: RegisterRequest = context
            .read_json()
            .map_err(|err| PipelineError::message(&err.message()))?;

        let encrypt_service = context
            .services()
            .resolve::<EncryptService>()
            .ok_or_else(|| PipelineError::message("encrypt service not registered"))?;

        let id_gen_service = context
            .services()
            .resolve::<IdGenerationService>()
            .ok_or_else(|| PipelineError::message("id generation service not registered"))?;

        let encrypted_password = encrypt_service
            .encrypt(&payload.password)
            .map_err(|e| PipelineError::message(&e.to_string()))?;

        let user_id = id_gen_service.generate();

        let user = User {
            id: user_id.clone(),
            password_hash: encrypted_password,
            email: payload.email,
            display_name: payload.display_name,
            created_at: Utc::now(),
        };

        let repo = context
            .services()
            .resolve::<Repository<User>>()
            .ok_or_else(|| PipelineError::message("user repository not registered"))?;

        repo.create(user)
            .await
            .map_err(|_| PipelineError::message("data error"))?;

        Ok(ResponseValue::new(Json(LoginResponse { token: user_id })))
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
