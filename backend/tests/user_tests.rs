use chrono::Utc;

use nimble_photos::entities::user::User;

#[test]
fn user_basic_properties() {
    let user = User {
        id: "u1".to_string(),
        email: "test@example.com".to_string(),
        display_name: "test user".to_string(),
        password_hash: "hashed".to_string(),
        created_at: Utc::now(),
        reset_token: None,
        reset_token_expires_at: None,
        verification_token: None,
        email_verified: false,
    };

    assert_eq!(user.id, "u1".to_string());
    assert_eq!(user.email, "test@example.com");
}
