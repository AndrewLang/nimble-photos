use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::uuid_id::HasOptionalUuidId;
use nimble_web::Entity;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::{PostgresEntity, value_builder::PostgresValueBuilder},
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::postgres::PgRow,
    sqlx::{FromRow, Row},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AlbumComment {
    pub id: Option<Uuid>,
    pub album_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_display_name: Option<String>,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub hidden: bool,
}

impl Entity for AlbumComment {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        self.id
            .as_ref()
            .expect("AlbumComment requires an id for Entity trait operations")
    }

    fn name() -> &'static str {
        "album_comment"
    }
}

impl HasOptionalUuidId for AlbumComment {
    fn id_slot(&mut self) -> &mut Option<Uuid> {
        &mut self.id
    }
}

#[cfg(feature = "postgres")]
impl<'r> FromRow<'r, PgRow> for AlbumComment {
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            album_id: row.try_get("album_id")?,
            user_id: row.try_get("user_id")?,
            user_display_name: row.try_get("user_display_name")?,
            body: row.try_get("body")?,
            created_at: row.try_get("created_at")?,
            hidden: row.try_get("hidden")?,
        })
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for AlbumComment {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> nimble_web::data::query::Value {
        nimble_web::data::query::Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "id",
            "album_id",
            "user_id",
            "user_display_name",
            "body",
            "created_at",
            "hidden",
        ]
    }

    fn insert_values(&self) -> Vec<nimble_web::data::query::Value> {
        vec![
            nimble_web::data::query::Value::Uuid(self.id.expect("id not set")),
            PostgresValueBuilder::optional_uuid(self.album_id),
            PostgresValueBuilder::optional_uuid(self.user_id),
            PostgresValueBuilder::optional_string(&self.user_display_name),
            PostgresValueBuilder::optional_string(&self.body),
            PostgresValueBuilder::optional_datetime(&self.created_at),
            nimble_web::data::query::Value::Bool(self.hidden),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["user_display_name", "body", "hidden"]
    }

    fn update_values(&self) -> Vec<nimble_web::data::query::Value> {
        vec![
            PostgresValueBuilder::optional_string(&self.user_display_name),
            PostgresValueBuilder::optional_string(&self.body),
            nimble_web::data::query::Value::Bool(self.hidden),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("album_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("user_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("user_display_name", ColumnType::Text),
            ColumnDef::new("body", ColumnType::Text).not_null(),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
            ColumnDef::new("hidden", ColumnType::Boolean)
                .not_null()
                .default("false"),
        ]
    }
}
