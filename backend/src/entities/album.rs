use super::uuid_id::HasOptionalUuidId;
use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::{PostgresEntity, value_builder::PostgresValueBuilder},
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::error::BoxDynError,
    sqlx::postgres::{PgTypeInfo, PgValueRef},
    sqlx::{Decode, FromRow, Postgres, Type},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlbumKind {
    Manual,
    Smart,
}

impl AlbumKind {
    fn as_str(&self) -> &'static str {
        match self {
            AlbumKind::Manual => "manual",
            AlbumKind::Smart => "smart",
        }
    }
}

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub create_date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub kind: AlbumKind,
    pub rules_json: Option<String>,
    pub thumbnail_hash: Option<String>,
    pub sort_order: i32,
    pub image_count: Option<i64>,
}

#[cfg(feature = "postgres")]
impl Type<Postgres> for AlbumKind {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("TEXT")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <&str as Type<Postgres>>::compatible(ty)
    }
}

#[cfg(feature = "postgres")]
impl<'r> Decode<'r, Postgres> for AlbumKind {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let kind = <&str as Decode<Postgres>>::decode(value)?;
        match kind {
            "manual" => Ok(AlbumKind::Manual),
            "smart" => Ok(AlbumKind::Smart),
            other => Err(BoxDynError::from(format!("invalid album kind: {other}"))),
        }
    }
}

impl Entity for Album {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        self.id
            .as_ref()
            .expect("Album entity requires an id for Entity trait operations")
    }

    fn name() -> &'static str {
        "album"
    }
}

impl HasOptionalUuidId for Album {
    fn id_slot(&mut self) -> &mut Option<Uuid> {
        &mut self.id
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
            "parent_id",
            "name",
            "create_date",
            "description",
            "category",
            "kind",
            "rules_json",
            "thumbnail_hash",
            "sort_order",
            "image_count",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        let id = self.id.as_ref().expect("id not set for Album insert");
        vec![
            Value::Uuid(*id),
            PostgresValueBuilder::optional_uuid(self.parent_id),
            Value::String(self.name.clone()),
            PostgresValueBuilder::optional_datetime(&self.create_date),
            PostgresValueBuilder::optional_string(&self.description),
            PostgresValueBuilder::optional_string(&self.category),
            Value::String(self.kind.as_str().to_string()),
            PostgresValueBuilder::optional_string(&self.rules_json),
            PostgresValueBuilder::optional_string(&self.thumbnail_hash),
            Value::Int(self.sort_order as i64),
            PostgresValueBuilder::optional_i64(self.image_count),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "parent_id",
            "name",
            "create_date",
            "description",
            "category",
            "kind",
            "rules_json",
            "thumbnail_hash",
            "sort_order",
            "image_count",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            PostgresValueBuilder::optional_uuid(self.parent_id),
            Value::String(self.name.clone()),
            PostgresValueBuilder::optional_datetime(&self.create_date),
            PostgresValueBuilder::optional_string(&self.description),
            PostgresValueBuilder::optional_string(&self.category),
            Value::String(self.kind.as_str().to_string()),
            PostgresValueBuilder::optional_string(&self.rules_json),
            PostgresValueBuilder::optional_string(&self.thumbnail_hash),
            Value::Int(self.sort_order as i64),
            PostgresValueBuilder::optional_i64(self.image_count),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("parent_id", ColumnType::Uuid),
            ColumnDef::new("name", ColumnType::Text).not_null(),
            ColumnDef::new("create_date", ColumnType::Timestamp),
            ColumnDef::new("description", ColumnType::Text),
            ColumnDef::new("category", ColumnType::Text),
            ColumnDef::new("kind", ColumnType::Text).not_null(),
            ColumnDef::new("rules_json", ColumnType::Text),
            ColumnDef::new("thumbnail_hash", ColumnType::Text),
            ColumnDef::new("sort_order", ColumnType::Integer).not_null(),
            ColumnDef::new("image_count", ColumnType::BigInt),
        ]
    }
}
