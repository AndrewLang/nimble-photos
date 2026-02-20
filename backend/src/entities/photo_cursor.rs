use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoCursor {
    #[serde(alias = "date_taken", alias = "dateTaken")]
    pub sort_date: DateTime<Utc>,
    pub id: Uuid,
}

impl PhotoCursor {
    pub fn encode(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        general_purpose::STANDARD.encode(json)
    }

    pub fn decode(encoded: &str) -> anyhow::Result<Self> {
        let bytes = general_purpose::STANDARD.decode(encoded)?;
        let cursor = serde_json::from_slice(&bytes)?;
        Ok(cursor)
    }
}
