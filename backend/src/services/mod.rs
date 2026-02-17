mod image_process_constants;
mod image_process_context;
mod image_process_step;

pub mod admin_user_service;
pub mod auth_service;
pub mod background_task_runner;
pub mod encrypt_service;
pub mod exif_service;
pub mod file_service;
pub mod hash_service;
pub mod id_generation_service;
pub mod image_categorizer;
pub mod image_pipeline;
pub mod image_process_steps;
pub mod photo_service;
pub mod photo_upload_service;
pub mod preview_extractor;
pub mod setting_service;
pub mod task_descriptor;
pub mod thumbnail_extractor;

pub use admin_user_service::AdminUserService;
pub use auth_service::AuthService;
pub use background_task_runner::BackgroundTaskRunner;
pub use encrypt_service::EncryptService;
pub use exif_service::ExifService;
pub use file_service::FileService;
pub use hash_service::HashService;
pub use id_generation_service::IdGenerationService;
pub use image_pipeline::ImageProcessPipeline;
pub use image_pipeline::ImageProcessPipelineContext;
pub use photo_service::PhotoService;
pub use photo_upload_service::PhotoUploadService;
pub use preview_extractor::PreviewExtractor;
pub use setting_service::SettingService;
pub use task_descriptor::TaskDescriptor;
pub use thumbnail_extractor::ThumbnailExtractor;

use std::sync::Arc;

use crate::entities::{setting::Setting, user::User, user_settings::UserSettings};
use nimble_web::AppBuilder;
use nimble_web::config::Configuration;
use nimble_web::data::repository::Repository;
use nimble_web::security::token::JwtTokenService;
use nimble_web::security::token::TokenService;

pub fn register_services(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.register_singleton(|provider| {
        let config = provider.get::<Configuration>();
        EncryptService::new(&config).expect("Failed to create EncryptService")
    });
    builder.register_singleton(|_| IdGenerationService::new());
    builder.register_singleton(|_| PhotoService::new());
    builder.register_singleton(|_| ExifService::new());
    builder.register_singleton(|_| HashService::new());
    builder.register_singleton(|_| FileService::new());
    builder.register_singleton(|_| PhotoUploadService::new());
    builder.register_singleton(|provider| {
        log::info!("Initializing BackgroundTaskRunner...");
        let configuration = provider.get::<Configuration>();
        let default_parallelism = std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(4);
        let configured_parallelism = configuration
            .get("background.parallelism")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(default_parallelism);
        let runner = BackgroundTaskRunner::new(configured_parallelism);
        runner
            .start()
            .expect("Failed to start background task runner");
        log::info!(
            "BackgroundTaskRunner started with parallelism: {}",
            configured_parallelism
        );
        runner
    });
    builder.register_singleton(|_| ThumbnailExtractor::new());
    builder.register_singleton(|_| PreviewExtractor::new());
    builder.register_singleton(|provider| {
        let configuration = provider.get::<Configuration>().as_ref().clone();
        ImageProcessPipeline::new(ImageProcessPipelineContext::new(
            Arc::clone(&provider),
            configuration,
        ))
    });
    builder.register_singleton(|provider| {
        let config = provider.get::<Configuration>();
        let secret = config
            .get("jwt.secret")
            .unwrap_or("super-secret-key-123")
            .to_string();
        let issuer = config.get("jwt.issuer").unwrap_or("nimble").to_string();

        let service = JwtTokenService::new(secret, issuer);
        Arc::new(service) as Arc<dyn TokenService>
    });
    builder.register_singleton(|provider| {
        let repo = provider.get::<Repository<User>>();
        let settings_repo = provider.get::<Repository<UserSettings>>();
        let encrypt = provider.get::<EncryptService>();
        let tokens = provider.get::<Arc<dyn TokenService>>();

        AuthService::new(
            repo,
            settings_repo,
            (*encrypt).clone(),
            tokens.as_ref().clone(),
        )
    });
    builder.register_singleton(|provider| {
        let settings_repo = provider.get::<Repository<Setting>>();
        SettingService::new(settings_repo)
    });
    builder.register_singleton(|provider| {
        let repo = provider.get::<Repository<User>>();
        AdminUserService::new(repo)
    });
    builder
}
