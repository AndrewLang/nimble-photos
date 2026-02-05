use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use uuid::Uuid;

use crate::services::SettingService;

use nimble_web::controller::controller::Controller;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use sysinfo::Disks;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLocation {
    pub id: String,
    pub label: String,
    pub path: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLocationResponse {
    pub id: String,
    pub label: String,
    pub path: String,
    pub is_default: bool,
    pub created_at: String,
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
}

pub struct StorageController;

impl Controller for StorageController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get("/api/storage/disks", DisksHandler)
                .with_policy(Policy::InRole("admin".to_string()))
                .build(),
            EndpointRoute::get("/api/storage/locations", ListStorageHandler)
                .with_policy(Policy::InRole("admin".to_string()))
                .build(),
            EndpointRoute::post("/api/storage/locations", CreateStorageHandler)
                .with_policy(Policy::InRole("admin".to_string()))
                .build(),
            EndpointRoute::put("/api/storage/locations/{id}/default", DefaultStorageHandler)
                .with_policy(Policy::InRole("admin".to_string()))
                .build(),
        ]
    }
}

struct DisksHandler;

#[async_trait]
impl HttpHandler for DisksHandler {
    async fn invoke(&self, _context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        Ok(ResponseValue::json(list_disks()))
    }
}

struct ListStorageHandler;

#[async_trait]
impl HttpHandler for ListStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let service = context.service::<SettingService>()?;
        let locations = load_locations(&service).await?;
        let disks = list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = match_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id,
                    label: location.label,
                    path: location.path,
                    is_default: location.is_default,
                    created_at: location.created_at,
                    disk,
                }
            })
            .collect::<Vec<_>>();

        Ok(ResponseValue::json(response))
    }
}

struct CreateStorageHandler;

#[async_trait]
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
        let mut locations = load_locations(&service).await?;

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
        };

        locations.push(new_location.clone());
        save_locations(&service, &locations).await?;

        let disk = match_disk(&new_location.path, &list_disks());

        Ok(ResponseValue::json(StorageLocationResponse {
            id: new_location.id,
            label: new_location.label,
            path: new_location.path,
            is_default: new_location.is_default,
            created_at: new_location.created_at,
            disk,
        }))
    }
}

struct DefaultStorageHandler;

#[async_trait]
impl HttpHandler for DefaultStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let id = context
            .route()
            .and_then(|route| route.params().get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;

        let service = context.service::<SettingService>()?;
        let mut locations = load_locations(&service).await?;
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

        save_locations(&service, &locations).await?;
        let disks = list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = match_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id,
                    label: location.label,
                    path: location.path,
                    is_default: location.is_default,
                    created_at: location.created_at,
                    disk,
                }
            })
            .collect::<Vec<_>>();

        Ok(ResponseValue::json(response))
    }
}

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

    items.sort_by_key(|disk| disk_sort_key(&disk.mount_point));
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

async fn load_locations(service: &SettingService) -> Result<Vec<StorageLocation>, PipelineError> {
    let setting = service.get("storage.locations").await?;
    let value = setting.value;
    serde_json::from_value(value).map_err(|_| PipelineError::message("Invalid storage settings"))
}

async fn save_locations(
    service: &SettingService,
    locations: &[StorageLocation],
) -> Result<(), PipelineError> {
    let value = json!(locations);
    service.update("storage.locations", value).await?;
    Ok(())
}
