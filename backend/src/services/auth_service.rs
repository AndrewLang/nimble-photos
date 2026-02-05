use crate::dtos::auth_dtos::LoginResponse;
use crate::entities::user::User;
use crate::entities::user_settings::UserSettings;
use crate::services::EncryptService;
use chrono::{Duration, Utc};
use nimble_web::data::paging::PageRequest;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::Query;
use nimble_web::data::query::Value;
#[cfg(feature = "postgres")]
use nimble_web::data::query::FilterOperator;
#[cfg(feature = "postgres")]
use nimble_web::data::query_builder::QueryBuilder;
use nimble_web::data::repository::Repository;
use nimble_web::identity::claims::Claims;
use nimble_web::identity::user::UserIdentity;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::security::token::TokenService;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    repo: Arc<Repository<User>>,
    settings_repo: Arc<Repository<UserSettings>>,
    encrypt_service: EncryptService,
    tokens: Arc<dyn TokenService>,
}

impl AuthService {
    pub fn new(
        repo: Arc<Repository<User>>,
        settings_repo: Arc<Repository<UserSettings>>,
        encrypt_service: EncryptService,
        tokens: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            settings_repo,
            repo,
            encrypt_service,
            tokens,
        }
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<LoginResponse, PipelineError> {
        let is_first_user = self
            .repo
            .query({
                let mut query = Query::<User>::new();
                query.paging = Some(PageRequest::new(1, 1));
                query
            })
            .await
            .map(|page| page.items.is_empty())
            .map_err(|_| PipelineError::message("data error"))?;

        let password_hash = self
            .encrypt_service
            .encrypt(password)
            .map_err(|e| PipelineError::message(&e.to_string()))?;

        let email_string = email.to_string();
        let email_value = Value::String(email_string.clone());
        if let Some(_) = self
            .repo
            .get_by("email", email_value)
            .await
            .map_err(|_| PipelineError::message("data error"))?
        {
            return Err(PipelineError::message("email already registered"));
        }

        let display_name_value = display_name.to_string();

        let user = User {
            id: Uuid::new_v4(),
            email: email_string,
            display_name: display_name_value.clone(),
            password_hash,
            created_at: Utc::now(),
            reset_token: None,
            reset_token_expires_at: None,
            verification_token: Some(Uuid::new_v4().to_string()),
            email_verified: false,
            roles: if is_first_user {
                Some("admin".to_string())
            } else {
                None
            },
        };

        let user_id = user.id;

        self.repo.insert(user).await.map_err(|err| {
            log::error!("User insert failed: {:?}", err);
            PipelineError::message("Failed to create user")
        })?;

        let settings = UserSettings {
            user_id: user_id.to_string(),
            display_name: display_name_value,
            avatar_url: None,
            theme: "light".to_string(),
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            created_at: Utc::now(),
        };

        self.settings_repo.insert(settings).await.map_err(|err| {
            log::error!("User settings insert failed: {:?}", err);
            PipelineError::message("Failed to create user settings")
        })?;

        self.issue_tokens(user_id).await
    }

    pub async fn has_admin_user(&self) -> Result<bool, PipelineError> {
        #[cfg(feature = "postgres")]
        {
            let query = QueryBuilder::<User>::new()
                .filter(
                    "roles",
                    FilterOperator::Contains,
                    Value::String("admin".to_string()),
                )
                .page(1, 1)
                .build();

            let page = self
                .repo
                .query(query)
                .await
                .map_err(|_| PipelineError::message("data error"))?;

            return Ok(!page.items.is_empty());
        }

        #[cfg(not(feature = "postgres"))]
        {
            let page = self
                .repo
                .query(Query::<User>::new())
                .await
                .map_err(|_| PipelineError::message("data error"))?;

            let has_admin = page.items.iter().any(|user| {
                user.roles
                    .as_ref()
                    .map(|roles| roles.split(',').any(|role| role.trim() == "admin"))
                    .unwrap_or(false)
            });

            return Ok(has_admin);
        }
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
            .encrypt_service
            .verify(password, &user.password_hash)
            .map_err(|e| PipelineError::message(&e.to_string()))?
        {
            return Err(PipelineError::message("invalid credentials"));
        }

        self.issue_tokens(user.id).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<LoginResponse, PipelineError> {
        let user_id = self
            .tokens
            .validate_refresh_token(refresh_token)
            .map_err(|e| PipelineError::message(&e.to_string()))?;
        let user_id = Uuid::parse_str(&user_id)
            .map_err(|_| PipelineError::message("invalid refresh token subject"))?;
        self.issue_tokens(user_id).await
    }

    pub fn logout(&self, refresh_token: &str) -> Result<(), PipelineError> {
        self.tokens
            .revoke_refresh_token(refresh_token)
            .map_err(|e| PipelineError::message(&e.to_string()))
    }

    pub async fn me(&self, user_id: &str) -> Result<User, PipelineError> {
        let id = Uuid::parse_str(user_id).map_err(|_| PipelineError::message("invalid user id"))?;
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
        let id = Uuid::parse_str(user_id).map_err(|_| PipelineError::message("invalid user id"))?;
        let mut user = self
            .repo
            .get(&id)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("user not found"))?;

        if !self
            .encrypt_service
            .verify(old_pw, &user.password_hash)
            .map_err(|e| PipelineError::message(&e.to_string()))?
        {
            return Err(PipelineError::message("invalid credentials"));
        }

        let new_hash = self
            .encrypt_service
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
            if Utc::now() > expires_at {
                return Err(PipelineError::message("token expired"));
            }
        } else {
            return Err(PipelineError::message("invalid token"));
        }

        let new_hash = self
            .encrypt_service
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

    pub async fn issue_reset_token(&self, email: &str) -> Result<String, PipelineError> {
        let value = Value::String(email.to_string());
        let mut user = self
            .repo
            .get_by("email", value)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("user not found"))?;

        let token = Uuid::new_v4().to_string();
        user.reset_token = Some(token.clone());
        user.reset_token_expires_at = Some(Utc::now() + Duration::minutes(30));

        self.repo
            .update(user)
            .await
            .map_err(|_| PipelineError::message("failed to update user"))?;

        Ok(token)
    }

    pub async fn issue_verification_token(&self, email: &str) -> Result<String, PipelineError> {
        let value = Value::String(email.to_string());
        let user = self
            .repo
            .get_by("email", value)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("user not found"))?;

        user.verification_token
            .clone()
            .ok_or_else(|| PipelineError::message("verification token missing"))
    }

    async fn issue_tokens(&self, user_id: Uuid) -> Result<LoginResponse, PipelineError> {
        let user = self
            .repo
            .get(&user_id)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("user not found"))?;

        let user_id_str = user_id.to_string();
        let mut claims = Claims::new();

        if let Some(roles_str) = user.roles {
            for role in roles_str.split(',') {
                let role = role.trim();
                if !role.is_empty() {
                    claims = claims.add_role(role);
                }
            }
        }

        let identity = UserIdentity::new(user_id_str.clone(), claims);

        Ok(LoginResponse {
            access_token: self
                .tokens
                .create_access_token(&identity)
                .map_err(|e| PipelineError::message(&e.to_string()))?,
            refresh_token: self
                .tokens
                .create_refresh_token(&user_id_str)
                .map_err(|e| PipelineError::message(&e.to_string()))?,
        })
    }
}
