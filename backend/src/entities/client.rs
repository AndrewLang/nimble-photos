use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::{PostgresEntity, value_builder::PostgresValueBuilder},
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::FromRow,
};

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub api_key_hash: String,
    pub is_active: bool,
    pub is_approved: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Client {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "Client"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for Client {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "id",
            "user_id",
            "name",
            "api_key_hash",
            "is_active",
            "is_approved",
            "last_seen_at",
            "created_at",
            "updated_at",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::Uuid(self.user_id),
            Value::String(self.name.clone()),
            Value::String(self.api_key_hash.clone()),
            Value::Bool(self.is_active),
            Value::Bool(self.is_approved),
            PostgresValueBuilder::optional_datetime(&self.last_seen_at),
            Value::DateTime(self.created_at),
            Value::DateTime(self.updated_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "name",
            "api_key_hash",
            "is_active",
            "is_approved",
            "last_seen_at",
            "updated_at",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            Value::String(self.api_key_hash.clone()),
            Value::Bool(self.is_active),
            Value::Bool(self.is_approved),
            PostgresValueBuilder::optional_datetime(&self.last_seen_at),
            Value::DateTime(self.updated_at),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("user_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("name", ColumnType::Text).not_null(),
            ColumnDef::new("api_key_hash", ColumnType::Text).not_null(),
            ColumnDef::new("is_active", ColumnType::Boolean)
                .not_null()
                .default("true"),
            ColumnDef::new("is_approved", ColumnType::Boolean)
                .not_null()
                .default("false"),
            ColumnDef::new("last_seen_at", ColumnType::Timestamp),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
            ColumnDef::new("updated_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
