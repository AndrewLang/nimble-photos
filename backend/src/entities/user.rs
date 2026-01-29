use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use nimble_web::data::postgres::PostgresEntity;
#[cfg(feature = "postgres")]
use nimble_web::data::query::Value;
#[cfg(feature = "postgres")]
use nimble_web::data::schema::{ColumnDef, ColumnType};
#[cfg(feature = "postgres")]
use sqlx::FromRow;

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl Entity for User {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "User"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for User {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::String(id.clone())
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "email", "password_hash", "created_at"]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.id.clone()),
            Value::String(self.email.clone()),
            Value::String(self.password_hash.clone()),
            Value::String(self.created_at.to_rfc3339()),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["email", "password_hash"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.email.clone()),
            Value::String(self.password_hash.clone()),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Text).primary_key(),
            ColumnDef::new("email", ColumnType::Text).not_null(),
            ColumnDef::new("password_hash", ColumnType::Text).not_null(),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
