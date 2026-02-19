use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::photo_browse::BrowseOptions;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::PostgresEntity,
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::postgres::PgRow,
    sqlx::{FromRow, Row},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientStorage {
    pub id: String,
    pub client_id: Uuid,
    pub storage_id: Uuid,
    #[serde(default)]
    pub browse_options: BrowseOptions,
}

impl ClientStorage {
    pub fn row_id(client_id: Uuid, storage_id: Uuid) -> String {
        format!("{client_id}:{storage_id}")
    }

    fn default_browse_options(&self) -> String {
        "{\"dimensions\":[\"Year\",\"Date\"],\"sortDirection\":\"Desc\",\"dateFormat\":\"yyyy-MM-dd\"}".to_string()
    }
}

impl Entity for ClientStorage {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "ClientStorage"
    }
}

#[cfg(feature = "postgres")]
impl<'r> FromRow<'r, PgRow> for ClientStorage {
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        let client_id: Uuid = row.try_get("client_id")?;
        let storage_id: Uuid = row.try_get("storage_id")?;
        let raw_options: Option<String> = row.try_get("browse_options")?;
        let browse_options = raw_options
            .as_deref()
            .and_then(|raw| serde_json::from_str::<BrowseOptions>(raw).ok())
            .unwrap_or_default();

        Ok(Self {
            id: row
                .try_get("id")
                .unwrap_or_else(|_| Self::row_id(client_id, storage_id)),
            client_id,
            storage_id,
            browse_options,
        })
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for ClientStorage {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::String(id.clone())
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "client_id", "storage_id", "browse_options"]
    }

    fn insert_values(&self) -> Vec<Value> {
        let browse_options = serde_json::to_string(&self.browse_options)
            .unwrap_or_else(|_| self.default_browse_options());
        vec![
            Value::String(self.id.clone()),
            Value::Uuid(self.client_id),
            Value::Uuid(self.storage_id),
            Value::String(browse_options),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["client_id", "storage_id", "browse_options"]
    }

    fn update_values(&self) -> Vec<Value> {
        let browse_options = serde_json::to_string(&self.browse_options)
            .unwrap_or_else(|_| self.default_browse_options());
        vec![
            Value::Uuid(self.client_id),
            Value::Uuid(self.storage_id),
            Value::String(browse_options),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Text).primary_key(),
            ColumnDef::new("client_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("storage_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("browse_options", ColumnType::Text).not_null(),
        ]
    }
}
