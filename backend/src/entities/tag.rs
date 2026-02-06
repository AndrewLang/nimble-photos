use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use sqlx::FromRow;

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub visibility: i16,
    pub created_at: Option<DateTime<Utc>>,
}

