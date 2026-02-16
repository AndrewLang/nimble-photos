use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::dtos::dashboard_settings_dto::{LogoUploadRequest, UpdateSettingPayload};
use crate::services::SettingService;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;

pub struct DashboardController;

impl Controller for DashboardController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/dashboard/settings", ListSettingsHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::get("/api/dashboard/settings/{key}", GetSettingHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::put("/api/dashboard/settings/{key}", UpdateSettingHandler)
                .with_policy(Policy::Authenticated)
                .build(),
            EndpointRoute::post(
                "/api/dashboard/settings/site.logo/upload",
                UploadLogoHandler,
            )
            .with_policy(Policy::Authenticated)
            .build(),
        ]
    }
}

struct ListSettingsHandler;

#[async_trait]
impl HttpHandler for ListSettingsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let service = context.service::<SettingService>()?;
        if !DashboardController::can_access_dashboard(context, &service).await? {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }
        let settings = service.list().await?;
        Ok(ResponseValue::json(settings))
    }
}

struct UpdateSettingHandler;

#[async_trait]
impl HttpHandler for UpdateSettingHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<UpdateSettingPayload>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let key = context
            .route()
            .and_then(|route| route.params().get("key"))
            .ok_or_else(|| PipelineError::message("key parameter missing"))?;

        let service = context.service::<SettingService>()?;
        if !DashboardController::can_update_setting(context, &service, key).await? {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }
        let updated = service.update(key, payload.value).await?;

        Ok(ResponseValue::json(updated))
    }
}

struct GetSettingHandler;

#[async_trait]
impl HttpHandler for GetSettingHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let key = context
            .route()
            .and_then(|route| route.params().get("key"))
            .ok_or_else(|| PipelineError::message("key parameter missing"))?;

        let service = context.service::<SettingService>()?;
        if !DashboardController::can_access_dashboard(context, &service).await? {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }
        let setting = service.get(key).await?;

        Ok(ResponseValue::json(setting))
    }
}

struct UploadLogoHandler;

impl UploadLogoHandler {
    fn parse_data_url(data_url: &str) -> Result<(&str, &str), PipelineError> {
        let (meta, data) = data_url
            .split_once(',')
            .ok_or_else(|| PipelineError::message("Invalid data URL"))?;

        if !meta.starts_with("data:") {
            return Err(PipelineError::message("Invalid data URL"));
        }

        let meta = &meta[5..];
        let (mime, encoding) = meta
            .split_once(';')
            .ok_or_else(|| PipelineError::message("Invalid data URL"))?;

        if encoding != "base64" {
            return Err(PipelineError::message("Logo must be base64 encoded"));
        }

        Ok((mime, data))
    }
}

#[async_trait]
impl HttpHandler for UploadLogoHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<LogoUploadRequest>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let (mime, encoded) = Self::parse_data_url(&payload.data_url)?;
        let extension = match mime {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/jpg" => "jpg",
            "image/svg+xml" => "svg",
            _ => {
                return Err(PipelineError::message(
                    "Unsupported logo format. Use PNG, JPG, or SVG.",
                ));
            }
        };

        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| PipelineError::message("Invalid logo data"))?;

        let folder = Path::new("data").join("logo");
        fs::create_dir_all(&folder)
            .map_err(|_| PipelineError::message("Failed to create logo directory"))?;

        let filename = format!("logo-{}.{}", Uuid::new_v4(), extension);
        let path = folder.join(&filename);
        fs::write(&path, bytes).map_err(|_| PipelineError::message("Failed to save logo"))?;

        let logo_url = format!("/assets/logo/{}", filename);
        let service = context.service::<SettingService>()?;
        if !DashboardController::can_update_setting(context, &service, "site.logo").await? {
            context.response_mut().set_status(403);
            return Ok(ResponseValue::empty());
        }
        let updated = service.update("site.logo", json!(logo_url)).await?;

        Ok(ResponseValue::json(updated))
    }
}

impl DashboardController {
    async fn can_access_dashboard(
        context: &HttpContext,
        service: &SettingService,
    ) -> Result<bool, PipelineError> {
        let roles = context
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().clone())
            .unwrap_or_default();
        service.can_access_dashboard(&roles).await
    }

    async fn can_update_setting(
        context: &HttpContext,
        service: &SettingService,
        key: &str,
    ) -> Result<bool, PipelineError> {
        let roles = context
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().clone())
            .unwrap_or_default();
        service.can_update_setting(&roles, key).await
    }
}
