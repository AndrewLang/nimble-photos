use super::uuid_id::HasOptionalUuidId;
use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::{PostgresEntity, value_builder::PostgresValueBuilder},
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::postgres::PgRow,
    sqlx::{FromRow, Row},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub id: Option<Uuid>,
    pub path: String,
    pub name: String,
    pub format: Option<String>,
    pub hash: Option<String>,
    pub size: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub date_imported: Option<DateTime<Utc>>,
    pub date_taken: Option<DateTime<Utc>>,
    pub thumbnail_path: Option<String>,
    pub thumbnail_optimized: Option<bool>,
    pub metadata_extracted: Option<bool>,
    pub is_raw: Option<bool>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumbnail_width: Option<u32>,
    pub thumbnail_height: Option<u32>,
}

impl Entity for Photo {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        self.id
            .as_ref()
            .expect("Photo entity requires an id for Entity trait operations")
    }

    fn name() -> &'static str {
        "photo"
    }
}

impl HasOptionalUuidId for Photo {
    fn id_slot(&mut self) -> &mut Option<Uuid> {
        &mut self.id
    }
}

#[cfg(feature = "postgres")]
fn optional_i32_as_u32(row: &PgRow, column: &str) -> sqlx::Result<Option<u32>> {
    row.try_get::<Option<i32>, _>(column)
        .map(|opt| opt.map(|value| value as u32))
}

#[cfg(feature = "postgres")]
impl<'r> FromRow<'r, PgRow> for Photo {
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            path: row.try_get("path")?,
            name: row.try_get("name")?,
            format: row.try_get("format")?,
            hash: row.try_get("hash")?,
            size: row.try_get("size")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            date_imported: row.try_get("date_imported")?,
            date_taken: row.try_get("date_taken")?,
            thumbnail_path: row.try_get("thumbnail_path")?,
            thumbnail_optimized: row.try_get("thumbnail_optimized")?,
            metadata_extracted: row.try_get("metadata_extracted")?,
            is_raw: row.try_get("is_raw")?,
            width: optional_i32_as_u32(row, "width")?,
            height: optional_i32_as_u32(row, "height")?,
            thumbnail_width: optional_i32_as_u32(row, "thumbnail_width")?,
            thumbnail_height: optional_i32_as_u32(row, "thumbnail_height")?,
        })
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for Photo {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "id",
            "path",
            "name",
            "format",
            "hash",
            "size",
            "created_at",
            "updated_at",
            "date_imported",
            "date_taken",
            "thumbnail_path",
            "thumbnail_optimized",
            "metadata_extracted",
            "is_raw",
            "width",
            "height",
            "thumbnail_width",
            "thumbnail_height",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        let id = self.id.as_ref().expect("id not set for Photo insert");
        vec![
            Value::Uuid(*id),
            Value::String(self.path.clone()),
            Value::String(self.name.clone()),
            PostgresValueBuilder::optional_string(&self.format),
            PostgresValueBuilder::optional_string(&self.hash),
            PostgresValueBuilder::optional_i64(self.size),
            PostgresValueBuilder::optional_datetime(&self.created_at),
            PostgresValueBuilder::optional_datetime(&self.updated_at),
            PostgresValueBuilder::optional_datetime(&self.date_imported),
            PostgresValueBuilder::optional_datetime(&self.date_taken),
            PostgresValueBuilder::optional_string(&self.thumbnail_path),
            PostgresValueBuilder::optional_bool(self.thumbnail_optimized),
            PostgresValueBuilder::optional_bool(self.metadata_extracted),
            PostgresValueBuilder::optional_bool(self.is_raw),
            PostgresValueBuilder::optional_u32(self.width),
            PostgresValueBuilder::optional_u32(self.height),
            PostgresValueBuilder::optional_u32(self.thumbnail_width),
            PostgresValueBuilder::optional_u32(self.thumbnail_height),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "path",
            "name",
            "format",
            "hash",
            "size",
            "created_at",
            "updated_at",
            "date_imported",
            "date_taken",
            "thumbnail_path",
            "thumbnail_optimized",
            "metadata_extracted",
            "is_raw",
            "width",
            "height",
            "thumbnail_width",
            "thumbnail_height",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.path.clone()),
            Value::String(self.name.clone()),
            PostgresValueBuilder::optional_string(&self.format),
            PostgresValueBuilder::optional_string(&self.hash),
            PostgresValueBuilder::optional_i64(self.size),
            PostgresValueBuilder::optional_datetime(&self.created_at),
            PostgresValueBuilder::optional_datetime(&self.updated_at),
            PostgresValueBuilder::optional_datetime(&self.date_imported),
            PostgresValueBuilder::optional_datetime(&self.date_taken),
            PostgresValueBuilder::optional_string(&self.thumbnail_path),
            PostgresValueBuilder::optional_bool(self.thumbnail_optimized),
            PostgresValueBuilder::optional_bool(self.metadata_extracted),
            PostgresValueBuilder::optional_bool(self.is_raw),
            PostgresValueBuilder::optional_u32(self.width),
            PostgresValueBuilder::optional_u32(self.height),
            PostgresValueBuilder::optional_u32(self.thumbnail_width),
            PostgresValueBuilder::optional_u32(self.thumbnail_height),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("path", ColumnType::Text).not_null(),
            ColumnDef::new("name", ColumnType::Text).not_null(),
            ColumnDef::new("format", ColumnType::Text),
            ColumnDef::new("hash", ColumnType::Text),
            ColumnDef::new("size", ColumnType::BigInt),
            ColumnDef::new("created_at", ColumnType::Timestamp),
            ColumnDef::new("updated_at", ColumnType::Timestamp),
            ColumnDef::new("date_imported", ColumnType::Timestamp),
            ColumnDef::new("date_taken", ColumnType::Timestamp),
            ColumnDef::new("thumbnail_path", ColumnType::Text),
            ColumnDef::new("thumbnail_optimized", ColumnType::Boolean),
            ColumnDef::new("metadata_extracted", ColumnType::Boolean),
            ColumnDef::new("is_raw", ColumnType::Boolean),
            ColumnDef::new("width", ColumnType::Integer),
            ColumnDef::new("height", ColumnType::Integer),
            ColumnDef::new("thumbnail_width", ColumnType::Integer),
            ColumnDef::new("thumbnail_height", ColumnType::Integer),
        ]
    }
}
