use nimble_photos::services::PhotoUploadService;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nimble_photos_photo_upload_service_{}_{}",
        std::process::id(),
        suffix
    ))
}

fn multipart_content_type(boundary: &str) -> String {
    format!("multipart/form-data; boundary={boundary}")
}

fn multipart_body(boundary: &str) -> Vec<u8> {
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"files\"; filename=\"a.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfile-a\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"files\"; filename=\"b.png\"\r\nContent-Type: image/png\r\n\r\nfile-b\r\n--{boundary}--\r\n"
    );
    body.into_bytes()
}

#[tokio::test]
async fn parse_multipart_files_reads_files_field_entries() {
    let service = PhotoUploadService::new();
    let boundary = "boundary-123";
    let content_type = multipart_content_type(boundary);
    let body = multipart_body(boundary);

    let files = service
        .parse_multipart_files(&content_type, body)
        .await
        .expect("failed to parse multipart files");

    assert_eq!(files.len(), 2);
    assert_eq!(files[0].file_name, "a.jpg");
    assert_eq!(files[1].file_name, "b.png");
    assert_eq!(files[0].bytes, b"file-a");
    assert_eq!(files[1].bytes, b"file-b");
}

#[tokio::test]
async fn persist_to_storage_temp_writes_uploaded_files() {
    let service = PhotoUploadService::new();
    let boundary = "boundary-456";
    let content_type = multipart_content_type(boundary);
    let body = multipart_body(boundary);
    let files = service
        .parse_multipart_files(&content_type, body)
        .await
        .expect("failed to parse multipart files");
    let temp_root = unique_temp_dir();
    fs::create_dir_all(&temp_root).expect("failed to create test temp root");

    let saved = service
        .persist_to_storage_temp(&temp_root, files)
        .await
        .expect("failed to persist files");

    assert_eq!(saved.len(), 2);
    for saved_file in saved {
        let full_path = temp_root.join(saved_file.relative_path);
        assert!(full_path.exists());
        let data = fs::read(&full_path).expect("failed to read persisted file");
        assert_eq!(data.len(), saved_file.byte_size);
    }

    let _ = fs::remove_dir_all(temp_root);
}
