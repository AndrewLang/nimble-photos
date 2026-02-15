use chrono::Utc;
use image::{ImageBuffer, Rgb};
use nimble_photos::entities::{exif::ExifModel, photo::Photo};
use nimble_photos::services::background_task_runner::BackgroundTaskRunner;
use nimble_photos::services::exif_service::ExifService;
use nimble_photos::services::file_service::FileService;
use nimble_photos::services::hash_service::HashService;
use nimble_photos::services::image_pipeline::{
    ImageProcessPipeline, ImageProcessRequest, ImageStorageLocation,
};
use nimble_photos::services::image_process_service::ImageProcessService;
use nimble_photos::services::photo_upload_service::StoredUploadFile;
use nimble_web::DataProvider;
use nimble_web::config::Configuration;
use nimble_web::data::memory_repository::MemoryRepository;
use nimble_web::data::query_builder::QueryBuilder;
use nimble_web::data::repository::Repository;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nimble_photos_image_pipeline_tests_{}_{}_{}",
        std::process::id(),
        name,
        nanos
    ))
}

fn write_test_image(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_fn(120, 80, |x, y| {
        let r = (x % 255) as u8;
        let g = (y % 255) as u8;
        let b = ((x + y) % 255) as u8;
        Rgb([r, g, b])
    });
    image
        .save_with_format(path, image::ImageFormat::Jpeg)
        .expect("failed to save test image");
}

fn test_configuration(thumbnail_root: &Path, preview_root: &Path) -> Configuration {
    let mut values = HashMap::new();
    values.insert(
        "thumbnail.base.path".to_string(),
        thumbnail_root.to_string_lossy().to_string(),
    );
    values.insert(
        "preview.base.path".to_string(),
        preview_root.to_string_lossy().to_string(),
    );
    Configuration::from_values(values)
}

async fn query_photos(repo: &Repository<Photo>) -> Vec<Photo> {
    repo.query(QueryBuilder::<Photo>::new().page(1, 10).build())
        .await
        .expect("photo query failed")
        .items
}

async fn query_exif(repo: &Repository<ExifModel>) -> Vec<ExifModel> {
    repo.query(QueryBuilder::<ExifModel>::new().page(1, 10).build())
        .await
        .expect("exif query failed")
        .items
}

#[tokio::test]
async fn pipeline_processes_uploaded_file_and_persists_metadata() {
    let root = unique_temp_dir("pipeline");
    let storage_root = root.join("storage");
    let temp_root = storage_root.join("temp");
    fs::create_dir_all(&temp_root).expect("failed to create storage temp directory");
    let thumbnail_root = root.join("thumbnails");
    let preview_root = root.join("previews");

    let file_name = "photo.jpg";
    let temp_file = temp_root.join(file_name);
    write_test_image(&temp_file);
    let file_size = fs::metadata(&temp_file).expect("metadata missing").len() as usize;

    let stored_file = StoredUploadFile {
        file_name: file_name.to_string(),
        relative_path: format!("temp/{}", file_name),
        byte_size: file_size,
        content_type: Some("image/jpeg".to_string()),
    };

    let storage = ImageStorageLocation::new(
        "storage-1",
        "Primary",
        &storage_root,
        Utc::now().to_rfc3339(),
    );

    let hash_service = Arc::new(HashService::new());
    let exif_service = Arc::new(ExifService::new());
    let image_service = Arc::new(ImageProcessService::new());
    let file_service = Arc::new(FileService::new());
    let runner = Arc::new(BackgroundTaskRunner::new(2));
    let photo_repo = Arc::new(Repository::new(Box::new(MemoryRepository::<Photo>::new())));
    let exif_repo = Arc::new(Repository::new(Box::new(
        MemoryRepository::<ExifModel>::new(),
    )));

    let pipeline = ImageProcessPipeline::new(
        runner,
        hash_service,
        exif_service,
        image_service,
        Arc::clone(&photo_repo),
        Arc::clone(&exif_repo),
        Arc::clone(&file_service),
        test_configuration(&thumbnail_root, &preview_root),
    );

    let request = ImageProcessRequest::from_upload(storage.clone(), stored_file.clone());
    pipeline
        .process(request)
        .await
        .expect("pipeline processing failed");

    assert!(
        !temp_file.exists(),
        "source file should be moved out of temp directory"
    );

    let photos = query_photos(&photo_repo).await;
    assert_eq!(photos.len(), 1, "one photo should be persisted");
    let photo = &photos[0];
    assert!(photo.hash.is_some(), "hash should be persisted");
    assert!(
        photo.thumbnail_path.is_some(),
        "thumbnail path should be set"
    );
    assert!(photo.thumbnail_width.unwrap_or(0) > 0);
    assert_eq!(photo.size, Some(file_size as i64));

    let final_path = storage_root.join(&photo.path);
    assert!(final_path.exists(), "final categorized file should exist");

    let thumbnail_path = thumbnail_root.join(photo.thumbnail_path.clone().unwrap());
    assert!(
        thumbnail_path.exists(),
        "thumbnail should be written to thumbnail root"
    );

    let exif_models = query_exif(&exif_repo).await;
    assert_eq!(exif_models.len(), 1, "exif metadata should be persisted");
    assert_eq!(
        exif_models[0].image_id, photo.id,
        "exif metadata must reference the photo"
    );
}
