use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use nimble_web::config::Configuration;
use nimble_web::data::memory_repository::MemoryRepository;
use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataProvider, DataResult};
use nimble_web::data::query::{Query, Value};
use nimble_web::data::repository::Repository;
use nimble_web::security::token::{JwtTokenService, TokenService};
use uuid::Uuid;

use nimble_photos::entities::{user::User, user_settings::UserSettings};
use nimble_photos::services::{AuthService, EncryptService};

const TEST_USER_ID_STR: &str = "00000000-0000-0000-0000-000000000002";

#[derive(Clone)]
struct InMemoryUserProvider {
    store: Arc<Mutex<HashMap<Uuid, User>>>,
}

impl InMemoryUserProvider {
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DataProvider<User> for InMemoryUserProvider {
    async fn create(&self, e: User) -> DataResult<User> {
        self.store.lock().unwrap().insert(e.id, e.clone());
        Ok(e)
    }

    async fn get(&self, id: &Uuid) -> DataResult<Option<User>> {
        Ok(self.store.lock().unwrap().get(id).cloned())
    }

    async fn update(&self, e: User) -> DataResult<User> {
        self.store.lock().unwrap().insert(e.id, e.clone());
        Ok(e)
    }

    async fn delete(&self, id: &Uuid) -> DataResult<bool> {
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

fn create_test_config() -> Configuration {
    let key = vec![0u8; 32];
    let mut values = HashMap::new();
    let val = STANDARD.encode(&key);
    values.insert("encryption.key".to_string(), val.clone());
    values.insert("Encryption.Key".to_string(), val.clone());
    values.insert("jwt.secret".to_string(), "test-secret".to_string());
    values.insert("jwt.issuer".to_string(), "test-issuer".to_string());
    Configuration::from_values(values)
}

fn create_auth_service() -> AuthService {
    let config = create_test_config();
    println!("Config created with keys: {:?}", config.clone());

    // Explicitly panic with message if fails
    let encrypt = EncryptService::new(&config).unwrap_or_else(|e| {
        panic!("EncryptService creation failed: {:?}", e);
    });

    let token_service = JwtTokenService::new("test-secret".to_string(), "test-issuer".to_string());
    let tokens = Arc::new(token_service) as Arc<dyn TokenService>;
    let memory_repo = InMemoryUserProvider::new();
    let repo = Repository::new(Box::new(memory_repo));

    let settings_repo = MemoryRepository::<UserSettings>::new();
    let settings_repository = Repository::new(Box::new(settings_repo));

    AuthService::new(Arc::new(repo), Arc::new(settings_repository), encrypt, tokens)
}

#[test]
fn simple_config_test() {
    let config = create_test_config();
    let _encrypt = EncryptService::new(&config).unwrap();
}

#[tokio::test]
async fn register_creates_user_and_returns_tokens() {
    let service = create_auth_service();
    let email = "test@example.com";
    let password = "password123";

    let result = service.register(email, password, "Test User").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.access_token.is_empty());
    assert!(!response.refresh_token.is_empty());
}

#[tokio::test]
async fn login_with_valid_credentials_returns_tokens() {
    let service = create_auth_service();
    let email = "test@example.com";
    let password = "password123";

    service
        .register(email, password, "Test User")
        .await
        .unwrap();

    let result = service.login(email, password).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.access_token.is_empty());
    assert!(!response.refresh_token.is_empty());
}

#[tokio::test]
async fn login_with_invalid_email_returns_error() {
    let service = create_auth_service();
    let email = "test@example.com";
    let password = "password123";

    service
        .register(email, password, "Test User")
        .await
        .unwrap();

    let result = service.login("wrong@example.com", password).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn login_with_invalid_password_returns_error() {
    let service = create_auth_service();
    let email = "test@example.com";
    let password = "password123";

    service
        .register(email, password, "Test User")
        .await
        .unwrap();

    let result = service.login(email, "wrongpassword").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn refresh_with_valid_token_returns_new_tokens() {
    let service = create_auth_service();
    let email = "test@example.com";
    let password = "password123";

    let register_response = service
        .register(email, password, "Test User")
        .await
        .unwrap();

    let result = service.refresh(&register_response.refresh_token);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.access_token.is_empty());
    assert!(!response.refresh_token.is_empty());
}

#[tokio::test]
async fn refresh_with_invalid_token_returns_error() {
    let service = create_auth_service();

    let result = service.refresh("invalid-token");

    assert!(result.is_err());
}

#[tokio::test]
async fn me_returns_user_for_valid_user_id() {
    let service = create_auth_service();
    let email = "test@example.com";
    let password = "password123";

    service
        .register(email, password, "Test User")
        .await
        .unwrap();

    let config = create_test_config();
    let token_service = JwtTokenService::new("test-secret".to_string(), "test-issuer".to_string());
    let tokens = Arc::new(token_service) as Arc<dyn TokenService>;

    let memory_repo = InMemoryUserProvider::new();
    let repo = Repository::new(Box::new(memory_repo));

    let test_user_id = Uuid::parse_str(TEST_USER_ID_STR).unwrap();
    let user = User {
        id: test_user_id,
        email: email.to_string(),
        display_name: email.to_string(),
        password_hash: "hash".to_string(),
        created_at: chrono::Utc::now(),
        reset_token: None,
        reset_token_expires_at: None,
        verification_token: None,
        email_verified: false,
    };

    repo.insert(user.clone()).await.unwrap();

    let encrypt = EncryptService::new(&config).unwrap();
    let settings_repo = MemoryRepository::<UserSettings>::new();
    let settings_repository = Repository::new(Box::new(settings_repo));
    let service = AuthService::new(
        Arc::new(repo),
        Arc::new(settings_repository),
        encrypt,
        tokens,
    );

    let result = service.me(&user.id.to_string()).await;

    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert_eq!(returned_user.id, user.id);
    assert_eq!(returned_user.email, user.email);
}

#[tokio::test]
async fn me_returns_error_for_invalid_user_id() {
    let service = create_auth_service();

    let result = service.me("nonexistent-id").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn logout_succeeds() {
    let service = create_auth_service();
    let email = "test@example.com";
    let password = "password123";

    let register_response = service
        .register(email, password, "Test User")
        .await
        .unwrap();

    let result = service.logout(&register_response.refresh_token);

    assert!(result.is_ok());
}
