use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};
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
pub struct Photo {
    pub id: Uuid,
    pub hash: String,
    pub path: String,
    pub file_name: String,
    pub file_size: i64,
    pub rating: Option<i16>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
            "hash",
            "path",
            "file_name",
            "file_size",
            "rating",
            "label",
            "description",
            "created_at",
            "updated_at",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::String(self.hash.clone()),
            Value::String(self.path.clone()),
            Value::String(self.file_name.clone()),
            Value::Int(self.file_size),
            match self.rating {
                Some(v) => Value::Int(v as i64),
                None => Value::Null,
            },
            match &self.label {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            match &self.description {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            Value::DateTime(self.created_at),
            Value::DateTime(self.updated_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "hash",
            "path",
            "file_name",
            "file_size",
            "rating",
            "label",
            "description",
            "updated_at",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.hash.clone()),
            Value::String(self.path.clone()),
            Value::String(self.file_name.clone()),
            Value::Int(self.file_size),
            match self.rating {
                Some(v) => Value::Int(v as i64),
                None => Value::Null,
            },
            match &self.label {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            match &self.description {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            Value::DateTime(self.updated_at),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("hash", ColumnType::Text).not_null(),
            ColumnDef::new("path", ColumnType::Text).not_null(),
            ColumnDef::new("file_name", ColumnType::Text).not_null(),
            ColumnDef::new("file_size", ColumnType::BigInt).not_null(),
            ColumnDef::new("rating", ColumnType::Integer),
            ColumnDef::new("label", ColumnType::Text),
            ColumnDef::new("description", ColumnType::Text),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
            ColumnDef::new("updated_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
