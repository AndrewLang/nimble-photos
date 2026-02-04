use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::uuid_id::HasOptionalUuidId;
use nimble_web::Entity;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::{
        value_builder::PostgresValueBuilder,
        PostgresEntity,
    },
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::postgres::PgRow,
    sqlx::{FromRow, Row},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PhotoComment {
    pub id: Option<Uuid>,
    pub photo_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_display_name: Option<String>,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl Entity for PhotoComment {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        self.id
            .as_ref()
            .expect("PhotoComment requires an id for Entity trait operations")
    }

    fn name() -> &'static str {
        "photo_comment"
    }
}

impl HasOptionalUuidId for PhotoComment {
    fn id_slot(&mut self) -> &mut Option<Uuid> {
        &mut self.id
    }
}

#[cfg(feature = "postgres")]
impl<'r> FromRow<'r, PgRow> for PhotoComment {
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            photo_id: row.try_get("photo_id")?,
            user_id: row.try_get("user_id")?,
            user_display_name: row.try_get("user_display_name")?,
            body: row.try_get("body")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for PhotoComment {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> nimble_web::data::query::Value {
        nimble_web::data::query::Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "photo_id", "user_id", "user_display_name", "body", "created_at"]
    }

    fn insert_values(&self) -> Vec<nimble_web::data::query::Value> {
        vec![
            nimble_web::data::query::Value::Uuid(self.id.expect("id not set")),
            PostgresValueBuilder::optional_uuid(self.photo_id),
            PostgresValueBuilder::optional_uuid(self.user_id),
            PostgresValueBuilder::optional_string(&self.user_display_name),
            PostgresValueBuilder::optional_string(&self.body),
            PostgresValueBuilder::optional_datetime(&self.created_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["user_display_name", "body"]
    }

    fn update_values(&self) -> Vec<nimble_web::data::query::Value> {
        vec![
            PostgresValueBuilder::optional_string(&self.user_display_name),
            PostgresValueBuilder::optional_string(&self.body),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("photo_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("user_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("user_display_name", ColumnType::Text),
            ColumnDef::new("body", ColumnType::Text).not_null(),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
