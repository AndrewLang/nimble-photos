use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use sysinfo::Disks;
use urlencoding::decode;
use uuid::Uuid;

use crate::entities::StorageLocation;
use crate::entities::client_storage::ClientStorage;
use crate::entities::photo_browse::{BrowseOptions, BrowseRequest, BrowseResponse};
use crate::entities::photo_cursor::PhotoCursor;
use crate::services::BrowseService;
use crate::services::SettingService;

use nimble_web::DataProvider;
use nimble_web::controller::controller::Controller;
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::identity::context::IdentityContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::{delete, get, post, put};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLocationResponse {
    pub id: String,
    pub label: String,
    pub path: String,
    pub is_default: bool,
    pub created_at: String,
    pub category_template: String,
    pub disk: Option<DiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStoragePayload {
    pub label: String,
    pub path: String,
    pub is_default: Option<bool>,
    pub category_template: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStoragePayload {
    pub label: Option<String>,
    pub path: Option<String>,
    pub is_default: Option<bool>,
    pub category_template: Option<String>,
}

#[derive(Clone, Copy)]
enum Permission {
    BrowsePhotos,
}

trait HttpContextPermissionExt {
    fn require(&self, permission: Permission) -> Result<(), PipelineError>;
}

impl HttpContextPermissionExt for HttpContext {
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
}

pub struct StorageHandlers;

impl Controller for StorageHandlers {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}

struct StorageSupport;

impl StorageSupport {
    fn list_disks() -> Vec<DiskInfo> {
        let disks = Disks::new_with_refreshed_list();

        let mut items = disks
            .list()
            .iter()
            .filter(|disk| !disk.is_removable())
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
            })
            .collect::<Vec<_>>();

        items.sort_by_key(|disk| Self::disk_sort_key(&disk.mount_point));
        items
    }

    fn disk_sort_key(mount_point: &str) -> (u8, String) {
        let normalized = mount_point.trim().to_ascii_lowercase();
        let bytes = normalized.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            return (0, normalized);
        }
        (1, normalized)
    }

    fn match_disk(path: &str, disks: &[DiskInfo]) -> Option<DiskInfo> {
        let path_lower = path.to_ascii_lowercase();
        disks
            .iter()
            .filter(|disk| !disk.mount_point.is_empty())
            .filter(|disk| path_lower.starts_with(&disk.mount_point.to_ascii_lowercase()))
            .max_by_key(|disk| disk.mount_point.len())
            .cloned()
    }

    async fn load_locations(
        service: &SettingService,
    ) -> Result<Vec<StorageLocation>, PipelineError> {
        let setting = service.get("storage.locations").await?;
        let value = setting.value;
        serde_json::from_value(value)
            .map_err(|_| PipelineError::message("Invalid storage settings"))
    }

    async fn save_locations(
        service: &SettingService,
        locations: &[StorageLocation],
    ) -> Result<(), PipelineError> {
        let value = json!(locations);
        service.update("storage.locations", value).await?;
        Ok(())
    }

    fn parse_browse_request(context: &HttpContext) -> Result<BrowseRequest, PipelineError> {
        let params = context.request().query_params();

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

    fn route_storage_id(context: &HttpContext) -> Result<String, PipelineError> {
        context
            .route()
            .and_then(|route| route.params().get("storageId"))
            .cloned()
            .ok_or_else(|| PipelineError::message("storageId parameter missing"))
    }

    fn current_client_id(context: &HttpContext) -> Result<Uuid, PipelineError> {
        let subject = context
            .get::<IdentityContext>()
            .ok_or_else(|| PipelineError::message("identity not found"))?
            .identity()
            .subject()
            .to_string();

        Uuid::parse_str(&subject).map_err(|_| PipelineError::message("invalid identity"))
    }

    async fn load_client_storage_settings(
        context: &HttpContext,
        client_id: Uuid,
        storage_id: &str,
    ) -> Result<BrowseOptions, PipelineError> {
        let storage_uuid = Uuid::parse_str(storage_id).ok();
        let repository = context.service::<Repository<ClientStorage>>()?;
        let configured = repository
            .get(&client_id)
            .await
            .map_err(|_| PipelineError::message("failed to load client storage settings"))?;

        if let (Some(settings), Some(expected_storage_id)) = (configured, storage_uuid) {
            if settings.storage_id == expected_storage_id {
                return Ok(settings.browse_options);
            }
        }

        Ok(BrowseOptions::default())
    }
}

struct DisksHandler;

#[async_trait]
#[get("/api/storage/disks", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for DisksHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::json(StorageSupport::list_disks()))
    }
}

struct ListStorageHandler;

#[async_trait]
#[get("/api/storage/locations", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for ListStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let service = context.service::<SettingService>()?;
        let locations = StorageSupport::load_locations(&service).await?;
        let disks = StorageSupport::list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = StorageSupport::match_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id,
                    label: location.label,
                    path: location.path,
                    is_default: location.is_default,
                    created_at: location.created_at,
                    category_template: location.category_template,
                    disk,
                }
            })
            .collect::<Vec<_>>();

        Ok(ResponseValue::json(response))
    }
}

struct CreateStorageHandler;

#[async_trait]
#[post("/api/storage/locations", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for CreateStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let payload = context
            .read_json::<CreateStoragePayload>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let label_value = payload.label.trim();
        if label_value.is_empty() {
            return Err(PipelineError::message("Storage label is required"));
        }

        let path_value = payload.path.trim();
        if path_value.is_empty() {
            return Err(PipelineError::message("Storage path is required"));
        }

        if !Path::new(path_value).exists() {
            log::warn!(
                "Storage path does not exist: {}, will create it.",
                path_value
            );
            std::fs::create_dir_all(path_value).map_err(|err| {
                PipelineError::message(&format!(
                    "Failed to create storage path '{}': {}",
                    path_value, err
                ))
            })?;
        }

        let service = context.service::<SettingService>()?;
        let mut locations = StorageSupport::load_locations(&service).await?;

        if locations
            .iter()
            .any(|location| location.path.eq_ignore_ascii_case(path_value))
        {
            return Err(PipelineError::message("Storage path already registered"));
        }

        let mut is_default = payload.is_default.unwrap_or(false);
        if locations.is_empty() {
            is_default = true;
        }

        if is_default {
            for location in locations.iter_mut() {
                location.is_default = false;
            }
        }

        let new_location = StorageLocation {
            id: Uuid::new_v4().to_string(),
            label: label_value.to_string(),
            path: path_value.to_string(),
            is_default,
            created_at: Utc::now().to_rfc3339(),
            category_template: payload
                .category_template
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("{year}/{date:%Y-%m-%d}/{fileName}")
                .to_string(),
        };

        locations.push(new_location.clone());
        StorageSupport::save_locations(&service, &locations).await?;

        let disk = StorageSupport::match_disk(&new_location.path, &StorageSupport::list_disks());

        Ok(ResponseValue::json(StorageLocationResponse {
            id: new_location.id,
            label: new_location.label,
            path: new_location.path,
            is_default: new_location.is_default,
            created_at: new_location.created_at,
            category_template: new_location.category_template,
            disk,
        }))
    }
}

struct UpdateStorageHandler;

#[async_trait]
#[put("/api/storage/locations/{id}", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for UpdateStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;

        let payload = context
            .read_json::<UpdateStoragePayload>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let service = context.service::<SettingService>()?;
        let mut locations = StorageSupport::load_locations(&service).await?;
        let index = locations
            .iter()
            .position(|location| location.id == *id)
            .ok_or_else(|| PipelineError::message("Storage location not found"))?;

        let current_id = locations[index].id.clone();

        if let Some(label) = &payload.label {
            let label_value = label.trim();
            if label_value.is_empty() {
                return Err(PipelineError::message("Storage label is required"));
            }
        }

        if let Some(path) = &payload.path {
            let path_value = path.trim();
            if path_value.is_empty() {
                return Err(PipelineError::message("Storage path is required"));
            }
            if !Path::new(path_value).exists() {
                return Err(PipelineError::message("Storage path does not exist"));
            }
            if locations
                .iter()
                .any(|entry| entry.id != current_id && entry.path.eq_ignore_ascii_case(path_value))
            {
                return Err(PipelineError::message("Storage path already registered"));
            }
        }

        {
            let location = &mut locations[index];

            if let Some(label) = &payload.label {
                location.label = label.trim().to_string();
            }

            if let Some(path) = &payload.path {
                location.path = path.trim().to_string();
            }

            if let Some(is_default) = payload.is_default {
                location.is_default = is_default;
            }

            if let Some(category_template) = &payload.category_template {
                let value = category_template.trim();
                if !value.is_empty() {
                    location.category_template = value.to_string();
                }
            }
        }

        if locations.iter().any(|location| location.is_default) {
            if let Some(default_id) = locations
                .iter()
                .find(|location| location.is_default)
                .map(|location| location.id.clone())
            {
                for location in locations.iter_mut() {
                    location.is_default = location.id == default_id;
                }
            }
        } else if let Some(first) = locations.first_mut() {
            first.is_default = true;
        }

        StorageSupport::save_locations(&service, &locations).await?;
        let disks = StorageSupport::list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = StorageSupport::match_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id,
                    label: location.label,
                    path: location.path,
                    is_default: location.is_default,
                    created_at: location.created_at,
                    category_template: location.category_template,
                    disk,
                }
            })
            .collect::<Vec<_>>();

        Ok(ResponseValue::json(response))
    }
}

struct DefaultStorageHandler;

#[async_trait]
#[put("/api/storage/locations/{id}/default", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for DefaultStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;

        let service = context.service::<SettingService>()?;
        let mut locations = StorageSupport::load_locations(&service).await?;
        let mut found = false;

        for location in locations.iter_mut() {
            if location.id == *id {
                location.is_default = true;
                found = true;
            } else {
                location.is_default = false;
            }
        }

        if !found {
            return Err(PipelineError::message("Storage location not found"));
        }

        StorageSupport::save_locations(&service, &locations).await?;
        let disks = StorageSupport::list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = StorageSupport::match_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id,
                    label: location.label,
                    path: location.path,
                    is_default: location.is_default,
                    created_at: location.created_at,
                    category_template: location.category_template,
                    disk,
                }
            })
            .collect::<Vec<_>>();

        Ok(ResponseValue::json(response))
    }
}

struct DeleteStorageHandler;

#[async_trait]
#[delete("/api/storage/locations/{id}", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for DeleteStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;

        let service = context.service::<SettingService>()?;
        let mut locations = StorageSupport::load_locations(&service).await?;
        let original_len = locations.len();
        locations.retain(|location| location.id != *id);

        if locations.len() == original_len {
            return Err(PipelineError::message("Storage location not found"));
        }

        if !locations.iter().any(|location| location.is_default) {
            if let Some(first) = locations.first_mut() {
                first.is_default = true;
            }
        }

        StorageSupport::save_locations(&service, &locations).await?;
        let disks = StorageSupport::list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = StorageSupport::match_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id,
                    label: location.label,
                    path: location.path,
                    is_default: location.is_default,
                    created_at: location.created_at,
                    category_template: location.category_template,
                    disk,
                }
            })
            .collect::<Vec<_>>();

        Ok(ResponseValue::json(response))
    }
}

struct BrowseStorageHandler;

#[async_trait]
#[get("/api/storage/{storageId}/browse")]
impl HttpHandler for BrowseStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        // if let Err(err) = context.require(Permission::BrowsePhotos) {
        //     context.response_mut().set_status(403);
        //     return Err(err);
        // }

        let storage_id = StorageSupport::route_storage_id(context)?;
        let request = StorageSupport::parse_browse_request(context)?;
        let path_segments = request.path_segments().map_err(|_| {
            context.response_mut().set_status(400);
            PipelineError::message("invalid browse path")
        })?;
        log::info!("Path segments: {:?}", path_segments);

        let setting_service = context.service::<SettingService>()?;
        let storage_locations = StorageSupport::load_locations(&setting_service).await?;
        let storage = storage_locations
            .into_iter()
            .find(|location| location.id == storage_id)
            .ok_or_else(|| {
                context.response_mut().set_status(404);
                PipelineError::message("storage not found")
            })?;

        let client_id = StorageSupport::current_client_id(context)?;
        let browse_options =
            StorageSupport::load_client_storage_settings(context, client_id, &storage.id).await?;
        log::info!(
            "Browsing storage '{}' with path '{}', options: {:?}, page size: {}, cursor: {:?}",
            storage.id,
            request.path.as_deref().unwrap_or(""),
            browse_options,
            request.page_size.unwrap_or(50),
            request.cursor.as_deref().unwrap_or("")
        );

        let cursor = match request.cursor.as_deref() {
            Some(raw) if !raw.trim().is_empty() => Some(
                PhotoCursor::decode(raw).map_err(|_| PipelineError::message("invalid cursor"))?,
            ),
            _ => None,
        };

        let page_size = request.page_size.unwrap_or(50);
        let browse_service = context.service::<BrowseService>()?;
        let response: BrowseResponse = browse_service
            .browse(
                &storage.id,
                &path_segments,
                &browse_options,
                page_size,
                cursor,
            )
            .await
            .map_err(|err| {
                let message = err.to_string();
                if message.contains("invalid browse path depth")
                    || message.contains("invalid digit found in string")
                    || message.contains("input contains invalid characters")
                    || message.contains("trailing input")
                    || message.contains("input is out of range")
                {
                    context.response_mut().set_status(400);
                    return PipelineError::message("invalid browse path");
                }
                PipelineError::message(&message)
            })?;

        Ok(ResponseValue::json(response))
    }
}
