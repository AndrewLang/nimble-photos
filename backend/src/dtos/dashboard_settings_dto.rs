use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::entities::setting::SettingValueType;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum SettingSection {
    General,
    Experience,
    Notifications,
}

impl SettingSection {
    pub fn label(&self) -> &'static str {
        match self {
            SettingSection::General => "General",
            SettingSection::Experience => "Experience",
            SettingSection::Notifications => "Notifications",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            SettingSection::General => "general",
            SettingSection::Experience => "experience",
            SettingSection::Notifications => "notifications",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingOptionDto {
    pub label: String,
    pub value: JsonValue,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDto {
    pub key: String,
    pub label: String,
    pub description: String,
    pub section: SettingSection,
    pub section_label: String,
    pub group: String,
    pub value_type: SettingValueType,
    pub value: JsonValue,
    pub default_value: JsonValue,
    pub updated_at: DateTime<Utc>,
    pub options: Option<Vec<SettingOptionDto>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingPayload {
    pub value: JsonValue,
}
