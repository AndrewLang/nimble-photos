use crate::entities::photo::Photo;
use nimble_web::data::paging::Page;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineGroup {
    pub title: String,
    pub photos: Page<Photo>,
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
