use crate::prelude::*;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineYearDays {
    pub year: i32,
    pub days: Vec<NaiveDate>,
}
