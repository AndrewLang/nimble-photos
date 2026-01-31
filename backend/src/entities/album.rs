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
pub struct Album {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Album {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "album"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for Album {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "id",
            "name",
            "description",
            "owner_id",
            "created_at",
            "updated_at",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::String(self.name.clone()),
            match &self.description {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            match self.owner_id {
                Some(v) => Value::String(v.to_string()),
                None => Value::Null,
            },
            Value::DateTime(self.created_at),
            Value::DateTime(self.updated_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["name", "description", "owner_id", "updated_at"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            match &self.description {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            match self.owner_id {
                Some(v) => Value::String(v.to_string()),
                None => Value::Null,
            },
            Value::DateTime(self.updated_at),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("name", ColumnType::Text).not_null(),
            ColumnDef::new("description", ColumnType::Text),
            ColumnDef::new("owner_id", ColumnType::Uuid),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
            ColumnDef::new("updated_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
