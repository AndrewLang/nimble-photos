use async_trait::async_trait;
use nimble_web::DataProvider;
use nimble_web::data::query::{Filter, FilterOperator, Query, Value};
use nimble_web::data::repository::Repository;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use urlencoding::decode;
use uuid::Uuid;

use crate::entities::client::Client;
use crate::entities::client_storage::ClientStorage;
use crate::entities::permission::Permission;
use crate::entities::photo_browse::BrowseOptions;
use crate::entities::photo_browse::BrowseRequest;

#[async_trait]
pub trait HttpContextExtensions {
    fn require(&self, permission: Permission) -> Result<(), PipelineError>;
    fn require_admin(&self) -> Result<(), PipelineError>;
    fn route_uuid(&self, key: &str) -> Result<Uuid, PipelineError>;
    fn current_user_id(&self) -> Result<Uuid, PipelineError>;
    fn extract_api_key(&self) -> Result<String, PipelineError>;
    fn parse_browse_request(&self) -> Result<BrowseRequest, PipelineError>;
    fn route_storage_id(&self) -> Result<Uuid, PipelineError>;
    fn current_client_id(&self) -> Result<Uuid, PipelineError>;
    async fn load_client_storage_settings(
        &self,
        client_id: Uuid,
        storage_id: Uuid,
    ) -> Result<BrowseOptions, PipelineError>;
    async fn validate_api_key(&mut self, api_key: &str) -> Result<Client, PipelineError>;
}

#[async_trait]
impl HttpContextExtensions for HttpContext {
    fn require(&self, permission: Permission) -> Result<(), PipelineError> {
        match permission {
            Permission::BrowsePhotos => {
                let allowed = self
                    .get::<IdentityContext>()
                    .map(|identity| identity.is_authenticated())
                    .unwrap_or(false);
                if allowed {
                    Ok(())
                } else {
                    Err(PipelineError::message("forbidden"))
                }
            }
        }
    }
    fn require_admin(&self) -> Result<(), PipelineError> {
        let is_admin = self
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().contains("admin"))
            .unwrap_or(false);
        if !is_admin {
            return Err(PipelineError::message("forbidden"));
        }
        Ok(())
    }

    fn route_uuid(&self, key: &str) -> Result<Uuid, PipelineError> {
        let raw = self
            .route()
            .and_then(|route| route.params().get(key))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        Uuid::parse_str(raw).map_err(|_| PipelineError::message("invalid uuid"))
    }

    fn parse_browse_request(&self) -> Result<BrowseRequest, PipelineError> {
        let params = self.request().query_params();

        let page_size = params
            .get("pageSize")
            .map(|value| value.parse::<i64>())
            .transpose()
            .map_err(|_| PipelineError::message("invalid pageSize"))?;

        let path = params
            .get("path")
            .map(|value| {
                decode(value)
                    .map(|decoded| decoded.into_owned())
                    .map_err(|_| PipelineError::message("invalid path encoding"))
            })
            .transpose()?;

        let cursor = params
            .get("cursor")
            .map(|value| {
                decode(value)
                    .map(|decoded| decoded.into_owned())
                    .map_err(|_| PipelineError::message("invalid cursor encoding"))
            })
            .transpose()?;

        Ok(BrowseRequest {
            path,
            page_size,
            cursor,
        })
    }

    fn route_storage_id(&self) -> Result<Uuid, PipelineError> {
        let raw = self
            .route()
            .and_then(|route| route.params().get("storageId"))
            .cloned()
            .ok_or_else(|| PipelineError::message("storageId parameter missing"))?;
        Uuid::parse_str(&raw).map_err(|_| PipelineError::message("invalid storageId"))
    }

    fn current_client_id(&self) -> Result<Uuid, PipelineError> {
        let subject = self
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?
            .identity()
            .subject()
            .to_string();

        Uuid::parse_str(&subject).map_err(|_| PipelineError::message("invalid identity"))
    }

    async fn load_client_storage_settings(
        &self,
        client_id: Uuid,
        storage_id: Uuid,
    ) -> Result<BrowseOptions, PipelineError> {
        let repository = self.service::<Repository<ClientStorage>>()?;
        let mut query = Query::<ClientStorage>::new();
        query.filters.push(Filter {
            field: "client_id".to_string(),
            operator: FilterOperator::Eq,
            value: Value::Uuid(client_id),
        });

        let configured = repository
            .query(query)
            .await
            .map_err(|_| PipelineError::message("failed to load client storage settings"))?
            .items
            .into_iter()
            .next();

        if let Some(settings) = configured {
            if settings.storage_id == storage_id {
                return Ok(settings.browse_options);
            }
        }

        Ok(BrowseOptions::default())
    }

    fn current_user_id(&self) -> Result<Uuid, PipelineError> {
        let subject = self
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?
            .identity()
            .subject()
            .to_string();
        Uuid::parse_str(&subject).map_err(|_| PipelineError::message("invalid identity"))
    }

    fn extract_api_key(&self) -> Result<String, PipelineError> {
        let raw = self
            .request()
            .headers()
            .get("authorization")
            .and_then(|header| header.strip_prefix("ApiKey "))
            .map(|token| token.to_string())
            .ok_or_else(|| PipelineError::message("apiKey parameter missing"))?;

        decode(&raw)
            .map(|v| v.into_owned())
            .map_err(|_| PipelineError::message("invalid apiKey encoding"))
    }

    async fn validate_api_key(&mut self, api_key: &str) -> Result<Client, PipelineError> {
        let client_repo = self.service::<Repository<Client>>()?;
        let clients = client_repo
            .query(Query::<Client>::new())
            .await
            .map_err(|_| PipelineError::message("failed to query clients"))?
            .items;

        let client = clients.into_iter().find(|client| {
            client.is_active && client.is_approved && api_key == client.api_key_hash
        });

        match client {
            Some(client) => Ok(client),
            None => {
                self.response_mut().set_status(401);
                Err(PipelineError::message("invalid api key"))
            }
        }
    }
}
