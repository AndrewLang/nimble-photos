use chrono::Utc;

use nimble_photos::dtos::user_profile_dto::UserProfileDto;
use nimble_photos::entities::{user::User, user_settings::UserSettings};

#[test]
fn user_settings_and_profile_dto_conversion() {
    let user = User {
        id: "u1".to_string(),
        email: "me@example.com".to_string(),
        password_hash: "x".to_string(),
        created_at: Utc::now(),
    };

    let settings = UserSettings {
        user_id: "u1".to_string(),
        display_name: "Display Name".to_string(),
        avatar_url: None,
        theme: "dark".to_string(),
        language: "en".to_string(),
        timezone: "UTC".to_string(),
        created_at: Utc::now(),
    };

    let dto: UserProfileDto = (user, settings).into();

    assert_eq!(dto.id, "u1");
    assert_eq!(dto.email, "me@example.com");
    assert_eq!(dto.display_name, "Display Name");
    assert_eq!(dto.theme, "dark");
}
