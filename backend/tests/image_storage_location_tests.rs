use nimble_photos::entities::ImageStorageLocation;
use std::path::PathBuf;

#[test]
fn image_storage_location_normalizes_relative_path() {
    let cwd = std::env::current_dir().expect("current directory missing");
    let relative = PathBuf::from("storage-relative");

    let storage = ImageStorageLocation::new("1", "Test", relative.clone(), "2026-02-15");

    assert_eq!(storage.path, cwd.join(relative));
    assert_eq!(storage.category_policy, "hash");
}

#[test]
fn image_storage_location_keeps_absolute_path() {
    let absolute = std::env::temp_dir().join("nimble-storage-abs");

    let storage = ImageStorageLocation::new("2", "Temp", absolute.clone(), "2026-02-15");

    assert_eq!(storage.path, absolute);
    assert_eq!(storage.category_policy, "hash");
}
