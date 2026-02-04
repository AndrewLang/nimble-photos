use chrono::{DateTime, Utc};
use serde_json::{Value as JsonValue, json};
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
    pub fn new(repository: Arc<Repository<Setting>>) -> Self {
        Self {
            repository,
            definitions: build_definitions(),
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
                .and_then(|entry| parse_value(&entry.value))
                .unwrap_or_else(|| def.default_value.clone());

            let updated_at = entity
                .as_ref()
                .map(|entry| entry.updated_at)
                .unwrap_or_else(Utc::now);

            results.push(def.to_dto(current_value, updated_at));
        }

        Ok(results)
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

        let parsed_value = parse_value(&saved.value).unwrap_or_else(|| def.default_value.clone());

        Ok(def.to_dto(parsed_value, saved.updated_at))
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
