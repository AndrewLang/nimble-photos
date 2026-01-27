use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
