use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

#[cfg(feature = "testbot")]
use serde::{Deserialize, Serialize};

use crate::dtos::auth_dtos::{
    ChangePasswordRequest, LoginRequest, LogoutRequest, RefreshTokenRequest, RegisterRequest,
    RegistrationStatusResponse, ResetPasswordRequest, VerifyEmailRequest,
};
use crate::dtos::user_profile_dto::UserProfileDto;
use crate::entities::{user::User, user_settings::UserSettings};
use crate::services::{AuthService, SettingService};

use nimble_web::controller::controller::Controller;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct AuthController;

impl Controller for AuthController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::post("/api/auth/register", RegisterHandler).build(),
            EndpointRoute::post("/api/auth/login", LoginHandler).build(),
            EndpointRoute::post("/api/auth/refresh", RefreshHandler).build(),
            EndpointRoute::post("/api/auth/logout", LogoutHandler).build(),
            EndpointRoute::post("/api/auth/change-password", ChangePasswordHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::post("/api/auth/reset-password", ResetPasswordHandler).build(),
            EndpointRoute::post("/api/auth/verify-email", VerifyEmailHandler).build(),
            EndpointRoute::get("/api/auth/registration-status", RegistrationStatusHandler).build(),
            EndpointRoute::get("/api/auth/me", MeHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            #[cfg(feature = "testbot")]
            EndpointRoute::post("/api/test/auth/reset-token", TestResetTokenHandler).build(),
            #[cfg(feature = "testbot")]
            EndpointRoute::post("/api/test/auth/verify-token", TestVerifyTokenHandler).build(),
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
            .register(&payload.email, &payload.password, &payload.display_name)
            .await?;

        Ok(ResponseValue::json(response))
    }
}

struct RegistrationStatusHandler;

#[async_trait]
impl HttpHandler for RegistrationStatusHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let auth_service = context.service::<AuthService>()?;
        let setting_service = context.service::<SettingService>()?;

        let has_admin = auth_service.has_admin_user().await?;
        let allow_registration = setting_service.is_registration_allowed().await?;

        Ok(ResponseValue::json(RegistrationStatusResponse {
            has_admin,
            allow_registration,
        }))
    }
}

struct MeHandler;

#[async_trait]
impl HttpHandler for MeHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let identity = context
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?;

        let subject = identity.identity().subject().to_string();
        let user_id =
            Uuid::parse_str(&subject).map_err(|_| PipelineError::message("invalid identity"))?;

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
                user_id,
                display_name: user.email.clone(),
                avatar_url: None,
                theme: "light".to_string(),
                language: "en".to_string(),
                timezone: "UTC".to_string(),
                created_at: Utc::now(),
            });

        let dto: UserProfileDto = (user, settings).into();

        Ok(ResponseValue::json(dto))
    }
}

struct RefreshHandler;

#[async_trait]
impl HttpHandler for RefreshHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: RefreshTokenRequest = context.json()?;
        let auth_service = context.service::<AuthService>()?;
        let response = auth_service.refresh(&payload.refresh_token).await?;

        Ok(ResponseValue::json(response))
    }
}

struct LogoutHandler;

#[async_trait]
impl HttpHandler for LogoutHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: LogoutRequest = context.json()?;
        let auth_service = context.service::<AuthService>()?;
        auth_service.logout(&payload.refresh_token)?;

        Ok(ResponseValue::empty())
    }
}

struct ChangePasswordHandler;

#[async_trait]
impl HttpHandler for ChangePasswordHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: ChangePasswordRequest = context.json()?;
        let identity = context
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?;
        let user_id = identity.identity().subject().to_string();

        let auth_service = context.service::<AuthService>()?;
        let old_pw = payload.old_password.clone();
        let new_pw = payload.new_password.clone();

        auth_service
            .change_password(&user_id, &old_pw, &new_pw)
            .await?;

        Ok(ResponseValue::empty())
    }
}

struct ResetPasswordHandler;

#[async_trait]
impl HttpHandler for ResetPasswordHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: ResetPasswordRequest = context.json()?;
        let auth_service = context.service::<AuthService>()?;
        let token = payload.token.clone();
        let new_pw = payload.new_password.clone();

        auth_service.reset_password(&token, &new_pw).await?;

        Ok(ResponseValue::empty())
    }
}

struct VerifyEmailHandler;

#[async_trait]
impl HttpHandler for VerifyEmailHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: VerifyEmailRequest = context.json()?;
        let auth_service = context.service::<AuthService>()?;
        let token = payload.token.clone();

        auth_service.verify_email(&token).await?;

        Ok(ResponseValue::empty())
    }
}

#[cfg(feature = "testbot")]
#[derive(Deserialize)]
struct TokenRequest {
    email: String,
}

#[cfg(feature = "testbot")]
#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

#[cfg(feature = "testbot")]
struct TestResetTokenHandler;

#[cfg(feature = "testbot")]
#[async_trait]
impl HttpHandler for TestResetTokenHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: TokenRequest = context.json()?;
        let auth_service = context.service::<AuthService>()?;
        let token = auth_service.issue_reset_token(&payload.email).await?;
        Ok(ResponseValue::json(TokenResponse { token }))
    }
}

#[cfg(feature = "testbot")]
struct TestVerifyTokenHandler;

#[cfg(feature = "testbot")]
#[async_trait]
impl HttpHandler for TestVerifyTokenHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload: TokenRequest = context.json()?;
        let auth_service = context.service::<AuthService>()?;
        let token = auth_service
            .issue_verification_token(&payload.email)
            .await?;
        Ok(ResponseValue::json(TokenResponse { token }))
    }
}
