use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::controllers::httpcontext_extensions::HttpContextExtensions;
use crate::entities::client::Client;
use crate::services::{EncryptService, SettingService};

use nimble_web::controller::controller::Controller;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::{Query, Value};
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
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
    device_name: String,
    device_type: String,
    client_version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterClientResponse {
    api_key: String,
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
        context.require_admin()?;

        let setting_service = context.service::<SettingService>()?;
        let policy = setting_service.client_approval_policy().await?;
        if policy != "manual" {
            return Err(PipelineError::message(
                "Client approval is only available when approval policy is manual",
            ));
        }

        let client_id = context.route_uuid("id")?;
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

        let client_id = context.route_uuid("id")?;
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
        context.require_admin()?;

        let client_id = context.route_uuid("id")?;
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

impl RegisterClientHandler {
    const AUTO_APPROVED_BY_UUID: &'static str = "00000000-0000-0000-0000-000000000001";

    fn auto_approved_by_uuid() -> Uuid {
        Uuid::parse_str(Self::AUTO_APPROVED_BY_UUID)
            .expect("invalid AUTO_APPROVED_BY_UUID constant")
    }

    fn normalized(value: &str, field_name: &str) -> Result<String, PipelineError> {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Err(PipelineError::message(&format!("{field_name} is required")));
        }

        Ok(normalized.to_string())
    }

    fn create_api_key(
        user_id: Uuid,
        client_id: Uuid,
        device_name: &str,
        device_type: &str,
        client_version: &str,
    ) -> String {
        let header = json!({
            "alg": "NIMBLE",
            "typ": "APIK"
        });
        let payload = json!({
            "sub": user_id,
            "cid": client_id,
            "deviceName": device_name,
            "deviceType": device_type,
            "clientVersion": client_version,
            "iat": Utc::now().timestamp(),
            "jti": Uuid::new_v4()
        });

        let header_part =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
        let payload_part =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());

        let mut signature_bytes = [0u8; 32];
        rand::rng().fill(&mut signature_bytes);
        let signature_part =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature_bytes);

        format!("{header_part}.{payload_part}.{signature_part}")
    }
}

#[async_trait]
#[post("/api/clients/register", policy = Policy::Authenticated)]
impl HttpHandler for RegisterClientHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let request = context
            .read_json::<RegisterClientRequest>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let device_name = Self::normalized(&request.device_name, "deviceName")?;
        let device_type = Self::normalized(&request.device_type, "deviceType")?;
        let client_version = Self::normalized(&request.client_version, "clientVersion")?;

        let user_id = context.current_user_id()?;

        let setting_service = context.service::<SettingService>()?;
        let encrypt_service = context.service::<EncryptService>()?;
        let policy = setting_service.client_approval_policy().await?;
        let is_approved = policy == "auto";
        let now = Utc::now();
        let client_id = Uuid::new_v4();
        let repo = context.service::<Repository<Client>>()?;

        let existing = repo
            .get_by("device_name", Value::String(device_name.clone()))
            .await
            .map_err(|_| PipelineError::message("failed to query existing client"))?;

        if let Some(existing_client) = existing {
            let response = RegisterClientResponse {
                api_key: existing_client.api_key_hash.clone(),
            };

            return Ok(ResponseValue::json(response));
        }

        let api_key = Self::create_api_key(
            user_id,
            client_id,
            &device_name,
            &device_type,
            &client_version,
        );
        let api_key_hash = encrypt_service
            .encrypt(&api_key)
            .map_err(|_| PipelineError::message("failed to protect api key"))?;

        let client = Client {
            id: client_id,
            user_id,
            name: device_name.clone(),
            device_name,
            device_type,
            version: client_version,
            api_key_hash,
            is_active: is_approved,
            is_approved,
            approved_by: if is_approved {
                Some(Self::auto_approved_by_uuid())
            } else {
                None
            },
            last_seen_at: now.into(),
            created_at: now,
            updated_at: now,
        };

        let _saved = repo
            .insert(client)
            .await
            .map_err(|_| PipelineError::message("failed to register client"))?;

        let response = RegisterClientResponse { api_key };

        Ok(ResponseValue::json(response))
    }
}
