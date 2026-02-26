use crate::entities::photo::{Photo, PhotoViewModel};
use nimble_web::data::paging::Page;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum TagRef {
    Id(Uuid),
    Name(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineGroup {
    pub title: String,
    pub photos: Page<PhotoViewModel>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoGroup {
    pub day: String,
    pub total_count: i64,
    pub photos_payload: Vec<PhotoViewModel>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PhotoLoc {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub photo: Photo,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoWithTags {
    #[serde(flatten)]
    pub photo: Photo,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoLocWithTags {
    #[serde(flatten)]
    pub loc: PhotoLoc,
    pub tags: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileResponse {
    pub file_name: String,
    pub relative_path: String,
    pub byte_size: usize,
    pub content_type: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPhotosResponse {
    pub storage_id: String,
    pub storage_path: String,
    pub uploaded_count: usize,
    pub files: Vec<UploadFileResponse>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePhotosPayload {
    pub photo_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePhotoTagsPayload {
    pub photo_ids: Vec<String>,
    pub tags: Vec<String>,
}
