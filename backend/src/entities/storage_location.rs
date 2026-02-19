use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::PostgresEntity,
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::FromRow,
};

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLocation {
    pub id: Uuid,
    pub label: String,
    pub path: String,
    pub is_default: bool,
    pub created_at: String,
    #[serde(default = "StorageLocation::default_category_template")]
    pub category_template: String,
}

impl StorageLocation {
    pub fn normalized_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.path);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    }

    fn default_category_template() -> String {
        "{year}/{date:%Y-%m-%d}/{fileName}".to_string()
    }
}

impl Entity for StorageLocation {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "storage"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for StorageLocation {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "id",
            "label",
            "path",
            "is_default",
            "created_at",
            "category_template",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::String(self.label.clone()),
            Value::String(self.path.clone()),
            Value::Bool(self.is_default),
            Value::String(self.created_at.clone()),
            Value::String(self.category_template.clone()),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "label",
            "path",
            "is_default",
            "created_at",
            "category_template",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.label.clone()),
            Value::String(self.path.clone()),
            Value::Bool(self.is_default),
            Value::String(self.created_at.clone()),
            Value::String(self.category_template.clone()),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("label", ColumnType::Text).not_null(),
            ColumnDef::new("path", ColumnType::Text).not_null(),
            ColumnDef::new("is_default", ColumnType::Boolean)
                .not_null()
                .default("false"),
            ColumnDef::new("created_at", ColumnType::Text).not_null(),
            ColumnDef::new("category_template", ColumnType::Text).not_null(),
        ]
    }
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClientStorageSettingsPayload {
    pub storage_ids: Vec<String>,
}
