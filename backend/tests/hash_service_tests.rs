use nimble_photos::services::HashService;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_file_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nimble_photos_hash_service_{}_{}.bin",
        std::process::id(),
        nanos
    ))
}

#[test]
fn compute_is_stable_for_same_input() {
    let service = HashService::new();
    let data = b"hash-service-stability-check".to_vec();
    let file_size = data.len();
    let file_date = UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);

    let first = service.compute(&data, file_size, file_date);
    let second = service.compute(&data, file_size, file_date);

    assert_eq!(first, second);
    assert!(!first.is_empty());
}

#[test]
fn compute_file_matches_compute_for_same_file_metadata() {
    let service = HashService::new();
    let path = unique_temp_file_path();
    let data = b"hash-service-file-compute-check".to_vec();

    fs::write(&path, &data).expect("failed to create temp test file");
    let metadata = fs::metadata(&path).expect("failed to load temp file metadata");
    let expected = service.compute(&data, metadata.len() as usize, metadata.modified().unwrap());

    let actual = service
        .compute_file(path.to_str().expect("invalid temp file path"))
        .expect("compute_file failed");

    assert_eq!(actual, expected);

    let _ = fs::remove_file(path);
}

#[test]
fn compute_file_returns_error_for_missing_path() {
    let service = HashService::new();
    let path = unique_temp_file_path();

    let result = service.compute_file(path.to_str().expect("invalid temp file path"));

    assert!(result.is_err());
}
