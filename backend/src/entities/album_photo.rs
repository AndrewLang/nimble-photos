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
pub struct AlbumPhoto {
    #[serde(default)]
    pub id: Uuid,
    #[serde(alias = "album_id")]
    pub album_id: Uuid,
    #[serde(alias = "photo_id")]
    pub photo_id: Uuid,
    #[serde(alias = "created_at")]
    pub created_at: Option<DateTime<Utc>>,
}

impl AlbumPhoto {
    pub fn new(album_id: Uuid, photo_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            album_id,
            photo_id,
            created_at: Some(Utc::now()),
        }
    }
}

impl Entity for AlbumPhoto {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "album_photo"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for AlbumPhoto {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &["id", "album_id", "photo_id", "created_at"]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::Uuid(self.album_id),
            Value::Uuid(self.photo_id),
            PostgresValueBuilder::optional_datetime(&self.created_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &["album_id", "photo_id", "created_at"]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.album_id),
            Value::Uuid(self.photo_id),
            PostgresValueBuilder::optional_datetime(&self.created_at),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid)
                .primary_key()
                .default("gen_random_uuid()"),
            ColumnDef::new("album_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("photo_id", ColumnType::Uuid).not_null(),
            ColumnDef::new("created_at", ColumnType::Timestamp).not_null(),
        ]
    }
}
