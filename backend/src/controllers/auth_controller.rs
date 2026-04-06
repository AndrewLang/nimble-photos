use async_trait::async_trait;
use chrono::Utc;

#[cfg(feature = "testbot")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use serde_json::json;

pub struct AuthController;

impl Controller for AuthController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::post("/api/auth/register", RegisterHandler).build(),
            EndpointRoute::post("/api/auth/login", LoginHandler).build(),
            EndpointRoute::post("/api/auth/refresh", RefreshHandler).build(),
            EndpointRoute::post("/api/auth/logout", LogoutHandler).build(),
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
        let setting_service = context.service::<SettingService>()?;
        let response = auth_service
            .register(&payload.email, &payload.password, &payload.display_name)
            .await?;
        setting_service
            .update("site.initialized", json!(true))
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
        let mut initialized = setting_service.is_site_initialized().await?;

        if has_admin && !initialized {
            setting_service
                .update("site.initialized", json!(true))
                .await?;
            initialized = true;
        }

        Ok(ResponseValue::json(RegistrationStatusResponse {
            has_admin,
            allow_registration,
            initialized,
        }))
    }
}

struct MeHandler;

#[async_trait]
impl HttpHandler for MeHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let user_id = context.current_user_id()?;
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
