use std::collections::HashSet;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use nimble_web::DataProvider;
use nimble_web::RequestBody;
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
use crate::entities::user_settings::UserSettings;
use crate::models::setting_consts::SettingConsts;
use crate::repositories::photo_repo::PhotoRepositoryExtensions;
use crate::services::SettingService;

#[async_trait]
pub trait HttpContextExtensions {
    fn require(&self, permission: Permission) -> Result<(), PipelineError>;
    fn require_admin(&self) -> Result<(), PipelineError>;
    fn current_user_id(&self) -> Result<Uuid, PipelineError>;
    fn extract_api_key(&self) -> Result<String, PipelineError>;
    fn parse_browse_request(&self) -> Result<BrowseRequest, PipelineError>;
    fn route_storage_id(&self) -> Result<Uuid, PipelineError>;
    fn hash(&self) -> Result<String, PipelineError>;
    fn default_preview_root(&self) -> PathBuf;
    fn is_admin(&self) -> bool;
    fn is_viewer(&self) -> bool;
    fn entity_id(&self) -> Result<Uuid, PipelineError>;
    fn page(&self) -> Result<u32, PipelineError>;
    fn page_size(&self) -> Result<u32, PipelineError>;
    fn param(&self, key: &str) -> Result<String, PipelineError>;
    fn id(&self, key: &str) -> Result<Uuid, PipelineError>;
    fn body_bytes(&self) -> Result<Vec<u8>, PipelineError>;
    async fn current_user_display_name(&self) -> Result<String, PipelineError>;
    async fn can_upload_photos(&self) -> Result<bool, PipelineError>;
    async fn can_access_dashboard(&self) -> Result<bool, PipelineError>;
    async fn can_update_setting(&self, key: &str) -> Result<bool, PipelineError>;
    async fn viewer_hidden_tags(&self) -> Result<HashSet<String>, PipelineError>;
    async fn current_client_id(&self) -> Result<Uuid, PipelineError>;
    async fn is_preview_exists(&self, hash: &str) -> bool;
    async fn load_client_storage_settings(
        &self,
        client_id: Uuid,
        storage_id: Uuid,
    ) -> Result<BrowseOptions, PipelineError>;
    async fn validate_api_key(&mut self, api_key: &str) -> Result<Client, PipelineError>;
    async fn get_preview_root(&self, hash: &str) -> Result<PathBuf, PipelineError>;
    async fn get_preview_path(&self, hash: &str) -> Result<PathBuf, PipelineError>;
    async fn get_preview_root_by_storage(&self, storage_id: Uuid)
    -> Result<PathBuf, PipelineError>;
    async fn get_preview_path_by_storage(
        &self,
        storage_id: Uuid,
        hash: &str,
    ) -> Result<PathBuf, PipelineError>;
    async fn get_thumbnail_root_by_storage(
        &self,
        storage_id: Uuid,
    ) -> Result<PathBuf, PipelineError>;
    async fn get_thumbnail_roots(&self) -> Result<Vec<PathBuf>, PipelineError>;
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

    fn is_admin(&self) -> bool {
        self.get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().contains("admin"))
            .unwrap_or(false)
    }

    fn is_viewer(&self) -> bool {
        self.get::<IdentityContext>()
            .map(|ctx| {
                let identity = ctx.identity();
                let roles = identity.claims().roles();
                roles.contains("viewer") && !roles.contains("admin")
            })
            .unwrap_or(false)
    }

    fn entity_id(&self) -> Result<Uuid, PipelineError> {
        let id = self
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        Uuid::parse_str(id).map_err(|_| PipelineError::message("invalid album id"))
    }

    fn param(&self, key: &str) -> Result<String, PipelineError> {
        self.route()
            .and_then(|route| route.params().get(key))
            .cloned()
            .ok_or_else(|| PipelineError::message(&format!("{} parameter missing", key)))
    }

    fn id(&self, key: &str) -> Result<Uuid, PipelineError> {
        let id = self
            .route()
            .and_then(|route| route.params().get(key))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;
        Uuid::parse_str(id).map_err(|_| PipelineError::message("invalid album id"))
    }

    fn page(&self) -> Result<u32, PipelineError> {
        let page: u32 = self
            .route()
            .and_then(|route| route.params().get("page"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        Ok(page)
    }

    fn page_size(&self) -> Result<u32, PipelineError> {
        let page: u32 = self
            .route()
            .and_then(|route| route.params().get("pageSize"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        Ok(page)
    }

    fn current_user_id(&self) -> Result<Uuid, PipelineError> {
        let subject = self
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?
            .identity()
            .subject()
            .to_string();
        Uuid::parse_str(&subject)
            .map_err(|_| PipelineError::message("Invalid identity: user ID is not valid"))
    }

    async fn current_user_display_name(&self) -> Result<String, PipelineError> {
        let user_id = self.current_user_id()?;
        let settings_repo = self.service::<Repository<UserSettings>>()?;
        let display_name = settings_repo
            .get(&user_id)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?
            .map(|settings| settings.display_name)
            .unwrap_or_else(|| "Anonymous".to_string());
        Ok(display_name)
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

    fn body_bytes(&self) -> Result<Vec<u8>, PipelineError> {
        match self.request().body() {
            RequestBody::Empty => Ok(Vec::new()),
            RequestBody::Text(text) => Ok(text.as_bytes().to_vec()),
            RequestBody::Bytes(bytes) => Ok(bytes.clone()),
            RequestBody::Stream(stream) => {
                let mut guard = stream
                    .lock()
                    .map_err(|_| PipelineError::message("request body stream lock error"))?;
                let mut collected = Vec::<u8>::new();
                loop {
                    let next_chunk = guard
                        .read_chunk()
                        .map_err(|error| PipelineError::message(&error.to_string()))?;
                    match next_chunk {
                        Some(chunk) => collected.extend_from_slice(&chunk),
                        None => break,
                    }
                }
                Ok(collected)
            }
        }
    }

    async fn can_upload_photos(&self) -> Result<bool, PipelineError> {
        let roles = self
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().clone())
            .unwrap_or_default();
        self.service::<SettingService>()?
            .can_upload_photos(&roles)
            .await
    }
    async fn can_access_dashboard(&self) -> Result<bool, PipelineError> {
        let roles = self
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().clone())
            .unwrap_or_default();
        self.service::<SettingService>()?
            .can_access_dashboard(&roles)
            .await
    }

    async fn can_update_setting(&self, key: &str) -> Result<bool, PipelineError> {
        let roles = self
            .get::<IdentityContext>()
            .map(|ctx| ctx.identity().claims().roles().clone())
            .unwrap_or_default();
        self.service::<SettingService>()?
            .can_update_setting(&roles, key)
            .await
    }

    async fn viewer_hidden_tags(&self) -> Result<HashSet<String>, PipelineError> {
        if !self.is_viewer() {
            return Ok(HashSet::new());
        }
        let settings = self.service::<SettingService>()?;
        settings.viewer_hidden_tags().await
    }

    async fn current_client_id(&self) -> Result<Uuid, PipelineError> {
        if let Some(identity) = self.get::<IdentityContext>() {
            let subject = identity.identity().subject().to_string();
            if let Ok(id) = Uuid::parse_str(&subject) {
                return Ok(id);
            }
        }

        let api_key = self.extract_api_key()?;

        let repository = self.service::<Repository<Client>>()?;
        let client = repository
            .get_by("api_key_hash", Value::String(api_key.clone()))
            .await
            .map_err(|_| PipelineError::message("failed to query client by api key"))?;

        match client {
            Some(client) if client.is_active && client.is_approved => Ok(client.id),
            _ => Err(PipelineError::message("Invalid api key")),
        }
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

    async fn validate_api_key(&mut self, api_key: &str) -> Result<Client, PipelineError> {
        let client_repo = self.service::<Repository<Client>>()?;
        let client = client_repo
            .get_by("api_key_hash", Value::String(api_key.to_string()))
            .await
            .map_err(|_| PipelineError::message("failed to query client by api key"))?;

        match client {
            Some(client) if client.is_active && client.is_approved => Ok(client),
            None => {
                self.response_mut().set_status(401);
                Err(PipelineError::message("Invalid api key"))
            }
            Some(_) => {
                self.response_mut().set_status(401);
                Err(PipelineError::message("Client is not active or approved"))
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

    async fn get_preview_root_by_storage(
        &self,
        storage_id: Uuid,
    ) -> Result<PathBuf, PipelineError> {
        let storage_repo = self.service::<Repository<StorageLocation>>()?;
        let storage = storage_repo
            .get(&storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage location"))?
            .ok_or_else(|| PipelineError::message("storage location not found"))?;
        Ok(storage.normalized_path().join(".previews"))
    }

    async fn get_preview_path_by_storage(
        &self,
        storage_id: Uuid,
        hash: &str,
    ) -> Result<PathBuf, PipelineError> {
        let preview_root = self.get_preview_root_by_storage(storage_id).await?;
        Ok(preview_root
            .join(&hash[0..2])
            .join(&hash[2..4])
            .join(format!("{hash}.jpg")))
    }

    async fn get_thumbnail_root_by_storage(
        &self,
        storage_id: Uuid,
    ) -> Result<PathBuf, PipelineError> {
        let storage_repo = self.service::<Repository<StorageLocation>>()?;
        let storage = storage_repo
            .get(&storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage location"))?
            .ok_or_else(|| PipelineError::message("storage location not found"))?;
        Ok(storage
            .normalized_path()
            .join(SettingConsts::THUMBNAIL_FOLDER))
    }

    async fn get_thumbnail_roots(&self) -> Result<Vec<PathBuf>, PipelineError> {
        let mut roots = Vec::<PathBuf>::new();
        if let Ok(storage_repo) = self.service::<Repository<StorageLocation>>() {
            if let Ok(page) = storage_repo.query(Query::<StorageLocation>::new()).await {
                for location in page.items {
                    let path = location
                        .normalized_path()
                        .join(SettingConsts::THUMBNAIL_FOLDER);
                    if !roots.contains(&path) {
                        roots.push(path);
                    }
                }
            }
        }

        let config = self.config();
        let default_legacy_base = format!("./{}", SettingConsts::THUMBNAIL_FOLDER);
        let legacy_base = config
            .get("thumbnail.base.path")
            .or_else(|| config.get("thumbnail.basepath"))
            .unwrap_or(default_legacy_base.as_str());
        let legacy_path = Path::new(legacy_base).to_path_buf();
        if !roots.contains(&legacy_path) {
            roots.push(legacy_path);
        }

        Ok(roots)
    }

    async fn is_preview_exists(&self, hash: &str) -> bool {
        match self.get_preview_path(hash).await {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }
}
