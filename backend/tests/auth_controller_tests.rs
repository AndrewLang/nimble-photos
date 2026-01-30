use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use nimble_photos::controllers::auth_controller::AuthController;
use nimble_photos::dtos::user_profile_dto::UserProfileDto;
use nimble_photos::entities::{user::User, user_settings::UserSettings};

use nimble_photos::services::{AuthService, EncryptService};
use nimble_web::data::memory_repository::MemoryRepository;
use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataProvider, DataResult};
use nimble_web::data::query::{Query, Value};
use nimble_web::data::repository::Repository;
use nimble_web::security::token::{JwtTokenService, TokenService};
use nimble_web::*;

#[derive(Clone)]
struct InMemoryUserProvider {
    store: Arc<Mutex<HashMap<String, User>>>,
}

impl InMemoryUserProvider {
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    fn seed(&self, entities: Vec<User>) {
        let mut store = self.store.lock().unwrap();
        for entity in entities {
            store.insert(entity.id.clone(), entity);
        }
    }
}

#[async_trait]
impl DataProvider<User> for InMemoryUserProvider {
    async fn create(&self, e: User) -> DataResult<User> {
        self.store.lock().unwrap().insert(e.id.clone(), e.clone());
        Ok(e)
    }
    async fn get(&self, id: &String) -> DataResult<Option<User>> {
        Ok(self.store.lock().unwrap().get(id).cloned())
    }
    async fn update(&self, e: User) -> DataResult<User> {
        self.store.lock().unwrap().insert(e.id.clone(), e.clone());
        Ok(e)
    }
    async fn delete(&self, id: &String) -> DataResult<bool> {
        Ok(self.store.lock().unwrap().remove(id).is_some())
    }
    async fn query(&self, _q: Query<User>) -> DataResult<Page<User>> {
        let store = self.store.lock().unwrap();
        let items: Vec<User> = store.values().cloned().collect();
        Ok(Page::new(items, 1, 1, 10))
    }
    async fn get_by(&self, column: &str, value: Value) -> DataResult<Option<User>> {
        if column == "email" {
            if let Value::String(email_val) = value {
                let store = self.store.lock().unwrap();
                for user in store.values() {
                    if user.email == email_val {
                        return Ok(Some(user.clone()));
                    }
                }
            }
        }
        Ok(None)
    }
}

use nimble_web::identity::claims::Claims;
use nimble_web::identity::user::UserIdentity;

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
        "{\"email\":\"me@example.com\",\"password\":\"x\"}".to_string(),
    ));

    let mut values = std::collections::HashMap::new();
    values.insert(
        "encryption.key".to_string(),
        "FMxHF3veLLoH25I7Hr9IOenHDKZwj6hcEYeQzTFww9s=".to_string(),
    );
    let config = Configuration::from_values(values);

    let encrypt_service = EncryptService::new(&config).unwrap();
    let encrypted_password = encrypt_service.encrypt("x").unwrap();

    let user_repo = InMemoryUserProvider::new();
    user_repo.seed(vec![User {
        id: "u1".to_string(),
        email: "me@example.com".to_string(),
        display_name: "Me".to_string(),
        password_hash: encrypted_password,
        created_at: chrono::Utc::now(),
        reset_token: None,
        reset_token_expires_at: None,
        verification_token: None,
        email_verified: false,
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
    container.register_singleton::<Arc<dyn TokenService>, _>(move |_| {
        let service = JwtTokenService::new("secret".to_string(), "issuer".to_string());
        Arc::new(service) as Arc<dyn TokenService>
    });
    container.register_singleton::<AuthService, _>(move |provider| {
        let repo = provider.resolve::<Repository<User>>().unwrap();
        let encrypt = provider.resolve::<EncryptService>().unwrap();
        let tokens = provider.resolve::<Arc<dyn TokenService>>().unwrap();
        AuthService::new(
            repo.clone(),
            encrypt.as_ref().clone(),
            tokens.as_ref().clone(),
        )
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

    match context.response().body() {
        nimble_web::http::response_body::ResponseBody::Text(json) => {
            let resp: serde_json::Value = serde_json::from_str(json).unwrap();
            assert!(resp.get("accessToken").is_some());
            assert!(resp.get("refreshToken").is_some());
        }
        _ => panic!("Unexpected body type"),
    }
}

#[test]
fn me_returns_profile_when_authenticated_and_repos_registered() {
    let mut registry = EndpointRegistry::new();
    registry.register::<AuthController>();

    let mut router = DefaultRouter::new();
    for route in registry.routes() {
        router.add_route(route.clone());
    }

    let user_repo = InMemoryUserProvider::new();
    user_repo.seed(vec![User {
        id: "u1".to_string(),
        email: "me@example.com".to_string(),
        display_name: "Me".to_string(),
        password_hash: "x".to_string(),
        created_at: chrono::Utc::now(),
        reset_token: None,
        reset_token_expires_at: None,
        verification_token: None,
        email_verified: false,
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

    // Add missing services
    let mut values = HashMap::new();
    values.insert(
        "encryption.key".to_string(),
        "FMxHF3veLLoH25I7Hr9IOenHDKZwj6hcEYeQzTFww9s=".to_string(),
    );
    values.insert(
        "Encryption.Key".to_string(),
        "FMxHF3veLLoH25I7Hr9IOenHDKZwj6hcEYeQzTFww9s=".to_string(),
    );
    let config_arc = Configuration::from_values(values);
    let config_clone = config_arc.clone();

    container.register_singleton::<Configuration, _>(move |_| config_clone.clone());

    container.register_singleton::<EncryptService, _>(move |provider| {
        let config = provider.resolve::<Configuration>().unwrap();
        EncryptService::new(&config).unwrap()
    });
    container.register_singleton::<Arc<dyn TokenService>, _>(move |_| {
        let service = JwtTokenService::new("secret".to_string(), "issuer".to_string());
        Arc::new(service) as Arc<dyn TokenService>
    });
    container.register_singleton::<AuthService, _>(move |provider| {
        let repo = provider.resolve::<Repository<User>>().unwrap();
        let encrypt = provider.resolve::<EncryptService>().unwrap();
        let tokens = provider.resolve::<Arc<dyn TokenService>>().unwrap();
        AuthService::new(
            repo.clone(), // already Arc
            encrypt.as_ref().clone(),
            tokens.as_ref().clone(),
        )
    });

    let services = container.build();

    let token_service = JwtTokenService::new("secret".to_string(), "issuer".to_string());
    let identity = UserIdentity::new("u1".to_string(), Claims::new());
    let token = TokenService::create_access_token(&token_service, &identity).unwrap();

    let mut request = HttpRequest::new("GET", "/api/auth/me");
    let header_val = format!("Bearer {}", token);
    request
        .headers_mut()
        .insert("authorization", header_val.as_str());

    let config = config_arc; // Use the same config
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
