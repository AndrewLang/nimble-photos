use std::sync::Arc;

use nimble_web::*;

use nimble_photos::controllers::auth_controller::AuthController;
use nimble_photos::dtos::user_profile_dto::UserProfileDto;
use nimble_photos::entities::{user::User, user_settings::UserSettings};

use nimble_photos::services::EncryptService;

#[test]
fn login_returns_token() {
    let mut registry = EndpointRegistry::new();
    registry.register::<AuthController>();

    let mut router = DefaultRouter::new();
    for route in registry.routes() {
        router.add_route(route.clone());
    }

    let mut request = HttpRequest::new("POST", "/api/auth/login");
    request.set_body(RequestBody::Text(
        "{\"id\":\"u1\",\"password\":\"x\"}".to_string(),
    ));

    let mut values = std::collections::HashMap::new();
    values.insert(
        "Encryption.Key".to_string(),
        "FMxHF3veLLoH25I7Hr9IOenHDKZwj6hcEYeQzTFww9s=".to_string(),
    );
    let config = Configuration::from_values(values);

    let encrypt_service = EncryptService::new(&config).unwrap();
    let encrypted_password = encrypt_service.encrypt("x").unwrap();

    let user_repo = MemoryRepository::<User>::new();
    user_repo.seed(vec![User {
        id: "u1".to_string(),
        email: "me@example.com".to_string(),
        display_name: "Me".to_string(),
        password_hash: encrypted_password,
        created_at: chrono::Utc::now(),
    }]);

    let mut container = ServiceContainer::new();
    let config_clone = config.clone();
    container.register_singleton::<Configuration, _>(move |_| config_clone.clone());
    container.register_singleton::<Repository<User>, _>(move |_| {
        Repository::new(Box::new(user_repo.clone()))
    });
    container.register_singleton::<EncryptService, _>(move |provider| {
        let config = provider.resolve::<Configuration>().unwrap();
        EncryptService::new(&config).unwrap()
    });

    let services = container.build();
    let mut context = HttpContext::new(request, services, config);

    let mut pipeline = Pipeline::new();
    pipeline.add(RoutingMiddleware::new(router));
    pipeline.add(ControllerInvokerMiddleware::new(Arc::new(registry)));
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);
    if context.response().status() != 200 {
        println!("Body: {:?}", context.response().body());
    }
    assert!(result.is_ok());
    assert_eq!(context.response().status(), 200);
    assert_eq!(
        context.response().body(),
        &nimble_web::http::response_body::ResponseBody::Text("{\"token\":\"u1\"}".to_string())
    );
}

#[test]
fn me_returns_profile_when_authenticated_and_repos_registered() {
    let mut registry = EndpointRegistry::new();
    registry.register::<AuthController>();

    let mut router = DefaultRouter::new();
    for route in registry.routes() {
        router.add_route(route.clone());
    }

    let user_repo = MemoryRepository::<User>::new();
    user_repo.seed(vec![User {
        id: "u1".to_string(),
        email: "me@example.com".to_string(),
        display_name: "Me".to_string(),
        password_hash: "x".to_string(),
        created_at: chrono::Utc::now(),
    }]);

    let settings_repo = MemoryRepository::<UserSettings>::new();
    settings_repo.seed(vec![UserSettings {
        user_id: "u1".to_string(),
        display_name: "Display Name".to_string(),
        avatar_url: None,
        theme: "dark".to_string(),
        language: "en".to_string(),
        timezone: "UTC".to_string(),
        created_at: chrono::Utc::now(),
    }]);

    let mut container = ServiceContainer::new();
    container.register_singleton::<Repository<User>, _>(move |_| {
        Repository::new(Box::new(user_repo.clone()))
    });
    container.register_singleton::<Repository<UserSettings>, _>(move |_| {
        Repository::new(Box::new(settings_repo.clone()))
    });

    let services = container.build();

    let mut request = HttpRequest::new("GET", "/api/auth/me");
    request.headers_mut().insert("authorization", "Bearer u1");

    let config = ConfigBuilder::new().build();
    let mut context = HttpContext::new(request, services, config);

    let mut pipeline = Pipeline::new();
    pipeline.add(RoutingMiddleware::new(router));
    pipeline.add(AuthenticationMiddleware::new());
    pipeline.add(AuthorizationMiddleware::new());
    pipeline.add(ControllerInvokerMiddleware::new(Arc::new(registry)));
    pipeline.add(EndpointExecutionMiddleware::new());

    let result = pipeline.run(&mut context);
    assert!(result.is_ok());
    assert_eq!(context.response().status(), 200);

    let expected = serde_json::to_string(&UserProfileDto {
        id: "u1".to_string(),
        email: "me@example.com".to_string(),
        display_name: "Display Name".to_string(),
        avatar_url: None,
        theme: "dark".to_string(),
        language: "en".to_string(),
        timezone: "UTC".to_string(),
    })
    .unwrap();

    assert_eq!(
        context.response().body(),
        &nimble_web::http::response_body::ResponseBody::Text(expected)
    );
}
