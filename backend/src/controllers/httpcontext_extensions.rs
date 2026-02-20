use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
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
use crate::entities::photo::Photo;
use crate::entities::photo_browse::BrowseOptions;
use crate::entities::photo_browse::BrowseRequest;
use crate::entities::storage_location::StorageLocation;
use crate::repositories::photo_repo::PhotoRepositoryExtensions;

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
    fn hash(&self) -> Result<String, PipelineError>;
    fn default_preview_root(&self) -> PathBuf;
    async fn is_preview_exists(&self, hash: &str) -> bool;
    async fn load_client_storage_settings(
        &self,
        client_id: Uuid,
        storage_id: Uuid,
    ) -> Result<BrowseOptions, PipelineError>;
    async fn validate_api_key(&mut self, api_key: &str) -> Result<Client, PipelineError>;
    async fn get_preview_root(&self, hash: &str) -> Result<PathBuf, PipelineError>;
    async fn get_preview_path(&self, hash: &str) -> Result<PathBuf, PipelineError>;
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

    fn hash(&self) -> Result<String, PipelineError> {
        let hash = self
            .route()
            .and_then(|route| route.params().get("hash"))
            .cloned()
            .ok_or_else(|| PipelineError::message("hash parameter missing"))?;

        if hash.len() < 4 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PipelineError::message("invalid thumbnail hash"));
        }

        Ok(hash)
    }

    fn default_preview_root(&self) -> PathBuf {
        if cfg!(windows) {
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                return Path::new(&user_profile)
                    .join("AppData")
                    .join("Local")
                    .join("photon");
            }
        }

        PathBuf::from("./previews")
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

    async fn get_preview_root(&self, hash: &str) -> Result<PathBuf, PipelineError> {
        let preview_storage_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .map_err(|_| PipelineError::message("invalid preview storage id"))?;
        let preview_root = self.default_preview_root();

        if let Ok(storage_repo) = self.service::<Repository<StorageLocation>>() {
            match storage_repo.get(&preview_storage_id).await {
                Ok(Some(_)) => {}
                Ok(None) => {
                    if let Err(err) = std::fs::create_dir_all(&preview_root) {
                        log::warn!(
                            "Failed to create default preview root '{}': {:?}",
                            preview_root.display(),
                            err
                        );
                    }

                    let preview_storage = StorageLocation {
                        id: preview_storage_id,
                        label: "Preview Cache".to_string(),
                        path: preview_root.to_string_lossy().to_string(),
                        is_default: false,
                        created_at: Utc::now().to_rfc3339(),
                        category_template: "{year}/{date:%Y-%m-%d}/{fileName}".to_string(),
                    };

                    if let Err(err) = storage_repo.insert(preview_storage).await {
                        log::warn!(
                            "Failed to create preview storage {}: {:?}",
                            preview_storage_id,
                            err
                        );
                    }
                }
                Err(err) => {
                    log::warn!(
                        "Failed to load preview storage {}: {:?}",
                        preview_storage_id,
                        err
                    );
                }
            }
        }

        let photo_repo = self.service::<Repository<Photo>>()?;
        let photo = photo_repo
            .find_by_hash(&hash)
            .await?
            .ok_or_else(|| PipelineError::message("preview not found"))?;

        let storage_id = photo.storage_id;
        if let Ok(storage_repo) = self.service::<Repository<StorageLocation>>() {
            match storage_repo.get(&storage_id).await {
                Ok(Some(storage)) => {
                    return Ok(storage.normalized_path().join(".previews"));
                }
                Ok(None) => {
                    log::warn!(
                        "Storage {} not found while resolving preview for hash {}",
                        storage_id,
                        hash
                    );
                }
                Err(err) => {
                    log::warn!(
                        "Failed to load storage {} for preview hash {}: {:?}",
                        storage_id,
                        hash,
                        err
                    );
                }
            }
        } else {
            log::warn!(
                "Storage repository unavailable while resolving preview for hash {}",
                hash
            );
        }

        Ok(preview_root)
    }

    async fn get_preview_path(&self, hash: &str) -> Result<PathBuf, PipelineError> {
        let preview_root = self.get_preview_root(hash).await?;
        Ok(preview_root
            .join(&hash[0..2])
            .join(&hash[2..4])
            .join(format!("{hash}.jpg")))
    }

    async fn is_preview_exists(&self, hash: &str) -> bool {
        match self.get_preview_path(hash).await {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }
}
