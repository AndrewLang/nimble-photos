pub mod encrypt_service;
pub mod id_generation_service;

pub use encrypt_service::EncryptService;
pub use id_generation_service::IdGenerationService;

use std::sync::Arc;

use nimble_web::AppBuilder;
use nimble_web::config::Configuration;
use nimble_web::security::token::JwtTokenService;
use nimble_web::security::token::TokenService;

pub fn register_services(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.register_singleton(|provider| {
        let config = provider
            .resolve::<Configuration>()
            .expect("Configuration not found");
        EncryptService::new(&config).expect("Failed to create EncryptService")
    });

    builder.register_singleton(|_| IdGenerationService::new());

    builder.register_singleton(|provider| {
        let config = provider
            .resolve::<Configuration>()
            .expect("Configuration not found");
        let secret = config
            .get("jwt.secret")
            .unwrap_or("super-secret-key-123")
            .to_string();
        let issuer = config.get("jwt.issuer").unwrap_or("nimble").to_string();

        let service = JwtTokenService::new(secret, issuer);
        Arc::new(service) as Arc<dyn TokenService>
    });

    builder
}
