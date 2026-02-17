use nimble_photos::entities::StorageLocation;
use std::path::PathBuf;

#[test]
fn image_storage_location_normalizes_relative_path() {
    let cwd = std::env::current_dir().expect("current directory missing");
    let relative = PathBuf::from("storage-relative");

    let storage = StorageLocation {
        id: "1".to_string(),
        label: "Test".to_string(),
        path: relative.to_string_lossy().to_string(),
        is_default: false,
        created_at: "2026-02-15".to_string(),
        category_template: "{year}/{date:%Y-%m-%d}/{fileName}".to_string(),
    };

    assert_eq!(storage.normalized_path(), cwd.join(relative));
    assert_eq!(storage.category_template, "{year}/{date:%Y-%m-%d}/{fileName}");
}

#[test]
fn image_storage_location_keeps_absolute_path() {
    let absolute = std::env::temp_dir().join("nimble-storage-abs");

    let storage = StorageLocation {
        id: "2".to_string(),
        label: "Temp".to_string(),
        path: absolute.to_string_lossy().to_string(),
        is_default: false,
        created_at: "2026-02-15".to_string(),
        category_template: "{year}/{date:%Y-%m-%d}/{fileName}".to_string(),
    };

    assert_eq!(storage.normalized_path(), absolute);
    assert_eq!(storage.category_template, "{year}/{date:%Y-%m-%d}/{fileName}");
}
