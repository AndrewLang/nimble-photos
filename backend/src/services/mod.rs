pub mod admin_user_service;
pub mod auth_service;
pub mod encrypt_service;
pub mod exif_service;
pub mod hash_service;
pub mod id_generation_service;
pub mod photo_service;
pub mod setting_service;

pub use admin_user_service::AdminUserService;
pub use auth_service::AuthService;
pub use encrypt_service::EncryptService;
pub use exif_service::ExifService;
pub use hash_service::HashService;
pub use id_generation_service::IdGenerationService;
pub use photo_service::PhotoService;
pub use setting_service::SettingService;

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
