use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};

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
