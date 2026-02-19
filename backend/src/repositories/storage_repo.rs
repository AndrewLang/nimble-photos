use async_trait::async_trait;
use sysinfo::Disks;
use uuid::Uuid;

use crate::entities::client_storage::ClientStorage;
use crate::entities::storage_location::{DiskInfo, StorageLocation};

use nimble_web::DataProvider;
use nimble_web::data::query::{Query, Value};
use nimble_web::data::repository::Repository;
use nimble_web::pipeline::pipeline::PipelineError;

#[async_trait]
pub trait StorageRepositoryExtensions {
    fn list_disks(&self) -> Vec<DiskInfo>;
    fn disk_sort_key(&self, mount_point: &str) -> (u8, String);
    fn find_disk(&self, path: &str, disks: &[DiskInfo]) -> Option<DiskInfo>;
    async fn load_locations(&self) -> Result<Vec<StorageLocation>, PipelineError>;
    async fn find_location_by_path(
        &self,
        path: &str,
    ) -> Result<Option<StorageLocation>, PipelineError>;
    async fn exists_by_path(&self, path: &str) -> Result<bool, PipelineError>;
    async fn exists_by_id(&self, id: Uuid) -> Result<bool, PipelineError>;
    async fn is_empty(&self) -> Result<bool, PipelineError>;
    async fn default_locations(&self) -> Result<Vec<StorageLocation>, PipelineError>;
    async fn reset_default(&self) -> Result<(), PipelineError>;
}

#[async_trait]
impl StorageRepositoryExtensions for Repository<StorageLocation> {
    fn list_disks(&self) -> Vec<DiskInfo> {
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

        items.sort_by_key(|disk| self.disk_sort_key(&disk.mount_point));
        items
    }

    fn disk_sort_key(&self, mount_point: &str) -> (u8, String) {
        let normalized = mount_point.trim().to_ascii_lowercase();
        let bytes = normalized.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            return (0, normalized);
        }
        (1, normalized)
    }

    fn find_disk(&self, path: &str, disks: &[DiskInfo]) -> Option<DiskInfo> {
        let path_lower = path.to_ascii_lowercase();
        disks
            .iter()
            .filter(|disk| !disk.mount_point.is_empty())
            .filter(|disk| path_lower.starts_with(&disk.mount_point.to_ascii_lowercase()))
            .max_by_key(|disk| disk.mount_point.len())
            .cloned()
    }

    async fn load_locations(&self) -> Result<Vec<StorageLocation>, PipelineError> {
        let locations = self
            .query(Query::<StorageLocation>::new())
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .items;
        Ok(locations)
    }

    async fn find_location_by_path(
        &self,
        path: &str,
    ) -> Result<Option<StorageLocation>, PipelineError> {
        self.get_by("path", Value::String(path.to_string()))
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))
    }

    async fn exists_by_path(&self, path: &str) -> Result<bool, PipelineError> {
        Ok(self.find_location_by_path(path).await?.is_some())
    }

    async fn exists_by_id(&self, id: Uuid) -> Result<bool, PipelineError> {
        Ok(self
            .get(&id)
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .is_some())
    }

    async fn is_empty(&self) -> Result<bool, PipelineError> {
        let count = self
            .query(Query::<StorageLocation>::new().with_page_size(1))
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .items
            .len();
        Ok(count == 0)
    }

    async fn default_locations(&self) -> Result<Vec<StorageLocation>, PipelineError> {
        let locations = self
            .query(
                Query::<StorageLocation>::new()
                    .with_filter("is_default", Value::Bool(true))
                    .with_page_size(100),
            )
            .await
            .map_err(|_| PipelineError::message("failed to load storage settings"))?
            .items;
        Ok(locations)
    }

    async fn reset_default(&self) -> Result<(), PipelineError> {
        let mut locations = self.default_locations().await?;
        for location in locations.iter_mut() {
            location.is_default = false;
            self.update(location.clone())
                .await
                .map_err(|_| PipelineError::message("failed to reset default storage settings"))?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait ClientStorageRepositoryExtensions {
    async fn for_client(&self, client_id: Uuid) -> Result<Vec<ClientStorage>, PipelineError>;
}

#[async_trait]
impl ClientStorageRepositoryExtensions for Repository<ClientStorage> {
    async fn for_client(&self, client_id: Uuid) -> Result<Vec<ClientStorage>, PipelineError> {
        let items = self
            .query(Query::<ClientStorage>::new().with_filter("client_id", Value::Uuid(client_id)))
            .await
            .map_err(|_| PipelineError::message("failed to load client storage settings"))?
            .items;
        Ok(items)
    }
}
