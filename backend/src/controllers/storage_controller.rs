use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

use crate::controllers::httpcontext_extensions::HttpContextExtensions;
use crate::entities::StorageLocation;
use crate::entities::client_storage::ClientStorage;
use crate::entities::photo_browse::{BrowseOptions, BrowseResponse};
use crate::entities::photo_cursor::PhotoCursor;
use crate::entities::storage_location::{
    CreateStoragePayload, StorageLocationResponse, UpdateClientStorageSettingsPayload,
    UpdateStoragePayload,
};
use crate::repositories::storage_repo::{
    ClientStorageRepositoryExtensions, StorageRepositoryExtensions,
};
use crate::repositories::validation::StringValidations;
use crate::services::BrowseService;

use nimble_web::DataProvider;
use nimble_web::controller::controller::Controller;
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::into_response::ResponseValue;
use nimble_web::security::policy::Policy;
use nimble_web::{delete, get, post, put};

pub struct StorageHandlers;

impl Controller for StorageHandlers {
    fn routes() -> Vec<EndpointRoute> {
        vec![]
    }
}

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
        let locations = repo.load_locations().await?;
        let disks = repo.list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = repo.find_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id.to_string(),
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

        let label_value = payload.label.trim().should_not_empty("Storage label")?;
        let path_value = payload.path.trim().should_not_empty("Storage path")?;

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

        let repository = context.service::<Repository<StorageLocation>>()?;
        if repository.exists_by_path(path_value).await? {
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
            if let Some(existing) = repository.find_location_by_path(&path_value).await? {
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
                    .load_locations()
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

        let locations = repository.load_locations().await?;
        let disks = repository.list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = repository.find_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id.to_string(),
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

        let locations = storage_repo.load_locations().await?;
        let disks = storage_repo.list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = storage_repo.find_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id.to_string(),
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

        let mut locations = repository.load_locations().await?;
        if !locations.iter().any(|location| location.is_default) {
            if let Some(mut first) = locations.first().cloned() {
                first.is_default = true;
                repository
                    .update(first)
                    .await
                    .map_err(|_| PipelineError::message("failed to save storage settings"))?;
                locations = repository.load_locations().await?;
            }
        }
        let disks = repository.list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = repository.find_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id.to_string(),
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

        let storage_id = context.route_storage_id()?;
        let request = context.parse_browse_request()?;
        let path_segments = request.path_segments().map_err(|_| {
            context.response_mut().set_status(400);
            PipelineError::message("invalid browse path")
        })?;

        let repository = context.service::<Repository<StorageLocation>>()?;
        let storage = repository
            .get(&storage_id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .ok_or_else(|| {
                context.response_mut().set_status(404);
                PipelineError::message("storage not found")
            })?;

        let client_id = context.current_client_id()?;
        let browse_options = context
            .load_client_storage_settings(client_id, storage.id)
            .await?;
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

struct ListStorageForClientController;

#[async_trait]
#[get("/api/storage/list")]
impl HttpHandler for ListStorageForClientController {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let api_key = context.extract_api_key()?;
        let _ = context.validate_api_key(&api_key).await?;

        let repository = context.service::<Repository<StorageLocation>>()?;
        let locations = repository.load_locations().await?;
        let disks = repository.list_disks();

        let response = locations
            .into_iter()
            .map(|location| {
                let disk = repository.find_disk(&location.path, &disks);
                StorageLocationResponse {
                    id: location.id.to_string(),
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

struct UpdateClientStorageSettingsHandler;

#[async_trait]
#[post("/api/client/storage/settings")]
impl HttpHandler for UpdateClientStorageSettingsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let api_key = context.extract_api_key()?;
        let client = context.validate_api_key(&api_key).await?;

        let payload = context
            .read_json::<UpdateClientStorageSettingsPayload>()
            .map_err(|err| PipelineError::message(err.message()))?;
        if payload.storage_ids.is_empty() {
            return Err(PipelineError::message("storageIds is required"));
        }
        log::info!(
            "Updating storage settings for client '{}', storageIds: {:?}",
            client.id,
            payload.storage_ids
        );

        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let storages = storage_repo.load_locations().await?;
        let configured_storage_ids = storages
            .iter()
            .map(|location| location.id.clone())
            .collect::<HashSet<_>>();

        let mut requested_storage_ids = HashSet::<Uuid>::new();
        for raw_storage_id in payload.storage_ids {
            let storage_id = Uuid::parse_str(&raw_storage_id)
                .map_err(|_| PipelineError::message("invalid storageId"))?;
            if !configured_storage_ids.contains(&storage_id) {
                context.response_mut().set_status(404);
                return Err(PipelineError::message("storage not found"));
            }
            requested_storage_ids.insert(storage_id);
        }

        let repository = context.service::<Repository<ClientStorage>>()?;
        let existing_rows = repository.for_client(client.id).await?;
        let existing_storage_ids = existing_rows
            .iter()
            .map(|item| item.storage_id)
            .collect::<HashSet<_>>();

        for storage_id in &requested_storage_ids {
            if existing_storage_ids.contains(storage_id) {
                continue;
            }

            let record = ClientStorage {
                id: Uuid::new_v4(),
                client_id: client.id,
                storage_id: *storage_id,
                browse_options: BrowseOptions::default(),
            };
            log::info!("Record to insert {:?}", record);

            match repository.insert(record).await {
                Ok(_) => {}
                Err(err) => {
                    let message = format!("{:?}", err).to_ascii_lowercase();
                    if message.contains("duplicate key")
                        || message.contains("unique constraint")
                        || message.contains("already exists")
                    {
                        continue;
                    }
                    return Err(PipelineError::message(
                        "failed to create client storage settings",
                    ));
                }
            }
        }

        let client_storage_ids = existing_storage_ids
            .union(&requested_storage_ids)
            .copied()
            .collect::<HashSet<Uuid>>();
        let client_locations = storages
            .into_iter()
            .filter(|location| client_storage_ids.contains(&location.id))
            .collect::<Vec<_>>();

        Ok(ResponseValue::json(client_locations))
    }
}
