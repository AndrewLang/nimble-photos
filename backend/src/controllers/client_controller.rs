use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::controllers::httpcontext_extensions::HttpContextExtentions;
use crate::entities::client::Client;
use crate::services::SettingService;

use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::Query;
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::{delete, get, post, put};

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterClientRequest {
    name: String,
    api_key_hash: String,
}

struct ListClientsHandler;

#[async_trait]
#[get("/api/clients", policy = Policy::Authenticated)]
impl HttpHandler for ListClientsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        HttpContextExtentions::require_admin(context)?;

        let repo = context.service::<Repository<Client>>()?;
        let page = repo
            .query(Query::<Client>::new())
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
        HttpContextExtentions::require_admin(context)?;

        let setting_service = context.service::<SettingService>()?;
        let policy = setting_service.client_approval_policy().await?;
        if policy != "manual" {
            return Err(PipelineError::message(
                "Client approval is only available when approval policy is manual",
            ));
        }

        let client_id = HttpContextExtentions::route_uuid(context, "id")?;
        let repo = context.service::<Repository<Client>>()?;
        let mut client = repo
            .get(&client_id)
            .await
            .map_err(|_| PipelineError::message("failed to load client"))?
            .ok_or_else(|| PipelineError::message("client not found"))?;

        client.is_approved = true;
        client.is_active = true;
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
        HttpContextExtentions::require_admin(context)?;

        let client_id = HttpContextExtentions::route_uuid(context, "id")?;
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

struct DeleteClientHandler;

#[async_trait]
#[delete("/api/clients/{id}", policy = Policy::Authenticated)]
impl HttpHandler for DeleteClientHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        HttpContextExtentions::require_admin(context)?;

        let client_id = HttpContextExtentions::route_uuid(context, "id")?;
        let repo = context.service::<Repository<Client>>()?;
        let deleted = repo
            .delete(&client_id)
            .await
            .map_err(|_| PipelineError::message("failed to delete client"))?;
        if !deleted {
            return Err(PipelineError::message("client not found"));
        }

        Ok(ResponseValue::json(serde_json::json!({ "deleted": true })))
    }
}

struct RegisterClientHandler;

#[async_trait]
#[post("/api/clients/register", policy = Policy::Authenticated)]
impl HttpHandler for RegisterClientHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let request = context
            .read_json::<RegisterClientRequest>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let name = request.name.trim();
        if name.is_empty() {
            return Err(PipelineError::message("client name is required"));
        }

        let api_key_hash = request.api_key_hash.trim();
        if api_key_hash.is_empty() {
            return Err(PipelineError::message("apiKeyHash is required"));
        }

        let user_id = HttpContextExtentions::current_user_id(context)?;
        let setting_service = context.service::<SettingService>()?;
        let policy = setting_service.client_approval_policy().await?;
        let is_approved = policy == "auto";
        let now = Utc::now();

        let repo = context.service::<Repository<Client>>()?;
        let client = Client {
            id: Uuid::new_v4(),
            user_id,
            name: name.to_string(),
            api_key_hash: api_key_hash.to_string(),
            is_active: is_approved,
            is_approved,
            last_seen_at: None,
            created_at: now,
            updated_at: now,
        };

        let saved = repo
            .insert(client)
            .await
            .map_err(|_| PipelineError::message("failed to register client"))?;

        Ok(ResponseValue::json(ClientResponse::from(saved)))
    }
}
