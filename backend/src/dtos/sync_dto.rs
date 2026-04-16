use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SyncAssetKind {
    Image,
    Preview,
    Thumbnail,
}

impl Default for SyncAssetKind {
    fn default() -> Self {
        Self::Image
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMetadataRequest {
    pub storage_id: String,
    pub image_id: String,
    pub hash: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFileItem {
    pub storage_id: String,
    pub hash: String,
    pub image_id: Option<String>,
    #[serde(default)]
    pub asset_kind: SyncAssetKind,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncFileStream {
    pub item: SyncFileItem,
    pub body: RequestBody,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFileResponse {
    pub image_id: String,
    pub storage_id: String,
    pub hash: String,
    pub asset_kind: SyncAssetKind,
    pub file: UploadFileResponse,
}
