use crate::dtos::auth_dtos::LoginResponse;
use crate::entities::user::User;
use crate::services::EncryptService;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::Value;
use nimble_web::data::repository::Repository;
use nimble_web::identity::claims::Claims;
use nimble_web::identity::user::UserIdentity;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::security::token::TokenService;
use std::sync::Arc;

pub struct AuthService {
    repo: Arc<Repository<User>>,
    encrypt: EncryptService,
    tokens: Arc<dyn TokenService>,
}

impl AuthService {
    pub fn new(
        repo: Arc<Repository<User>>,
        encrypt: EncryptService,
        tokens: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            repo,
            encrypt,
            tokens,
        }
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, PipelineError> {
        let password_hash = self
            .encrypt
            .encrypt(password)
            .map_err(|e| PipelineError::message(&e.to_string()))?;

        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.to_string(),
            display_name: email.to_string(),
            password_hash,
            created_at: chrono::Utc::now(),
        };

        let user_id = user.id.clone();

        self.repo
            .insert(user)
            .await
            .map_err(|_| PipelineError::message("failed to create user"))?;

        self.issue_tokens(&user_id)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse, PipelineError> {
        let user = self
            .repo
            .get_by("email", Value::String(email.to_string()))
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("invalid credentials"))?;

        if !self
            .encrypt
            .verify(password, &user.password_hash)
            .map_err(|e| PipelineError::message(&e.to_string()))?
        {
            return Err(PipelineError::message("invalid credentials"));
        }

        self.issue_tokens(&user.id)
    }

    pub fn refresh(&self, refresh_token: &str) -> Result<LoginResponse, PipelineError> {
        let user_id = self
            .tokens
            .validate_refresh_token(refresh_token)
            .map_err(|e| PipelineError::message(&e.to_string()))?;
        self.issue_tokens(&user_id)
    }

    pub fn logout(&self, refresh_token: &str) -> Result<(), PipelineError> {
        self.tokens
            .revoke_refresh_token(refresh_token)
            .map_err(|e| PipelineError::message(&e.to_string()))
    }

    pub async fn me(&self, user_id: &str) -> Result<User, PipelineError> {
        self.repo
            .get(&user_id.to_string())
            .await
            .map_err(|e| PipelineError::message(&format!("data error: {:?}", e)))?
            .ok_or_else(|| PipelineError::message("user not found"))
    }

    fn issue_tokens(&self, user_id: &str) -> Result<LoginResponse, PipelineError> {
        let identity = UserIdentity::new(user_id.to_string(), Claims::new());

        Ok(LoginResponse {
            access_token: self
                .tokens
                .create_access_token(&identity)
                .map_err(|e| PipelineError::message(&e.to_string()))?,
            refresh_token: self
                .tokens
                .create_refresh_token(user_id)
                .map_err(|e| PipelineError::message(&e.to_string()))?,
        })
    }
}
