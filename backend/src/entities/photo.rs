use chrono::{DateTime, NaiveDate, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use crate::repositories::postgres_extensions::PostgresExtensions;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::{PostgresEntity, value_builder::PostgresValueBuilder},
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::postgres::PgRow,
    sqlx::{FromRow, Row},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoViewModel {
    pub id: Uuid,
    pub hash: String,
    pub name: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    pub id: Uuid,
    #[serde(alias = "storage_id")]
    pub storage_id: Uuid,
    pub path: String,
    pub name: String,
    pub format: Option<String>,
    pub hash: Option<String>,
    pub size: Option<i64>,
    #[serde(alias = "created_at")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(alias = "updated_at")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(alias = "date_imported")]
    pub date_imported: Option<DateTime<Utc>>,
    #[serde(alias = "date_taken")]
    pub date_taken: Option<DateTime<Utc>>,
    #[serde(alias = "metadata_extracted")]
    pub metadata_extracted: Option<bool>,
    #[serde(alias = "is_raw")]
    pub is_raw: Option<bool>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(alias = "day_date")]
    pub day_date: NaiveDate,
    #[serde(alias = "sort_date")]
    pub sort_date: DateTime<Utc>,
}

impl Entity for Photo {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "photo"
    }
}

#[cfg(feature = "postgres")]
impl<'r> FromRow<'r, PgRow> for Photo {
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            storage_id: row.try_get("storage_id")?,
            path: row.try_get("path")?,
            name: row.try_get("name")?,
            format: row.try_get("format")?,
            hash: row.try_get("hash")?,
            size: row.try_get("size")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            date_imported: row.try_get("date_imported")?,
            date_taken: row.try_get("date_taken")?,
            metadata_extracted: row.try_get("metadata_extracted")?,
            is_raw: row.try_get("is_raw")?,
            width: PostgresExtensions::optional_i32_as_u32(row, "width")?,
            height: PostgresExtensions::optional_i32_as_u32(row, "height")?,
            day_date: row.try_get("day_date")?,
            sort_date: row.try_get("sort_date")?,
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
            "storage_id",
            "path",
            "name",
            "format",
            "hash",
            "size",
            "created_at",
            "updated_at",
            "date_imported",
            "date_taken",
            "metadata_extracted",
            "is_raw",
            "width",
            "height",
            "day_date",
            "sort_date",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::Uuid(self.storage_id),
            Value::String(self.path.clone()),
            Value::String(self.name.clone()),
            PostgresValueBuilder::optional_string(&self.format),
            PostgresValueBuilder::optional_string(&self.hash),
            PostgresValueBuilder::optional_i64(self.size),
            PostgresValueBuilder::optional_datetime(&self.created_at),
            PostgresValueBuilder::optional_datetime(&self.updated_at),
            PostgresValueBuilder::optional_datetime(&self.date_imported),
            PostgresValueBuilder::optional_datetime(&self.date_taken),
            PostgresValueBuilder::optional_bool(self.metadata_extracted),
            PostgresValueBuilder::optional_bool(self.is_raw),
            PostgresValueBuilder::optional_u32(self.width),
            PostgresValueBuilder::optional_u32(self.height),
            Value::Date(self.day_date),
            Value::DateTime(self.sort_date.clone()),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "storage_id",
            "path",
            "name",
            "format",
            "hash",
            "size",
            "created_at",
            "updated_at",
            "date_imported",
            "date_taken",
            "metadata_extracted",
            "is_raw",
            "width",
            "height",
            "day_date",
            "sort_date",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.storage_id),
            Value::String(self.path.clone()),
            Value::String(self.name.clone()),
            PostgresValueBuilder::optional_string(&self.format),
            PostgresValueBuilder::optional_string(&self.hash),
            PostgresValueBuilder::optional_i64(self.size),
            PostgresValueBuilder::optional_datetime(&self.created_at),
            PostgresValueBuilder::optional_datetime(&self.updated_at),
            PostgresValueBuilder::optional_datetime(&self.date_imported),
            PostgresValueBuilder::optional_datetime(&self.date_taken),
            PostgresValueBuilder::optional_bool(self.metadata_extracted),
            PostgresValueBuilder::optional_bool(self.is_raw),
            PostgresValueBuilder::optional_u32(self.width),
            PostgresValueBuilder::optional_u32(self.height),
            Value::Date(self.day_date),
            Value::DateTime(self.sort_date.clone()),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("storage_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("path", ColumnType::Text).not_null(),
            ColumnDef::new("name", ColumnType::Text).not_null(),
            ColumnDef::new("format", ColumnType::Text),
            ColumnDef::new("hash", ColumnType::Text),
            ColumnDef::new("size", ColumnType::BigInt),
            ColumnDef::new("created_at", ColumnType::Timestamp),
            ColumnDef::new("updated_at", ColumnType::Timestamp),
            ColumnDef::new("date_imported", ColumnType::Timestamp),
            ColumnDef::new("date_taken", ColumnType::Timestamp),
            ColumnDef::new("metadata_extracted", ColumnType::Boolean),
            ColumnDef::new("is_raw", ColumnType::Boolean),
            ColumnDef::new("width", ColumnType::Integer),
            ColumnDef::new("height", ColumnType::Integer),
            ColumnDef::new("day_date", ColumnType::Custom("DATE")).not_null(),
            ColumnDef::new("sort_date", ColumnType::Timestamp).not_null(),
        ]
    }
}
