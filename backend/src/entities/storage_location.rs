use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLocation {
    pub id: String,
    pub label: String,
    pub path: String,
    pub is_default: bool,
    pub created_at: String,
    #[serde(default = "StorageLocation::default_category_template")]
    pub category_template: String,
}

impl StorageLocation {
    pub fn normalized_path(&self) -> PathBuf {
        let path = PathBuf::from(&self.path);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    }

    fn default_category_template() -> String {
        "{year}/{date:%Y-%m-%d}/{fileName}".to_string()
    }
}
