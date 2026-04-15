use nimble_photos::controllers::storage_controller::StorageController;
use nimble_photos::entities::{ExifModel, Photo, StorageLocation};
use nimble_photos::services::{
    BackgroundTaskRunner, EventBusService, ExifService, FileService, HashService,
    ImageProcessPipeline, ImageProcessPipelineContext, PhotoUploadService, PreviewExtractor,
    StorageService, ThumbnailExtractor,
};
use nimble_web::testkit::request::HttpRequestBuilder;
use nimble_web::testkit::response::ResponseAssertions;
use nimble_web::{AppBuilder, Application, DataProvider, HttpRequest, HttpResponse, MemoryRepository, QueryBuilder, Repository};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn unique_temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nimble_photos_storage_sync_controller_{}_{}",
        std::process::id(),
        suffix
    ))
}

fn build_app(
    storage_seed: Vec<StorageLocation>,
    photo_seed: Vec<Photo>,
    exif_seed: Vec<ExifModel>,
) -> Application {
    let storage_provider = MemoryRepository::<StorageLocation>::new();
    storage_provider.seed(storage_seed);

    let photo_provider = MemoryRepository::<Photo>::new();
    photo_provider.seed(photo_seed);

    let exif_provider = MemoryRepository::<ExifModel>::new();
    exif_provider.seed(exif_seed);

    let mut builder = AppBuilder::new();
    builder.use_controller::<StorageController>();

    builder.register_singleton({
        let storage_provider = storage_provider.clone();
        move |_| Repository::<StorageLocation>::new(Box::new(storage_provider.clone()))
    });
    builder.register_singleton({
        let photo_provider = photo_provider.clone();
        move |_| Repository::<Photo>::new(Box::new(photo_provider.clone()))
    });
    builder.register_singleton({
        let exif_provider = exif_provider.clone();
        move |_| Repository::<ExifModel>::new(Box::new(exif_provider.clone()))
    });
    builder.register_singleton(|provider| {
        let storage_repo = provider.get::<Repository<StorageLocation>>();
        let photo_repo = provider.get::<Repository<Photo>>();
        let exif_repo = provider.get::<Repository<ExifModel>>();
        StorageService::new(storage_repo, photo_repo, exif_repo)
    });
    builder.register_singleton(|_| PhotoUploadService::new(64 * 1024 * 1024));
    builder.register_singleton(|_| HashService::new());
    builder.register_singleton(|_| ExifService::new());
    builder.register_singleton(|_| ThumbnailExtractor::new());
    builder.register_singleton(|_| PreviewExtractor::new());
    builder.register_singleton(|_| FileService::new());
    builder.register_singleton(|_| BackgroundTaskRunner::new(1));
    builder.register_singleton(|_| EventBusService::new(16));
    builder.register_singleton(|provider| {
        let configuration = provider.get::<nimble_web::Configuration>().as_ref().clone();
        ImageProcessPipeline::new(ImageProcessPipelineContext::new(provider, configuration))
    });

    builder.build()
}

fn handle_request(app: &Application, request: HttpRequest) -> HttpResponse {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    runtime.block_on(app.handle_http_request(request))
}

fn sample_storage(path: &str) -> StorageLocation {
    StorageLocation {
        id: Uuid::new_v4(),
        label: "Sync Storage".to_string(),
        path: path.to_string(),
        is_default: true,
        readonly: false,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        category_template: "{year}/{date:%Y-%m-%d}/{fileName}".to_string(),
    }
}

fn sample_photo(storage_id: Uuid, hash: &str, size: i64) -> Photo {
    Photo {
        id: Uuid::new_v4(),
        storage_id,
        path: "sample.jpg".to_string(),
        name: "sample.jpg".to_string(),
        format: Some("jpg".to_string()),
        hash: Some(hash.to_string()),
        size: Some(size),
        created_at: None,
        updated_at: None,
        date_imported: None,
        date_taken: None,
        year: None,
        month_day: None,
        metadata_extracted: None,
        artist: None,
        make: None,
        model: None,
        lens_make: None,
        lens_model: None,
        exposure_time: None,
        iso: None,
        aperture: None,
        focal_length: None,
        label: None,
        rating: None,
        flagged: None,
        is_raw: None,
        width: None,
        height: None,
        orientation: None,
        day_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date"),
        sort_date: chrono::Utc::now(),
    }
}

fn sync_multipart_content_type(boundary: &str) -> String {
    format!("multipart/form-data; boundary={boundary}")
}

fn sync_multipart_body(boundary: &str, item_json: &str, file_bytes: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"item\"\r\nContent-Type: application/json\r\n\r\n{item_json}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sync.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

#[test]
fn sync_storage_check_handler_returns_missing_files() {
    let temp_root = unique_temp_dir();
    fs::create_dir_all(&temp_root).expect("create temp root");
    let storage = sample_storage(temp_root.to_string_lossy().as_ref());
    let existing_photo = sample_photo(storage.id, "existing-hash", 5);
    let app = build_app(vec![storage.clone()], vec![existing_photo], Vec::new());

    let payload = json!({
        "storageId": storage.id.to_string(),
        "files": [
            { "hash": "existing-hash", "fileSize": 5 },
            { "hash": "missing-hash", "fileSize": 9 }
        ]
    });

    let request = HttpRequestBuilder::post("/api/storage/sync/check")
        .header("content-type", "application/json")
        .body(&payload.to_string())
        .build();

    let response = handle_request(&app, request);
    response.assert_status(200);

    let parsed: serde_json::Value = response.assert_json();
    assert_eq!(
        parsed,
        json!({
            "missingFiles": [
                { "hash": "missing-hash", "fileSize": 9 }
            ]
        })
    );

    let _ = fs::remove_dir_all(temp_root);
}

#[test]
fn sync_storage_metadata_handler_persists_metadata() {
    let temp_root = unique_temp_dir();
    fs::create_dir_all(&temp_root).expect("create temp root");
    let storage = sample_storage(temp_root.to_string_lossy().as_ref());
    let photo = sample_photo(storage.id, "sync-hash", 5);
    let photo_id = photo.id;
    let app = build_app(vec![storage.clone()], vec![photo], Vec::new());

    let payload = json!({
        "storageId": storage.id.to_string(),
        "hash": "sync-hash",
        "metadata": {
            "make": "Canon",
            "model": "EOS R5",
            "artist": "Andy",
            "datetimeOriginal": "2026:04:01 12:30:45",
            "imageWidth": 8192,
            "imageLength": 5464,
            "iso": 400,
            "fNumber": 2.8,
            "orientation": 1
        }
    });

    let request = HttpRequestBuilder::post("/api/storage/sync/metadata")
        .header("content-type", "application/json")
        .body(&payload.to_string())
        .build();

    let response = handle_request(&app, request);
    response.assert_status(200);

    let parsed: serde_json::Value = response.assert_json();
    assert_eq!(parsed["storageId"], storage.id.to_string());
    assert_eq!(parsed["hash"], "sync-hash");
    assert_eq!(parsed["metadata"]["make"], "Canon");
    assert_eq!(parsed["metadata"]["imageWidth"], 8192);

    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let photo_repo = app.services().get::<Repository<Photo>>();
    let exif_repo = app.services().get::<Repository<ExifModel>>();

    let saved_photo = runtime
        .block_on(photo_repo.get(&photo_id))
        .expect("load saved photo")
        .expect("photo exists");
    assert_eq!(saved_photo.make.as_deref(), Some("Canon"));
    assert_eq!(saved_photo.model.as_deref(), Some("EOS R5"));
    assert_eq!(saved_photo.artist.as_deref(), Some("Andy"));
    assert_eq!(saved_photo.width, Some(8192));
    assert_eq!(saved_photo.height, Some(5464));
    assert_eq!(saved_photo.iso, Some(400));
    assert_eq!(saved_photo.orientation, Some(1));
    assert_eq!(saved_photo.metadata_extracted, Some(true));
    assert_eq!(
        saved_photo.date_taken,
        Some(
            chrono::DateTime::parse_from_rfc3339("2026-04-01T12:30:45Z")
                .expect("expected timestamp")
                .with_timezone(&chrono::Utc),
        )
    );

    let saved_exif = runtime
        .block_on(exif_repo.all(QueryBuilder::<ExifModel>::new().build()))
        .expect("load saved metadata")
        .into_iter()
        .find(|metadata| metadata.image_id == photo_id)
        .expect("metadata exists");
    assert_eq!(saved_exif.hash, "sync-hash");
    assert_eq!(saved_exif.image_id, photo_id);
    assert_eq!(saved_exif.make.as_deref(), Some("Canon"));
    assert_eq!(saved_exif.model.as_deref(), Some("EOS R5"));
    assert_eq!(saved_exif.iso, Some(400));
    assert_eq!(saved_exif.orientation, Some(1));

    let _ = fs::remove_dir_all(temp_root);
}

#[test]
fn sync_storage_file_handler_persists_file_and_returns_upload_info() {
    let temp_root = unique_temp_dir();
    fs::create_dir_all(&temp_root).expect("create temp root");
    let storage = sample_storage(temp_root.to_string_lossy().as_ref());
    let app = build_app(vec![storage.clone()], Vec::new(), Vec::new());

    let file_bytes = b"streamed-file";
    let item_json = json!({
        "storageId": storage.id.to_string(),
        "hash": "sync-hash",
        "fileName": "sync.jpg",
        "fileSize": file_bytes.len(),
        "contentType": "image/jpeg"
    })
    .to_string();
    let boundary = "sync-boundary-123";
    let body = sync_multipart_body(boundary, &item_json, file_bytes);

    let mut request = HttpRequest::new("POST", "/api/storage/sync");
    request
        .headers_mut()
        .insert("content-type", &sync_multipart_content_type(boundary));
    request.set_body(nimble_web::RequestBody::Bytes(body));

    let response = handle_request(&app, request);
    response.assert_status(200);

    let parsed: serde_json::Value = response.assert_json();
    assert_eq!(parsed["storageId"], storage.id.to_string());
    assert_eq!(parsed["hash"], "sync-hash");

    let relative_path = parsed["file"]["relativePath"]
        .as_str()
        .expect("relativePath");
    let full_path = temp_root.join(relative_path);
    assert!(full_path.exists(), "expected synced file at {}", full_path.display());
    assert_eq!(fs::read(&full_path).expect("read synced file"), file_bytes);

    let _ = fs::remove_dir_all(temp_root);
}
