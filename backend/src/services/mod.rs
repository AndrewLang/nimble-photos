pub mod encrypt_service;
pub mod id_generation_service;

pub use encrypt_service::EncryptService;
pub use id_generation_service::IdGenerationService;

use nimble_web::AppBuilder;
use nimble_web::config::Configuration;

pub fn register_services(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.register_singleton(|provider| {
        let config = provider
            .resolve::<Configuration>()
            .expect("Configuration not found");
        EncryptService::new(&config).expect("Failed to create EncryptService")
    });

    builder.register_singleton(|_| IdGenerationService::new());

    builder
}
