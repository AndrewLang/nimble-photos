use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::photo_comment::PhotoComment;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoCommentDto {
    pub id: Uuid,
    pub photo_id: Uuid,
    pub user_id: Uuid,
    pub user_display_name: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl From<PhotoComment> for PhotoCommentDto {
    fn from(comment: PhotoComment) -> Self {
        Self {
            id: comment.id.unwrap_or_else(Uuid::new_v4),
            photo_id: comment.photo_id.unwrap_or_else(Uuid::new_v4),
            user_id: comment.user_id.unwrap_or_else(Uuid::new_v4),
            user_display_name: comment.user_display_name,
            body: comment.body.unwrap_or_default(),
            created_at: comment.created_at.unwrap_or_else(Utc::now),
        }
    }
}
