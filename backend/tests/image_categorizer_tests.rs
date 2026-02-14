use chrono::{TimeZone, Utc};
use nimble_photos::services::hash_service::HashService;
use nimble_photos::services::image_categorizer::{CategorizeRequest, ImageCategorizerRegistry};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nimble_photos_image_categorizer_tests_{}_{}_{}",
        std::process::id(),
        name,
        nanos
    ))
}

fn write_test_file(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    fs::write(path, contents).expect("failed to write test file");
}

#[test]
fn hash_categorizer_moves_file_into_hashed_buckets() {
    let root = unique_temp_dir("hash");
    let source = root.join("incoming.bin");
    let destination_root = root.join("storage");
    fs::create_dir_all(&destination_root).expect("failed to create destination root");
    write_test_file(&source, b"hello world");

    let registry = ImageCategorizerRegistry::with_defaults(Arc::new(HashService::new()));
    let categorizer = registry.get("hash").expect("hash categorizer missing");

    let request = CategorizeRequest::new(&source, &destination_root, "photo.bin");
    let result = categorizer.categorize(&request).expect("categorize failed");

    assert!(result.hash.is_some(), "hash categorizer must compute hash");
    assert!(
        !source.exists(),
        "source file should be moved out of temp location"
    );
    assert!(
        result.final_path.exists(),
        "final categorized file should exist"
    );

    let hash = result.hash.unwrap();
    assert!(
        result
            .relative_path
            .starts_with(&format!("{}/{}/", &hash[0..2], &hash[2..4])),
        "relative path must include hash folders"
    );
}

#[test]
fn date_categorizer_uses_date_taken_or_file_metadata() {
    let root = unique_temp_dir("date");
    let destination_root = root.join("storage");
    fs::create_dir_all(&destination_root).expect("failed to create destination root");
    let source_with_date = root.join("with_date.jpg");
    write_test_file(&source_with_date, b"a photo with date");

    let registry = ImageCategorizerRegistry::with_defaults(Arc::new(HashService::new()));
    let categorizer = registry.get("date").expect("date categorizer missing");

    let date_taken = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
    let request_with_date =
        CategorizeRequest::new(&source_with_date, &destination_root, "dated.jpg")
            .with_date_taken(Some(date_taken));
    let result = categorizer
        .categorize(&request_with_date)
        .expect("categorize with date failed");
    assert!(
        result.relative_path.starts_with("2024-01-02/"),
        "date folder should match provided date"
    );

    let source_without_date = root.join("without_date.jpg");
    write_test_file(&source_without_date, b"a photo without date");
    let request_without_date =
        CategorizeRequest::new(&source_without_date, &destination_root, "nodate.jpg");
    let result_without_date = categorizer
        .categorize(&request_without_date)
        .expect("categorize without explicit date failed");

    let today = Utc::now().format("%Y-%m-%d").to_string();
    assert!(
        result_without_date
            .relative_path
            .starts_with(&format!("{}/", today)),
        "fallback date folder should use file metadata or current date"
    );
}

#[test]
fn registry_caches_instances_per_name() {
    let registry = ImageCategorizerRegistry::with_defaults(Arc::new(HashService::new()));
    let first = registry.get("hash").expect("hash categorizer missing");
    let second = registry.get("hash").expect("hash categorizer missing");
    assert!(
        Arc::ptr_eq(&first, &second),
        "registry should cache categorizer instances"
    );
}
