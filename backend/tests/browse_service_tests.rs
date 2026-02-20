use nimble_photos::entities::photo_browse::BrowseOptions;
use nimble_photos::services::BrowseService;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

#[test]
fn browse_request_path_segments_split_correctly() {
    let request = nimble_photos::entities::photo_browse::BrowseRequest {
        path: Some("2026/Nikon/2026-01-25".to_string()),
        page_size: Some(50),
        cursor: None,
    };

    assert_eq!(
        request.path_segments().unwrap(),
        vec![
            "2026".to_string(),
            "Nikon".to_string(),
            "2026-01-25".to_string()
        ]
    );
}

#[test]
fn browse_request_path_segments_decode_encoded_slash() {
    let request = nimble_photos::entities::photo_browse::BrowseRequest {
        path: Some("2025%2F2025-06-11".to_string()),
        page_size: Some(50),
        cursor: None,
    };

    assert_eq!(
        request.path_segments().unwrap(),
        vec!["2025".to_string(), "2025-06-11".to_string()]
    );
}

#[tokio::test]
async fn browse_service_returns_error_for_invalid_depth() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@localhost:5432/nimble_photos")
        .expect("lazy pool should be created");
    let service = BrowseService::new(Arc::new(pool));
    let options = BrowseOptions::default();
    let segments = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let storage_id = Uuid::new_v4();

    let result = service
        .browse(&storage_id, &segments, &options, 50, None)
        .await;

    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().to_string(),
        "invalid browse path depth"
    );
}
