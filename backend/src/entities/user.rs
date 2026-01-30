use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::PostgresEntity,
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::FromRow,
};

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub reset_token: Option<String>,
    pub reset_token_expires_at: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
    #[serde(default)]
    pub email_verified: bool,
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
        &[
            "id",
            "email",
            "display_name",
            "password_hash",
            "created_at",
            "reset_token",
            "reset_token_expires_at",
            "verification_token",
            "email_verified",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.id.clone()),
            Value::String(self.email.clone()),
            Value::String(self.display_name.clone()),
            Value::String(self.password_hash.clone()),
            Value::DateTime(self.created_at),
            match &self.reset_token {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            match self.reset_token_expires_at {
                Some(v) => Value::DateTime(v),
                None => Value::Null,
            },
            match &self.verification_token {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            Value::Bool(self.email_verified),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "email",
            "display_name",
            "password_hash",
            "reset_token",
            "reset_token_expires_at",
            "verification_token",
            "email_verified",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.email.clone()),
            Value::String(self.display_name.clone()),
            Value::String(self.password_hash.clone()),
            match &self.reset_token {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            match self.reset_token_expires_at {
                Some(v) => Value::DateTime(v),
                None => Value::Null,
            },
            match &self.verification_token {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            Value::Bool(self.email_verified),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Text).primary_key(),
            ColumnDef::new("email", ColumnType::Text)
                .not_null()
                .unique(),
            ColumnDef::new("display_name", ColumnType::Text).not_null(),
            ColumnDef::new("password_hash", ColumnType::Text).not_null(),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
            ColumnDef::new("reset_token", ColumnType::Text),
            ColumnDef::new("reset_token_expires_at", ColumnType::Timestamp),
            ColumnDef::new("verification_token", ColumnType::Text),
            ColumnDef::new("email_verified", ColumnType::Boolean)
                .not_null()
                .default("false"),
        ]
    }
}
