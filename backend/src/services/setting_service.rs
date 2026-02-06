use chrono::{DateTime, Utc};
use serde_json::{Value as JsonValue, json};
use std::collections::HashSet;
use std::sync::Arc;

use crate::dtos::dashboard_settings_dto::{SettingDto, SettingOptionDto, SettingSection};
use crate::entities::setting::{Setting, SettingValueType};

use nimble_web::DataProvider;
use nimble_web::data::repository::Repository;
use nimble_web::pipeline::pipeline::PipelineError;

pub struct SettingService {
    repository: Arc<Repository<Setting>>,
    definitions: Vec<SettingDefinition>,
}

impl SettingService {
    const ROLE_PERMISSIONS_KEY: &'static str = "security.rolePermissions";
    const ACTION_DASHBOARD_ACCESS: &'static str = "dashboard.access";
    const ACTION_SETTINGS_GENERAL_UPDATE: &'static str = "settings.general.update";
    const ACTION_PHOTOS_UPLOAD: &'static str = "photos.upload";
    const ACTION_COMMENTS_CREATE: &'static str = "comments.create";

    pub fn new(repository: Arc<Repository<Setting>>) -> Self {
        Self {
            repository,
            definitions: Self::build_definitions(),
        }
    }

    pub async fn list(&self) -> Result<Vec<SettingDto>, PipelineError> {
        self.ensure_defaults().await?;

        let mut results = Vec::new();
        for def in &self.definitions {
            let key = def.key.to_string();
            let entity = self.repository.get(&key).await.map_err(|e| {
                let msg = format!("Failed to load setting {}: {:?}", key, e);
                PipelineError::message(&msg)
            })?;

            let current_value = entity
                .as_ref()
                .and_then(|entry| Self::parse_value(&entry.value))
                .unwrap_or_else(|| def.default_value.clone());

            let updated_at = entity
                .as_ref()
                .map(|entry| entry.updated_at)
                .unwrap_or_else(Utc::now);

            results.push(def.to_dto(current_value, updated_at));
        }

        Ok(results)
    }

    pub async fn get(&self, key: &str) -> Result<SettingDto, PipelineError> {
        self.ensure_defaults().await?;

        let def = self
            .definitions
            .iter()
            .find(|d| d.key == key)
            .ok_or_else(|| PipelineError::message("Unknown setting"))?;

        let key_owned = def.key.to_string();
        let entity = self.repository.get(&key_owned).await.map_err(|e| {
            let msg = format!("Failed to load setting {}: {:?}", key_owned, e);
            PipelineError::message(&msg)
        })?;

        let current_value = entity
            .as_ref()
            .and_then(|entry| Self::parse_value(&entry.value))
            .unwrap_or_else(|| def.default_value.clone());

        let updated_at = entity
            .as_ref()
            .map(|entry| entry.updated_at)
            .unwrap_or_else(Utc::now);

        Ok(def.to_dto(current_value, updated_at))
    }

    pub async fn update(&self, key: &str, value: JsonValue) -> Result<SettingDto, PipelineError> {
        let def = self
            .definitions
            .iter()
            .find(|d| d.key == key)
            .ok_or_else(|| PipelineError::message("Unknown setting"))?;

        if !def.value_type.matches(&value) {
            return Err(PipelineError::message("Invalid value type for setting"));
        }

        let serialized = serde_json::to_string(&value).map_err(|err| {
            let msg = format!("Failed to serialize setting value: {err}");
            PipelineError::message(&msg)
        })?;

        let now = Utc::now();
        let key_owned = def.key.to_string();
        let existing = self.repository.get(&key_owned).await.map_err(|e| {
            let msg = format!("Failed to load setting {}: {:?}", key_owned, e);
            PipelineError::message(&msg)
        })?;

        let created_at = existing
            .as_ref()
            .map(|entry| entry.created_at)
            .unwrap_or(now);

        let entity = Setting {
            key: def.key.to_string(),
            value: serialized,
            value_type: def.value_type,
            group: def.group.to_string(),
            created_at,
            updated_at: now,
        };

        let saved = if existing.is_some() {
            self.repository.update(entity).await.map_err(|err| {
                let msg = format!("Failed to update setting: {:?}", err);
                PipelineError::message(&msg)
            })?
        } else {
            self.repository.insert(entity).await.map_err(|err| {
                let msg = format!("Failed to insert setting: {:?}", err);
                PipelineError::message(&msg)
            })?
        };

        let parsed_value = Self::parse_value(&saved.value).unwrap_or_else(|| def.default_value.clone());

        Ok(def.to_dto(parsed_value, saved.updated_at))
    }

    pub async fn is_site_public(&self) -> Result<bool, PipelineError> {
        self.get_bool_setting("site.public").await
    }

    pub async fn is_registration_allowed(&self) -> Result<bool, PipelineError> {
        self.get_bool_setting("site.allowRegistration").await
    }

    pub async fn is_photo_upload_enabled(&self) -> Result<bool, PipelineError> {
        self.get_bool_setting("photo.manage.uploadsEnabled").await
    }

    pub async fn can_access_dashboard(&self, roles: &HashSet<String>) -> Result<bool, PipelineError> {
        self.is_action_allowed(roles, Self::ACTION_DASHBOARD_ACCESS)
            .await
    }

    pub async fn can_upload_photos(&self, roles: &HashSet<String>) -> Result<bool, PipelineError> {
        self.is_action_allowed(roles, Self::ACTION_PHOTOS_UPLOAD).await
    }

    pub async fn can_create_comments(
        &self,
        roles: &HashSet<String>,
    ) -> Result<bool, PipelineError> {
        self.is_action_allowed(roles, Self::ACTION_COMMENTS_CREATE).await
    }

    pub async fn can_update_setting(
        &self,
        roles: &HashSet<String>,
        key: &str,
    ) -> Result<bool, PipelineError> {
        if roles.contains("admin") {
            return Ok(true);
        }

        let def = self.definitions.iter().find(|d| d.key == key);
        let Some(definition) = def else {
            return Ok(false);
        };

        if roles.contains("contributor") && definition.section == SettingSection::General {
            return self
                .is_action_allowed(roles, Self::ACTION_SETTINGS_GENERAL_UPDATE)
                .await;
        }

        Ok(false)
    }

    async fn get_bool_setting(&self, key: &str) -> Result<bool, PipelineError> {
        let owned_key = key.to_string();
        let entry = self.repository.get(&owned_key).await.map_err(|e| {
            let msg = format!("Failed to load setting {}: {:?}", owned_key, e);
            PipelineError::message(&msg)
        })?;

        if let Some(stored) = entry {
            if let Some(parsed) = Self::parse_value(&stored.value).and_then(|json| json.as_bool()) {
                return Ok(parsed);
            }
        }

        Ok(self.definition_default_bool(key))
    }

    fn definition_default_bool(&self, key: &str) -> bool {
        self.definitions
            .iter()
            .find(|def| def.key == key)
            .and_then(|def| def.default_value.as_bool())
            .unwrap_or(false)
    }

    async fn role_permissions_config(&self) -> Result<JsonValue, PipelineError> {
        let entry = self
            .repository
            .get(&Self::ROLE_PERMISSIONS_KEY.to_string())
            .await
            .map_err(|e| {
                let msg = format!(
                    "Failed to load setting {}: {:?}",
                    Self::ROLE_PERMISSIONS_KEY,
                    e
                );
                PipelineError::message(&msg)
            })?;

        if let Some(stored) = entry {
            if let Some(parsed) = Self::parse_value(&stored.value) {
                return Ok(parsed);
            }
        }

        Ok(self
            .definitions
            .iter()
            .find(|d| d.key == Self::ROLE_PERMISSIONS_KEY)
            .map(|d| d.default_value.clone())
            .unwrap_or_else(|| json!({})))
    }

    async fn is_action_allowed(
        &self,
        roles: &HashSet<String>,
        action: &str,
    ) -> Result<bool, PipelineError> {
        if roles.contains("admin") {
            return Ok(true);
        }

        let config = self.role_permissions_config().await?;
        for role in roles {
            if self.role_has_action(&config, role, action) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn role_has_action(&self, config: &JsonValue, role: &str, action: &str) -> bool {
        let Some(role_config) = config.get(role) else {
            return false;
        };

        if role_config
            .get("*")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return true;
        }

        role_config
            .get(action)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    async fn ensure_defaults(&self) -> Result<(), PipelineError> {
        for def in &self.definitions {
            let key = def.key.to_string();
            let exists = self.repository.get(&key).await.map_err(|e| {
                let msg = format!("Failed to verify setting {}: {:?}", key, e);
                PipelineError::message(&msg)
            })?;

            match exists {
                Some(entry) => {
                    if entry.group != def.group {
                        let updated = Setting {
                            key: entry.key.clone(),
                            value: entry.value.clone(),
                            value_type: entry.value_type,
                            group: def.group.to_string(),
                            created_at: entry.created_at,
                            updated_at: Utc::now(),
                        };

                        self.repository.update(updated).await.map_err(|err| {
                            let msg = format!("Failed to migrate setting {}: {:?}", def.key, err);
                            PipelineError::message(&msg)
                        })?;
                    }
                }
                None => {
                    let now = Utc::now();
                    let entity = Setting {
                        key: def.key.to_string(),
                        value: serde_json::to_string(&def.default_value).map_err(|err| {
                            let msg = format!("Failed to serialize default for {}: {err}", def.key);
                            PipelineError::message(&msg)
                        })?,
                        value_type: def.value_type,
                        group: def.group.to_string(),
                        created_at: now,
                        updated_at: now,
                    };

                    self.repository.insert(entity).await.map_err(|err| {
                        let msg = format!("Failed to store setting {}: {:?}", def.key, err);
                        PipelineError::message(&msg)
                    })?;
                }
            }
        }
        Ok(())
    }

    fn build_definitions() -> Vec<SettingDefinition> {
        vec![
            SettingDefinition {
                key: "site.title",
                label: "Site title",
                description: "Displayed in the header and shared links",
                section: SettingSection::General,
                group: SettingSection::General.slug(),
                value_type: SettingValueType::String,
                default_value: json!("Nimble Photos"),
                options: None,
            },
            SettingDefinition {
                key: "site.tagline",
                label: "Site tagline",
                description: "Short description below the logo",
                section: SettingSection::General,
                group: SettingSection::General.slug(),
                value_type: SettingValueType::String,
                default_value: json!("My photo stories"),
                options: None,
            },
            SettingDefinition {
                key: "site.logo",
                label: "Site logo",
                description: "URL for the brand logo shown in the header",
                section: SettingSection::General,
                group: SettingSection::General.slug(),
                value_type: SettingValueType::String,
                default_value: json!(""),
                options: None,
            },
            SettingDefinition {
                key: "site.public",
                label: "Public gallery",
                description: "When enabled visitors can browse and view photos without signing in. Otherwise only authenticated users can access the library.",
                section: SettingSection::General,
                group: SettingSection::General.slug(),
                value_type: SettingValueType::Boolean,
                default_value: json!(true),
                options: None,
            },
            SettingDefinition {
                key: "site.allowRegistration",
                label: "Allow registration",
                description: "When enabled visitors can create their own accounts from the register screen.",
                section: SettingSection::General,
                group: SettingSection::General.slug(),
                value_type: SettingValueType::Boolean,
                default_value: json!(true),
                options: None,
            },
            SettingDefinition {
                key: "site.allowComments",
                label: "Allow comments",
                description: "When enabled users can add comments on photo and album detail views.",
                section: SettingSection::General,
                group: SettingSection::General.slug(),
                value_type: SettingValueType::Boolean,
                default_value: json!(true),
                options: None,
            },
            SettingDefinition {
                key: "security.rolePermissions",
                label: "Role permissions",
                description: "JSON map for role-based actions. Actions: dashboard.access, settings.general.update, photos.upload, comments.create.",
                section: SettingSection::Security,
                group: SettingSection::Security.slug(),
                value_type: SettingValueType::Json,
                default_value: json!({
                    "admin": { "*": true },
                    "contributor": {
                        "dashboard.access": true,
                        "settings.general.update": true,
                        "photos.upload": true,
                        "comments.create": true
                    },
                    "viewer": {
                        "dashboard.access": false,
                        "settings.general.update": false,
                        "photos.upload": false,
                        "comments.create": false
                    }
                }),
                options: None,
            },
            SettingDefinition {
                key: "storage.locations",
                label: "Storage locations",
                description: "Folders used to store uploaded photos and generated assets.",
                section: SettingSection::General,
                group: SettingSection::General.slug(),
                value_type: SettingValueType::Json,
                default_value: json!([]),
                options: None,
            },
            SettingDefinition {
                key: "photo.manage.uploadsEnabled",
                label: "Upload photos",
                description: "Allow uploads (including the scan endpoint) when authenticated members add new images.",
                section: SettingSection::PhotoManage,
                group: SettingSection::PhotoManage.slug(),
                value_type: SettingValueType::Boolean,
                default_value: json!(true),
                options: None,
            },
            SettingDefinition {
                key: "experience.gridColumns",
                label: "Gallery columns",
                description: "Columns used for the main gallery grid",
                section: SettingSection::Experience,
                group: SettingSection::Experience.slug(),
                value_type: SettingValueType::Number,
                default_value: json!(3),
                options: None,
            },
            SettingDefinition {
                key: "experience.defaultView",
                label: "Default landing view",
                description: "View presented to visitors by default",
                section: SettingSection::Experience,
                group: SettingSection::Experience.slug(),
                value_type: SettingValueType::String,
                default_value: json!("timeline"),
                options: Some(vec![
                    SettingOption {
                        label: "Timeline",
                        value: json!("timeline"),
                    },
                    SettingOption {
                        label: "Gallery",
                        value: json!("gallery"),
                    },
                    SettingOption {
                        label: "Map",
                        value: json!("map"),
                    },
                ]),
            },
            SettingDefinition {
                key: "experience.tipsEnabled",
                label: "Show tips",
                description: "Offer contextual tips across the app",
                section: SettingSection::Experience,
                group: SettingSection::Experience.slug(),
                value_type: SettingValueType::Boolean,
                default_value: json!(true),
                options: None,
            },
            SettingDefinition {
                key: "notifications.emailSummary",
                label: "Email summaries",
                description: "Send a weekly recap of new photos",
                section: SettingSection::Notifications,
                group: SettingSection::Notifications.slug(),
                value_type: SettingValueType::Boolean,
                default_value: json!(false),
                options: None,
            },
            SettingDefinition {
                key: "notifications.dailyDigestHour",
                label: "Daily digest hour",
                description: "UTC hour for the digest email",
                section: SettingSection::Notifications,
                group: SettingSection::Notifications.slug(),
                value_type: SettingValueType::Number,
                default_value: json!(18),
                options: None,
            },
        ]
    }

    fn parse_value(raw: &str) -> Option<JsonValue> {
        serde_json::from_str(raw).ok()
    }
}

#[derive(Clone)]
struct SettingDefinition {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    section: SettingSection,
    group: &'static str,
    value_type: SettingValueType,
    default_value: JsonValue,
    options: Option<Vec<SettingOption>>,
}

impl SettingDefinition {
    fn to_dto(&self, current_value: JsonValue, updated_at: DateTime<Utc>) -> SettingDto {
        SettingDto {
            key: self.key.to_string(),
            label: self.label.to_string(),
            description: self.description.to_string(),
            section: self.section,
            section_label: self.section.label().to_string(),
            group: self.group.to_string(),
            value_type: self.value_type,
            value: current_value,
            default_value: self.default_value.clone(),
            updated_at,
            options: self
                .options
                .as_ref()
                .map(|opts| opts.iter().map(|option| option.to_dto()).collect()),
        }
    }
}

#[derive(Clone)]
struct SettingOption {
    label: &'static str,
    value: JsonValue,
}

impl SettingOption {
    fn to_dto(&self) -> SettingOptionDto {
        SettingOptionDto {
            label: self.label.to_string(),
            value: self.value.clone(),
        }
    }
}
