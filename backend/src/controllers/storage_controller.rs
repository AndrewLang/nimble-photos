use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::entities::StorageLocation;
use crate::entities::storage_location::{
    CreateStoragePayload, StorageLocationResponse, UpdateStoragePayload,
};
use crate::repositories::storage_repo::StorageRepositoryExtensions;
use crate::repositories::validation::StringValidations;

use nimble_web::DataProvider;
use nimble_web::HttpContext;
use nimble_web::HttpHandler;
use nimble_web::PipelineError;
use nimble_web::Policy;
use nimble_web::Repository;
use nimble_web::ResponseValue;
use nimble_web::{delete, get, post, put};

struct DisksHandler;

#[async_trait]
#[get("/api/storage/disks", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for DisksHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        Ok(ResponseValue::json(storage_repo.list_disks()))
    }
}

struct ListStorageHandler;

#[async_trait]
#[get("/api/storage/locations", policy = Policy::InRole("admin".to_string()))]
impl HttpHandler for ListStorageHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repo = context.service::<Repository<StorageLocation>>()?;
        let locations = repo.load_storages().await?;

        let response = repo
            .to_storage_responses(locations)
            .map_err(|_| PipelineError::message("failed to load storage settings"))?;

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

        let label_value = payload.label.trim().should_not_empty("Storage label")?;
        let mount_point = payload.mount_point.trim().should_not_empty("Mount point")?;
        let path_value = payload.path.trim().should_not_empty("Storage path")?;
        let full_path = Path::new(mount_point).join(path_value);
        let full_path_value = full_path.to_string_lossy().to_string();

        if !full_path.exists() {
            log::warn!(
                "Storage path does not exist: {}, will create it.",
                path_value
            );

            fs::create_dir_all(&full_path).map_err(|err| {
                PipelineError::message(&format!(
                    "Failed to create storage path '{}': {}",
                    full_path.display(),
                    err
                ))
            })?;
        }

        let repository = context.service::<Repository<StorageLocation>>()?;
        if repository.exists_by_path(&full_path_value).await? {
            return Err(PipelineError::message("Storage path already registered"));
        }

        let mut is_default = payload.is_default.unwrap_or(false);
        if repository.is_empty().await? {
            is_default = true;
        }

        if is_default {
            repository.reset_default().await?;
        }

        let new_location = StorageLocation {
            id: Uuid::new_v4(),
            label: label_value.to_string(),
            path: full_path_value,
            is_default,
            readonly: false,
            created_at: Utc::now().to_rfc3339(),
            category_template: payload
                .category_template
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("{year}/{date:%Y-%m-%d}/{fileName}")
                .to_string(),
        };

        repository
            .insert(new_location.clone())
            .await
            .map_err(|_| PipelineError::message("failed to save storage settings"))?;

        let disk = repository.find_disk(&new_location.path, &repository.list_disks());

        Ok(ResponseValue::json(StorageLocationResponse {
            id: new_location.id.to_string(),
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
            .ok_or_else(|| PipelineError::message("id parameter missing"))
            .and_then(|value| {
                Uuid::parse_str(value).map_err(|_| PipelineError::message("invalid id parameter"))
            })?;

        let payload = context
            .read_json::<UpdateStoragePayload>()
            .map_err(|err| PipelineError::message(err.message()))?;

        let repository = context.service::<Repository<StorageLocation>>()?;
        let mut location = repository
            .get(&id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| PipelineError::message("Storage location not found"))?;

        if let Some(label) = &payload.label {
            let label_value = label.trim().should_not_empty("Storage label");
            location.label = label_value?.to_string();
        }

        if let Some(path) = &payload.path {
            let path_value = path.trim().should_not_empty("Storage path")?.to_string();
            if !Path::new(&path_value).exists() {
                return Err(PipelineError::message("Storage path does not exist"));
            }
            if let Some(existing) = repository.find_storage_by_path(&path_value).await? {
                if existing.id != location.id {
                    return Err(PipelineError::message("Storage path already registered"));
                }
            }

            location.path = path_value;
        }

        if let Some(category_template) = &payload.category_template {
            let value = category_template
                .trim()
                .should_not_empty("Category template");
            location.category_template = value?.to_string();
        }

        if let Some(is_default) = payload.is_default {
            if is_default {
                repository.reset_default().await?;
                location.is_default = true;
            } else if location.is_default {
                location.is_default = false;
                if let Some(mut replacement) = repository
                    .load_storages()
                    .await?
                    .into_iter()
                    .find(|entry| entry.id != location.id)
                {
                    replacement.is_default = true;
                    repository
                        .update(replacement)
                        .await
                        .map_err(|_| PipelineError::message("failed to save storage settings"))?;
                } else {
                    location.is_default = true;
                }
            } else {
                location.is_default = false;
            }
        }

        repository
            .update(location)
            .await
            .map_err(|_| PipelineError::message("failed to save storage settings"))?;

        let locations = repository.load_storages().await?;
        let response = repository
            .to_storage_responses(locations)
            .map_err(|_| PipelineError::message("failed to load storage settings"))?;

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
            .ok_or_else(|| PipelineError::message("id parameter missing"))
            .and_then(|value| {
                Uuid::parse_str(value).map_err(|_| PipelineError::message("invalid id parameter"))
            })?;

        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let mut location = storage_repo
            .get(&id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| PipelineError::message("Storage location not found"))?;

        storage_repo.reset_default().await?;
        location.is_default = true;
        storage_repo
            .update(location)
            .await
            .map_err(|_| PipelineError::message("failed to save storage settings"))?;

        let locations = storage_repo.load_storages().await?;
        let response = storage_repo
            .to_storage_responses(locations)
            .map_err(|_| PipelineError::message("failed to load storage settings"))?;

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
            .ok_or_else(|| PipelineError::message("id parameter missing"))
            .and_then(|value| {
                Uuid::parse_str(value).map_err(|_| PipelineError::message("invalid id parameter"))
            })?;

        let repository = context.service::<Repository<StorageLocation>>()?;
        let deleted_location = repository
            .get(&id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?;
        if deleted_location.is_none() {
            return Err(PipelineError::message("Storage location not found"));
        }

        repository
            .delete(&id)
            .await
            .map_err(|_| PipelineError::message("failed to save storage settings"))?;

        let mut locations = repository.load_storages().await?;
        if !locations.iter().any(|location| location.is_default) {
            if let Some(mut first) = locations.first().cloned() {
                first.is_default = true;
                repository
                    .update(first)
                    .await
                    .map_err(|_| PipelineError::message("failed to save storage settings"))?;
                locations = repository.load_storages().await?;
            }
        }
        let response = repository
            .to_storage_responses(locations)
            .map_err(|_| PipelineError::message("failed to load storage settings"))?;

        Ok(ResponseValue::json(response))
    }
}
