use crate::prelude::*;

#[cfg(feature = "postgres")]
use sqlx::FromRow;

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoTag {
    pub photo_id: Uuid,
    pub tag_id: Uuid,
}
