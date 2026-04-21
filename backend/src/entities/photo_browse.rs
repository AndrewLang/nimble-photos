use crate::prelude::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowseNodeType {
    Folders,
    Photos,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsePhoto {
    pub id: Uuid,
    pub storage_id: Uuid,
    pub file_name: String,
    pub name: String,
    pub format: Option<String>,
    pub hash: Option<String>,
    pub size: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub date_imported: Option<DateTime<Utc>>,
    pub date_taken: Option<DateTime<Utc>>,
    pub year: Option<i32>,
    pub month_day: Option<String>,
    pub metadata_extracted: Option<bool>,
    pub artist: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub exposure_time: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<f32>,
    pub label: Option<String>,
    pub rating: Option<u8>,
    pub flagged: Option<i8>,
    pub is_raw: Option<bool>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<u16>,
    pub day_date: NaiveDate,
    pub sort_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseResponse {
    pub node_type: BrowseNodeType,

    pub folders: Option<Vec<StorageFolder>>,
    pub photos: Option<Vec<BrowsePhoto>>,

    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseOptions {
    #[serde(default = "BrowseOptions::default_dimensions")]
    pub dimensions: Vec<BrowseDimension>,
    #[serde(default = "BrowseOptions::default_sort_direction")]
    pub sort_direction: SortDirection,
    #[serde(default = "BrowseOptions::default_date_format")]
    pub date_format: String,
}

impl Default for BrowseOptions {
    fn default() -> Self {
        Self {
            dimensions: Self::default_dimensions(),
            sort_direction: Self::default_sort_direction(),
            date_format: Self::default_date_format(),
        }
    }
}

impl BrowseOptions {
    fn default_dimensions() -> Vec<BrowseDimension> {
        vec![BrowseDimension::Year, BrowseDimension::Date]
    }

    fn default_sort_direction() -> SortDirection {
        SortDirection::Desc
    }

    fn default_date_format() -> String {
        "yyyy-MM-dd".to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageFolder {
    pub name: String,
    pub full_path: String,
    pub depth: usize,
    pub file_count: i64,
    pub has_children: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BrowseDimension {
    Year,
    Date,
    Month,
    Camera,
    Rating,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseRequest {
    pub path: Option<String>,
    pub page_size: Option<i64>,
    pub cursor: Option<String>,
}

impl BrowseRequest {
    pub fn path_segments(&self) -> anyhow::Result<Vec<String>> {
        let decoded_path = self
            .path
            .as_deref()
            .map(urlencoding::decode)
            .transpose()?
            .map(|value| value.into_owned())
            .unwrap_or_default();

        let segments: Vec<String> = decoded_path
            .as_str()
            .trim()
            .trim_matches('/')
            .split('/')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
            })
            .collect();

        if segments.iter().any(|s| s.contains("..")) {
            anyhow::bail!("Invalid path segment");
        }

        Ok(segments)
    }
}
