use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserDto {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub email_verified: bool,
    pub roles: Vec<String>,
}

impl From<User> for AdminUserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            created_at: user.created_at,
            email_verified: user.email_verified,
            roles: parse_roles(user.roles.as_deref()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRolesRequest {
    pub roles: Vec<String>,
}

fn parse_roles(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or_default()
        .split(',')
        .map(|role| role.trim())
        .filter(|role| !role.is_empty())
        .map(ToString::to_string)
        .collect()
}
