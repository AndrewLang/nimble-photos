use crate::prelude::*;

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
    pub year: Option<i32>,
    #[serde(alias = "month_day")]
    pub month_day: Option<String>,
    #[serde(alias = "metadata_extracted")]
    pub metadata_extracted: Option<bool>,
    pub artist: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    #[serde(alias = "lens_make")]
    pub lens_make: Option<String>,
    #[serde(alias = "lens_model")]
    pub lens_model: Option<String>,
    #[serde(alias = "exposure_time")]
    pub exposure_time: Option<String>,
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    #[serde(alias = "focal_length")]
    pub focal_length: Option<f32>,
    pub label: Option<String>,
    pub rating: Option<u8>,
    pub flagged: Option<i8>,
    #[serde(alias = "is_raw")]
    pub is_raw: Option<bool>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<u16>,
    #[serde(alias = "day_date")]
    pub day_date: NaiveDate,
    #[serde(alias = "sort_date")]
    pub sort_date: DateTime<Utc>,
}

impl Default for Photo {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            storage_id: Uuid::nil(),
            path: String::new(),
            name: String::new(),
            format: None,
            hash: None,
            size: None,
            created_at: Some(now),
            updated_at: Some(now),
            date_imported: Some(now),
            date_taken: None,
            year: None,
            month_day: None,
            metadata_extracted: Some(false),
            artist: None,
            make: None,
            model: None,
            lens_make: None,
            lens_model: None,
            exposure_time: None,
            iso: None,
            aperture: None,
            focal_length: None,
            label: None,
            rating: None,
            flagged: None,
            is_raw: None,
            width: None,
            height: None,
            orientation: None,
            day_date: now.date_naive(),
            sort_date: now,
        }
    }
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
            year: PostgresExtensions::optional_i32_as_i32(row, "year")?,
            month_day: row.try_get("month_day")?,
            metadata_extracted: row.try_get("metadata_extracted")?,
            artist: row.try_get("artist")?,
            make: row.try_get("make")?,
            model: row.try_get("model")?,
            lens_make: row.try_get("lens_make")?,
            lens_model: row.try_get("lens_model")?,
            exposure_time: row.try_get("exposure_time")?,
            iso: PostgresExtensions::optional_i32_as_u32(row, "iso")?,
            aperture: row.try_get("aperture")?,
            focal_length: row.try_get("focal_length")?,
            label: row.try_get("label")?,
            rating: PostgresExtensions::optional_i32_as_u8(row, "rating")?,
            flagged: PostgresExtensions::optional_i32_as_i8(row, "flagged")?,
            is_raw: row.try_get("is_raw")?,
            width: PostgresExtensions::optional_i32_as_u32(row, "width")?,
            height: PostgresExtensions::optional_i32_as_u32(row, "height")?,
            orientation: PostgresExtensions::optional_i32_as_u16(row, "orientation")?,
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
            "year",
            "month_day",
            "metadata_extracted",
            "artist",
            "make",
            "model",
            "lens_make",
            "lens_model",
            "exposure_time",
            "iso",
            "aperture",
            "focal_length",
            "label",
            "rating",
            "flagged",
            "is_raw",
            "width",
            "height",
            "orientation",
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
            PostgresValueBuilder::optional_i32(self.year),
            PostgresValueBuilder::optional_string(&self.month_day),
            PostgresValueBuilder::optional_bool(self.metadata_extracted),
            PostgresValueBuilder::optional_string(&self.artist),
            PostgresValueBuilder::optional_string(&self.make),
            PostgresValueBuilder::optional_string(&self.model),
            PostgresValueBuilder::optional_string(&self.lens_make),
            PostgresValueBuilder::optional_string(&self.lens_model),
            PostgresValueBuilder::optional_string(&self.exposure_time),
            PostgresValueBuilder::optional_u32(self.iso),
            PostgresValueBuilder::optional_f32(self.aperture),
            PostgresValueBuilder::optional_f32(self.focal_length),
            PostgresValueBuilder::optional_string(&self.label),
            PostgresValueBuilder::optional_u8(self.rating),
            PostgresValueBuilder::optional_i8(self.flagged),
            PostgresValueBuilder::optional_bool(self.is_raw),
            PostgresValueBuilder::optional_u32(self.width),
            PostgresValueBuilder::optional_u32(self.height),
            PostgresValueBuilder::optional_u16(self.orientation),
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
            "year",
            "month_day",
            "metadata_extracted",
            "artist",
            "make",
            "model",
            "lens_make",
            "lens_model",
            "exposure_time",
            "iso",
            "aperture",
            "focal_length",
            "label",
            "rating",
            "flagged",
            "is_raw",
            "width",
            "height",
            "orientation",
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
            PostgresValueBuilder::optional_i32(self.year),
            PostgresValueBuilder::optional_string(&self.month_day),
            PostgresValueBuilder::optional_bool(self.metadata_extracted),
            PostgresValueBuilder::optional_string(&self.artist),
            PostgresValueBuilder::optional_string(&self.make),
            PostgresValueBuilder::optional_string(&self.model),
            PostgresValueBuilder::optional_string(&self.lens_make),
            PostgresValueBuilder::optional_string(&self.lens_model),
            PostgresValueBuilder::optional_string(&self.exposure_time),
            PostgresValueBuilder::optional_u32(self.iso),
            PostgresValueBuilder::optional_f32(self.aperture),
            PostgresValueBuilder::optional_f32(self.focal_length),
            PostgresValueBuilder::optional_string(&self.label),
            PostgresValueBuilder::optional_u8(self.rating),
            PostgresValueBuilder::optional_i8(self.flagged),
            PostgresValueBuilder::optional_bool(self.is_raw),
            PostgresValueBuilder::optional_u32(self.width),
            PostgresValueBuilder::optional_u32(self.height),
            PostgresValueBuilder::optional_u16(self.orientation),
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
            ColumnDef::new("year", ColumnType::Integer),
            ColumnDef::new("month_day", ColumnType::Text),
            ColumnDef::new("metadata_extracted", ColumnType::Boolean),
            ColumnDef::new("artist", ColumnType::Text),
            ColumnDef::new("make", ColumnType::Text),
            ColumnDef::new("model", ColumnType::Text),
            ColumnDef::new("lens_make", ColumnType::Text),
            ColumnDef::new("lens_model", ColumnType::Text),
            ColumnDef::new("exposure_time", ColumnType::Text),
            ColumnDef::new("iso", ColumnType::Integer),
            ColumnDef::new("aperture", ColumnType::Float),
            ColumnDef::new("focal_length", ColumnType::Float),
            ColumnDef::new("label", ColumnType::Text),
            ColumnDef::new("rating", ColumnType::Integer),
            ColumnDef::new("flagged", ColumnType::Integer),
            ColumnDef::new("is_raw", ColumnType::Boolean),
            ColumnDef::new("width", ColumnType::Integer),
            ColumnDef::new("height", ColumnType::Integer),
            ColumnDef::new("orientation", ColumnType::Integer),
            ColumnDef::new("day_date", ColumnType::Custom("DATE")).not_null(),
            ColumnDef::new("sort_date", ColumnType::Timestamp).not_null(),
        ]
    }
}
