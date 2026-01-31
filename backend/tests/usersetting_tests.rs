use chrono::Utc;
use uuid::Uuid;

use nimble_photos::dtos::user_profile_dto::UserProfileDto;
use nimble_photos::entities::{user::User, user_settings::UserSettings};

const USER_ID_STR: &str = "00000000-0000-0000-0000-000000000001";

#[test]
fn user_settings_and_profile_dto_conversion() {
    let user_id = Uuid::parse_str(USER_ID_STR).unwrap();
    let user = User {
        id: user_id,
        email: "me@example.com".to_string(),
        display_name: "Me".to_string(),
        password_hash: "x".to_string(),
        created_at: Utc::now(),
        reset_token: None,
        reset_token_expires_at: None,
        verification_token: None,
        email_verified: false,
    };

    let settings = UserSettings {
        user_id: USER_ID_STR.to_string(),
        display_name: "Display Name".to_string(),
        avatar_url: None,
        theme: "dark".to_string(),
        language: "en".to_string(),
        timezone: "UTC".to_string(),
        created_at: Utc::now(),
    };

    let dto: UserProfileDto = (user, settings).into();

    assert_eq!(dto.id.to_string(), USER_ID_STR);
    assert_eq!(dto.email, "me@example.com");
    assert_eq!(dto.display_name, "Display Name");
    assert_eq!(dto.theme, "dark");
}
