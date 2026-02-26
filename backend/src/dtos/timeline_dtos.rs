use chrono::NaiveDate;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineYearDays {
    pub year: i32,
    pub days: Vec<NaiveDate>,
}
