use crate::prelude::*;

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
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub visibility: i16,
    pub created_at: Option<DateTime<Utc>>,
}

impl Entity for Tag {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "Tag"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for Tag {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "name", "visibility", "created_at"]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::String(self.name.clone()),
            Value::Int(self.visibility as i64),
            PostgresValueBuilder::optional_datetime(&self.created_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["name", "visibility"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.name.clone()),
            Value::Int(self.visibility as i64),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("name", ColumnType::Text).not_null(),
            ColumnDef::new("visibility", ColumnType::Integer)
                .not_null()
                .default("0"),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
