use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::album_comment::AlbumComment;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCommentDto {
    pub id: Uuid,
    pub album_id: Uuid,
    pub user_id: Uuid,
    pub user_display_name: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub hidden: bool,
}

impl From<AlbumComment> for AlbumCommentDto {
    fn from(comment: AlbumComment) -> Self {
        Self {
            id: comment.id.unwrap_or_else(Uuid::new_v4),
            album_id: comment.album_id.unwrap_or_else(Uuid::new_v4),
            user_id: comment.user_id.unwrap_or_else(Uuid::new_v4),
            user_display_name: comment.user_display_name,
            body: comment.body.unwrap_or_default(),
            created_at: comment.created_at.unwrap_or_else(Utc::now),
            hidden: comment.hidden,
        }
    }
}
