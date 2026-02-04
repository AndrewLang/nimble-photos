use async_trait::async_trait;

use crate::dtos::dashboard_settings_dto::UpdateSettingPayload;
use crate::services::SettingService;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
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
            EndpointRoute::put("/api/dashboard/settings/{key}", UpdateSettingHandler)
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
        let updated = service.update(key, payload.value).await?;

        Ok(ResponseValue::json(updated))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nimble_web::security::policy::Policy;

    #[test]
    fn routes_are_authenticated() {
        let routes = DashboardController::routes();
        assert_eq!(routes.len(), 2);

        let list_route = &routes[0];
        assert_eq!(list_route.route.method(), "GET");
        assert_eq!(list_route.route.path(), "/api/dashboard/settings");
        assert_eq!(
            list_route.endpoint.metadata().policy(),
            Some(&Policy::Authenticated)
        );

        let update_route = &routes[1];
        assert_eq!(update_route.route.method(), "PUT");
        assert_eq!(update_route.route.path(), "/api/dashboard/settings/{key}");
        assert_eq!(
            update_route.endpoint.metadata().policy(),
            Some(&Policy::Authenticated)
        );
    }
}
