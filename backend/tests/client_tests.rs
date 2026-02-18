use chrono::Utc;
use nimble_photos::entities::client::Client;
use uuid::Uuid;

#[test]
fn client_basic_properties() {
    let id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let client = Client {
        id,
        user_id,
        name: "Mobile App".to_string(),
        device_name: "Pixel 9".to_string(),
        device_type: "phone".to_string(),
        version: "1.0.0".to_string(),
        api_key_hash: "hashed_key".to_string(),
        is_active: true,
        is_approved: false,
        approved_by: None,
        last_seen_at: None,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(client.id, id);
    assert_eq!(client.user_id, user_id);
    assert_eq!(client.name, "Mobile App");
    assert_eq!(client.device_name, "Pixel 9");
    assert_eq!(client.device_type, "phone");
    assert_eq!(client.version, "1.0.0");
    assert_eq!(client.api_key_hash, "hashed_key");
    assert!(client.is_active);
    assert!(!client.is_approved);
    assert!(client.approved_by.is_none());
    assert!(client.last_seen_at.is_none());
}
