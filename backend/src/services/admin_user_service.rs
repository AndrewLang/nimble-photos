use std::sync::Arc;
use uuid::Uuid;

use nimble_web::data::provider::DataProvider;
use nimble_web::data::query::Query;
use nimble_web::data::repository::Repository;
use nimble_web::pipeline::pipeline::PipelineError;

use crate::dtos::admin_user_dto::AdminUserDto;
use crate::entities::user::User;

pub struct AdminUserService {
    repo: Arc<Repository<User>>,
}

impl AdminUserService {
    pub fn new(repo: Arc<Repository<User>>) -> Self {
        Self { repo }
    }

    pub async fn list_users(&self) -> Result<Vec<AdminUserDto>, PipelineError> {
        let page = self
            .repo
            .query(Query::<User>::new())
            .await
            .map_err(|_| PipelineError::message("data error"))?;

        let mut users = page.items;
        users.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(users.into_iter().map(AdminUserDto::from).collect())
    }

    pub async fn update_roles(
        &self,
        user_id: Uuid,
        incoming_roles: Vec<String>,
    ) -> Result<AdminUserDto, PipelineError> {
        let mut user = self
            .repo
            .get(&user_id)
            .await
            .map_err(|_| PipelineError::message("data error"))?
            .ok_or_else(|| PipelineError::message("user not found"))?;

        let normalized = Self::normalize_roles(incoming_roles);
        if normalized.is_empty() {
            return Err(PipelineError::message("At least one role is required"));
        }

        let existing_roles = Self::parse_roles(user.roles.as_deref());
        let removing_admin = existing_roles.iter().any(|role| role == "admin")
            && !normalized.iter().any(|role| role == "admin");
        if removing_admin && !self.has_other_admin(user_id).await? {
            return Err(PipelineError::message(
                "Cannot remove admin from the last admin user",
            ));
        }

        user.roles = Some(normalized.join(","));

        let updated = self
            .repo
            .update(user)
            .await
            .map_err(|_| PipelineError::message("failed to update user roles"))?;

        Ok(AdminUserDto::from(updated))
    }

    async fn has_other_admin(&self, user_id: Uuid) -> Result<bool, PipelineError> {
        let page = self
            .repo
            .query(Query::<User>::new())
            .await
            .map_err(|_| PipelineError::message("data error"))?;

        Ok(page.items.iter().any(|user| {
            user.id != user_id
                && Self::parse_roles(user.roles.as_deref())
                    .iter()
                    .any(|role| role == "admin")
        }))
    }

    fn parse_roles(raw: Option<&str>) -> Vec<String> {
        raw.unwrap_or_default()
            .split(',')
            .map(|role| role.trim())
            .filter(|role| !role.is_empty())
            .map(ToString::to_string)
            .collect()
    }

    fn normalize_roles(roles: Vec<String>) -> Vec<String> {
        let mut normalized: Vec<String> = Vec::new();
        for role in roles {
            let value = role.trim().to_ascii_lowercase();
            if value.is_empty() {
                continue;
            }
            if !value
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
            {
                continue;
            }
            if !normalized.contains(&value) {
                normalized.push(value);
            }
        }
        normalized
    }
}
