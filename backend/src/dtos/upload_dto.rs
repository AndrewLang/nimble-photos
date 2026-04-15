use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckFileItem {
    pub hash: String,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckFileRequest {
    pub storage_id: String,
    pub files: Vec<CheckFileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckFileResponse {
    pub missing_files: Vec<CheckFileItem>,
}

#[derive(Clone, Debug)]
pub struct SyncFileData {
    pub file_name: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFileItem {
    pub hash: String,
    pub file_name: String,
    pub file_size: u64,
    pub metadata: serde_json::Value,

    pub has_original: bool,
    pub has_thumbnail: bool,
    pub has_preview: bool,
}
