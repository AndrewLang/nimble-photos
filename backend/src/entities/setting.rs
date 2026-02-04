use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use nimble_web::Entity;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::PostgresEntity,
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::{
        database::Database,
        decode::Decode,
        encode::{Encode, IsNull},
        error::BoxDynError,
        postgres::{PgTypeInfo, PgValueRef, Postgres},
        Type,
    },
    sqlx::FromRow,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "postgres", derive(FromRow))]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub value_type: SettingValueType,
    #[cfg_attr(feature = "postgres", sqlx(rename = "group_name"))]
    pub group: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SettingValueType {
    String,
    Boolean,
    Number,
    Json,
}

impl SettingValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SettingValueType::String => "string",
            SettingValueType::Boolean => "boolean",
            SettingValueType::Number => "number",
            SettingValueType::Json => "json",
        }
    }

    pub fn matches(&self, value: &JsonValue) -> bool {
        match self {
            SettingValueType::String => value.is_string(),
            SettingValueType::Boolean => value.is_boolean(),
            SettingValueType::Number => value.is_number(),
            SettingValueType::Json => true,
        }
    }
}

#[cfg(feature = "postgres")]
impl Type<Postgres> for SettingValueType {
    fn type_info() -> PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <String as Type<Postgres>>::compatible(ty)
    }
}

#[cfg(feature = "postgres")]
impl<'r> Decode<'r, Postgres> for SettingValueType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let raw = <String as Decode<Postgres>>::decode(value)?;
        SettingValueType::from_str(&raw).map_err(|_| {
            BoxDynError::from(format!(
                "Invalid setting value type stored in database: {}",
                raw
            ))
        })
    }
}

#[cfg(feature = "postgres")]
impl Encode<'_, Postgres> for SettingValueType {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<Postgres>>::encode_by_ref(&self.as_str().to_string(), buf)
    }
}

impl Display for SettingValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SettingValueType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.to_ascii_lowercase();
        match normalized.as_str() {
            "string" => Ok(SettingValueType::String),
            "boolean" => Ok(SettingValueType::Boolean),
            "number" => Ok(SettingValueType::Number),
            "json" => Ok(SettingValueType::Json),
            _ => Err(()),
        }
    }
}

impl Entity for Setting {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.key
    }

    fn name() -> &'static str {
        "Setting"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for Setting {
    fn id_column() -> &'static str {
        "key"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::String(id.clone())
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "key",
            "value_type",
            "value",
            "group_name",
            "created_at",
            "updated_at",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.key.clone()),
            Value::String(self.value_type.to_string()),
            Value::String(self.value.clone()),
            Value::String(self.group.clone()),
            Value::DateTime(self.created_at),
            Value::DateTime(self.updated_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["value_type", "value", "group_name", "updated_at"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.value_type.to_string()),
            Value::String(self.value.clone()),
            Value::String(self.group.clone()),
            Value::DateTime(self.updated_at),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("key", ColumnType::Text).primary_key(),
            ColumnDef::new("value_type", ColumnType::Text).not_null(),
            ColumnDef::new("value", ColumnType::Text).not_null(),
            ColumnDef::new("group_name", ColumnType::Text).not_null(),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
            ColumnDef::new("updated_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
