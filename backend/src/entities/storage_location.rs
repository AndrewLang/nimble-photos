use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ImageStorageLocation {
    pub id: String,
    pub label: String,
    pub path: PathBuf,
    pub created_at: String,
    pub category_policy: String,
}

impl ImageStorageLocation {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        path: impl Into<PathBuf>,
        created_at: impl Into<String>,
    ) -> Self {
        let path = Self::normalize_path(path.into());
        Self {
            id: id.into(),
            label: label.into(),
            path,
            created_at: created_at.into(),
            category_policy: "hash".to_string(),
        }
    }

    fn normalize_path(path: PathBuf) -> PathBuf {
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    }
}
