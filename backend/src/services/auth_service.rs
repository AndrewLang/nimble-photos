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
            reset_token: None,
            reset_token_expires_at: None,
            verification_token: Some(uuid::Uuid::new_v4().to_string()),
            email_verified: false,
        };

        let user_id = user.id.clone();

        self.repo
            .insert(user)
            .await
            .map_err(|_| PipelineError::message("failed to create user"))?;

        self.issue_tokens(&user_id)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse, PipelineError> {
        let email_val = email.to_string();
        let value = Value::String(email_val);
        let user = self
            .repo
            .get_by("email", value)
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
        let id = user_id.to_string();
        self.repo
            .get(&id)
            .await
            .map_err(|e| PipelineError::message(&format!("data error: {:?}", e)))?
            .ok_or_else(|| PipelineError::message("user not found"))
    }

    pub async fn change_password(
        &self,
        user_id: &str,
        old_pw: &str,
        new_pw: &str,
    ) -> Result<(), PipelineError> {
        let id = user_id.to_string();
        let mut user = self
            .repo
            .get(&id)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("user not found"))?;

        if !self
            .encrypt
            .verify(old_pw, &user.password_hash)
            .map_err(|e| PipelineError::message(&e.to_string()))?
        {
            return Err(PipelineError::message("invalid credentials"));
        }

        let new_hash = self
            .encrypt
            .encrypt(new_pw)
            .map_err(|e| PipelineError::message(&e.to_string()))?;

        user.password_hash = new_hash;
        self.repo
            .update(user)
            .await
            .map_err(|_| PipelineError::message("failed to update user"))?;
        Ok(())
    }

    pub async fn reset_password(&self, token: &str, new_pw: &str) -> Result<(), PipelineError> {
        let token_val = token.to_string();
        let value = Value::String(token_val);
        let mut user = self
            .repo
            .get_by("reset_token", value)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("invalid token"))?;

        if let Some(expires_at) = user.reset_token_expires_at {
            if chrono::Utc::now() > expires_at {
                return Err(PipelineError::message("token expired"));
            }
        } else {
            return Err(PipelineError::message("invalid token"));
        }

        let new_hash = self
            .encrypt
            .encrypt(new_pw)
            .map_err(|e| PipelineError::message(&e.to_string()))?;

        user.password_hash = new_hash;
        user.reset_token = None;
        user.reset_token_expires_at = None;

        self.repo
            .update(user)
            .await
            .map_err(|_| PipelineError::message("failed to update user"))?;
        Ok(())
    }

    pub async fn verify_email(&self, token: &str) -> Result<(), PipelineError> {
        let token_val = token.to_string();
        let value = Value::String(token_val);
        let mut user = self
            .repo
            .get_by("verification_token", value)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("invalid token"))?;

        user.email_verified = true;
        user.verification_token = None;

        self.repo
            .update(user)
            .await
            .map_err(|_| PipelineError::message("failed to update user"))?;
        Ok(())
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
