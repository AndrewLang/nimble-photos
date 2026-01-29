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
pub struct UserSettings {
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub theme: String,
    pub language: String,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
}

impl Entity for UserSettings {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.user_id
    }

    fn name() -> &'static str {
        "UserSettings"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for UserSettings {
    fn id_column() -> &'static str {
        "user_id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::String(id.clone())
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "user_id",
            "display_name",
            "avatar_url",
            "theme",
            "language",
            "timezone",
            "created_at",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.user_id.clone()),
            Value::String(self.display_name.clone()),
            match &self.avatar_url {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            Value::String(self.theme.clone()),
            Value::String(self.language.clone()),
            Value::String(self.timezone.clone()),
            Value::String(self.created_at.to_rfc3339()),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "display_name",
            "avatar_url",
            "theme",
            "language",
            "timezone",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.display_name.clone()),
            match &self.avatar_url {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            Value::String(self.theme.clone()),
            Value::String(self.language.clone()),
            Value::String(self.timezone.clone()),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("user_id", ColumnType::Text).primary_key(),
            ColumnDef::new("display_name", ColumnType::Text).not_null(),
            ColumnDef::new("avatar_url", ColumnType::Text),
            ColumnDef::new("theme", ColumnType::Text).not_null(),
            ColumnDef::new("language", ColumnType::Text).not_null(),
            ColumnDef::new("timezone", ColumnType::Text).not_null(),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
