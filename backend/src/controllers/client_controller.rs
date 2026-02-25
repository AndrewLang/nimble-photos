use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::controllers::httpcontext_extensions::HttpContextExtensions;
use crate::entities::client::Client;
use crate::services::SettingService;

use nimble_web::controller::controller::Controller;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::{QueryBuilder, get, put};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientResponse {
    id: Uuid,
    user_id: Uuid,
    name: String,
    is_active: bool,
    is_approved: bool,
    last_seen_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<Client> for ClientResponse {
    fn from(value: Client) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            name: value.name,
            is_active: value.is_active,
            is_approved: value.is_approved,
            last_seen_at: value.last_seen_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

pub struct ClientHandlers;

impl Controller for ClientHandlers {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}

struct ListClientsHandler;

#[async_trait]
#[get("/api/clients", policy = Policy::Authenticated)]
impl HttpHandler for ListClientsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        context.require_admin()?;

        let repo = context.service::<Repository<Client>>()?;
        let query = QueryBuilder::new().page(1, 100).build();
        let page = repo
            .query(query)
            .await
            .map_err(|_| PipelineError::message("failed to query clients"))?;

        let mut clients = page.items;
        clients.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let response = clients
            .into_iter()
            .map(ClientResponse::from)
            .collect::<Vec<_>>();
        Ok(ResponseValue::json(response))
    }
}

struct ApproveClientHandler;

#[async_trait]
#[put("/api/clients/{id}/approve", policy = Policy::Authenticated)]
impl HttpHandler for ApproveClientHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        context.require_admin()?;

        let setting_service = context.service::<SettingService>()?;
        let policy = setting_service.client_approval_policy().await?;
        if policy != "manual" {
            return Err(PipelineError::message(
                "Client approval is only available when approval policy is manual",
            ));
        }

        let client_id = context.id("id")?;
        let repo = context.service::<Repository<Client>>()?;
        let mut client = repo
            .get(&client_id)
            .await
            .map_err(|_| PipelineError::message("failed to load client"))?
            .ok_or_else(|| PipelineError::message("client not found"))?;

        let approver_id = context.current_user_id()?;
        client.is_approved = true;
        client.is_active = true;
        client.approved_by = Some(approver_id);
        client.updated_at = Utc::now();

        let updated = repo
            .update(client)
            .await
            .map_err(|_| PipelineError::message("failed to approve client"))?;
        Ok(ResponseValue::json(ClientResponse::from(updated)))
    }
}

struct RevokeClientHandler;

#[async_trait]
#[put("/api/clients/{id}/revoke", policy = Policy::Authenticated)]
impl HttpHandler for RevokeClientHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        context.require_admin()?;

        let client_id = context.id("id")?;
        let repo = context.service::<Repository<Client>>()?;
        let mut client = repo
            .get(&client_id)
            .await
            .map_err(|_| PipelineError::message("failed to load client"))?
            .ok_or_else(|| PipelineError::message("client not found"))?;

        client.is_active = false;
        client.updated_at = Utc::now();

        let updated = repo
            .update(client)
            .await
            .map_err(|_| PipelineError::message("failed to revoke client"))?;
        Ok(ResponseValue::json(ClientResponse::from(updated)))
    }
}
