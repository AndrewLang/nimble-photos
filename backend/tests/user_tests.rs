use chrono::Utc;
use uuid::Uuid;

use nimble_photos::entities::user::User;

const USER_ID_STR: &str = "00000000-0000-0000-0000-000000000001";

#[test]
fn user_basic_properties() {
    let user_id = Uuid::parse_str(USER_ID_STR).unwrap();
    let user = User {
        id: user_id,
        email: "test@example.com".to_string(),
        display_name: "test user".to_string(),
        password_hash: "hashed".to_string(),
        created_at: Utc::now(),
        reset_token: None,
        reset_token_expires_at: None,
        verification_token: None,
        email_verified: false,
        roles: None,
    };

    assert_eq!(user.id, user_id);
    assert_eq!(user.email, "test@example.com");
}
