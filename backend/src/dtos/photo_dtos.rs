use crate::entities::photo::Photo;
use nimble_web::data::paging::Page;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineGroup {
    pub title: String,
    pub photos: Page<Photo>,
}
