use chrono::Utc;
use image::{ImageBuffer, Rgb};
use nimble_photos::entities::StorageLocation;
use nimble_photos::entities::{exif::ExifModel, photo::Photo};
use nimble_photos::services::background_task_runner::BackgroundTaskRunner;
use nimble_photos::services::exif_service::ExifService;
use nimble_photos::services::file_service::FileService;
use nimble_photos::services::hash_service::HashService;
use nimble_photos::services::image_pipeline::{
    ImageProcessPayload, ImageProcessPipeline, ImageProcessPipelineContext,
};
use nimble_photos::services::photo_upload_service::StoredUploadFile;
use nimble_photos::services::{PreviewExtractor, ThumbnailExtractor};
use nimble_web::DataProvider;
use nimble_web::ServiceContainer;
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

fn create_storage(id: &str, label: &str, root: &Path) -> StorageLocation {
    StorageLocation {
        id: id.to_string(),
        label: label.to_string(),
        path: root.to_string_lossy().to_string(),
        is_default: false,
        created_at: Utc::now().to_rfc3339(),
        category_template: "hash".to_string(),
    }
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
    let thumbnail_root = storage_root.join("thumbnails");
    let preview_root = storage_root.join("previews");

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

    let storage = create_storage("storage-1", "Primary", &storage_root);

    let mut container = ServiceContainer::new();
    container.register_singleton::<BackgroundTaskRunner, _>(|_| BackgroundTaskRunner::new(2));
    container.register_singleton::<HashService, _>(|_| HashService::new());
    container.register_singleton::<ExifService, _>(|_| ExifService::new());
    container.register_singleton::<ThumbnailExtractor, _>(|_| ThumbnailExtractor::new());
    container.register_singleton::<PreviewExtractor, _>(|_| PreviewExtractor::new());
    container.register_singleton::<Repository<Photo>, _>(|_| Repository::new(Box::new(
        MemoryRepository::<Photo>::new(),
    )));
    container.register_singleton::<Repository<ExifModel>, _>(|_| Repository::new(Box::new(
        MemoryRepository::<ExifModel>::new(),
    )));
    container.register_singleton::<FileService, _>(|_| FileService::new());
    let provider = Arc::new(container.build());

    let pipeline = ImageProcessPipeline::new(ImageProcessPipelineContext::new(
        Arc::clone(&provider),
        test_configuration(&thumbnail_root, &preview_root),
    ));

    let request = ImageProcessPayload::from_upload(storage.clone(), stored_file.clone());
    pipeline
        .process(request)
        .await
        .expect("pipeline processing failed");

    assert!(
        !temp_file.exists(),
        "source file should be moved out of temp directory"
    );

    let photo_repo = provider.get::<Repository<Photo>>();
    let photos = query_photos(&photo_repo).await;
    assert_eq!(photos.len(), 1, "one photo should be persisted");
    let photo = &photos[0];
    assert!(photo.hash.is_some(), "hash should be persisted");
    assert!(
        photo.thumbnail_path.is_some(),
        "thumbnail path should be set"
    );
    assert_eq!(photo.size, Some(file_size as i64));

    let final_path = PathBuf::from(&photo.path);
    assert!(final_path.exists(), "final categorized file should exist");
    assert!(
        final_path
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value == file_name)
            .unwrap_or(false),
        "categorized file should keep original file name"
    );

    let thumbnail_path = PathBuf::from(photo.thumbnail_path.clone().unwrap());
    assert!(
        thumbnail_path.exists(),
        "thumbnail should be written to thumbnail root"
    );

    let exif_repo = provider.get::<Repository<ExifModel>>();
    let exif_models = query_exif(&exif_repo).await;
    assert_eq!(exif_models.len(), 1, "exif metadata should be persisted");
    assert_eq!(
        exif_models[0].image_id, photo.id,
        "exif metadata must reference the photo"
    );
}

#[test]
fn enqueue_uploaded_files_schedules_task_for_each_file() {
    let storage_root = std::env::temp_dir().join("pipeline-enqueue");
    let thumbnail_root = storage_root.join("thumbnails");
    let preview_root = storage_root.join("previews");

    let mut container = ServiceContainer::new();
    container.register_singleton::<BackgroundTaskRunner, _>(|_| BackgroundTaskRunner::new(2));
    container.register_singleton::<HashService, _>(|_| HashService::new());
    container.register_singleton::<ExifService, _>(|_| ExifService::new());
    container.register_singleton::<ThumbnailExtractor, _>(|_| ThumbnailExtractor::new());
    container.register_singleton::<PreviewExtractor, _>(|_| PreviewExtractor::new());
    container.register_singleton::<Repository<Photo>, _>(|_| Repository::new(Box::new(
        MemoryRepository::<Photo>::new(),
    )));
    container.register_singleton::<Repository<ExifModel>, _>(|_| Repository::new(Box::new(
        MemoryRepository::<ExifModel>::new(),
    )));
    container.register_singleton::<FileService, _>(|_| FileService::new());
    let provider = Arc::new(container.build());

    let pipeline = ImageProcessPipeline::new(ImageProcessPipelineContext::new(
        Arc::clone(&provider),
        test_configuration(&thumbnail_root, &preview_root),
    ));

    let storage = create_storage("enqueue-storage", "Enqueue", &storage_root);
    let files = vec![
        StoredUploadFile {
            file_name: "first.jpg".to_string(),
            relative_path: "temp/first.jpg".to_string(),
            byte_size: 512,
            content_type: Some("image/jpeg".to_string()),
        },
        StoredUploadFile {
            file_name: "second.jpg".to_string(),
            relative_path: "temp/second.jpg".to_string(),
            byte_size: 1024,
            content_type: Some("image/jpeg".to_string()),
        },
    ];
    let file_count = files.len();

    pipeline
        .enqueue_files(storage, files)
        .expect("enqueue should succeed");

    let runner = provider.get::<BackgroundTaskRunner>();
    assert_eq!(runner.queued_count(), file_count);
}
