use crate::entities::{user::User, user_settings::UserSettings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileDto {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub theme: String,
    pub language: String,
    pub timezone: String,
}

impl From<(User, UserSettings)> for UserProfileDto {
    fn from((user, settings): (User, UserSettings)) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: settings.display_name,
            avatar_url: settings.avatar_url,
            theme: settings.theme,
            language: settings.language,
            timezone: settings.timezone,
        }
    }
}
