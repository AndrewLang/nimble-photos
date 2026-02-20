use nimble_photos::entities::StorageLocation;
use nimble_photos::services::image_pipeline::ImageProcessPayload;
use std::path::PathBuf;
use uuid::Uuid;

fn make_storage(path: PathBuf) -> StorageLocation {
    StorageLocation {
        id: Uuid::new_v4(),
        label: "Primary".to_string(),
        path: path.to_string_lossy().to_string(),
        is_default: true,
        created_at: "2026-02-17T00:00:00Z".to_string(),
        category_template: "{year}/{date:%Y-%m-%d}/{fileName}".to_string(),
    }
}

#[test]
fn source_path_joins_storage_path_and_relative_path() {
    let root = std::env::temp_dir().join("nimble-image-path-test-source");
    let payload = ImageProcessPayload {
        storage: make_storage(root.clone()),
        relative_path: "temp/abcd1234.jpg".to_string(),
        file_name: "abcd1234.jpg".to_string(),
        byte_size: 42,
        content_type: Some("image/jpeg".to_string()),
    };

    assert_eq!(
        payload.source_path(),
        root.join("temp").join("abcd1234.jpg")
    );
}

#[test]
fn working_directory_matches_storage_normalized_path() {
    let root = std::env::temp_dir().join("nimble-image-path-test-workdir");
    let payload = ImageProcessPayload {
        storage: make_storage(root.clone()),
        relative_path: "temp/file.jpg".to_string(),
        file_name: "file.jpg".to_string(),
        byte_size: 42,
        content_type: None,
    };

    assert_eq!(payload.working_directory(), root);
}
