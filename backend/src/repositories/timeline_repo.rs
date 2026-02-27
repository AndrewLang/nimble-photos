use async_trait::async_trait;
use nimble_web::DataProvider;

use crate::dtos::timeline_dtos::TimelineYearDays;
use crate::entities::TimelineDay;

use nimble_web::PipelineError;
use nimble_web::QueryBuilder;
use nimble_web::Repository;

#[async_trait]
pub trait TimelineRepositoryExtensions {
    async fn get_years(&self) -> Result<Vec<i32>, PipelineError>;
    async fn get_yeardays(&self) -> Result<Vec<TimelineYearDays>, PipelineError>;
    async fn get_days(&self, limit: u32, offset: u32) -> Result<Vec<TimelineDay>, PipelineError>;
}

#[async_trait]
impl TimelineRepositoryExtensions for Repository<TimelineDay> {
    async fn get_years(&self) -> Result<Vec<i32>, PipelineError> {
        let query = QueryBuilder::new()
            .distinct_by("year")
            .sort_desc("year")
            .build();
        let days = self
            .all(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(days.into_iter().map(|row| row.year).collect())
    }

    async fn get_yeardays(&self) -> Result<Vec<TimelineYearDays>, PipelineError> {
        let query = QueryBuilder::new().sort_desc("day_date").build();
        let days = self
            .all(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        let mut result: Vec<TimelineYearDays> = Vec::new();
        let mut current_year: Option<i32> = None;

        for day in days {
            if current_year != Some(day.year) {
                result.push(TimelineYearDays {
                    year: day.year,
                    days: Vec::new(),
                });
                current_year = Some(day.year);
            }

            result.last_mut().unwrap().days.push(day.day_date);
        }

        Ok(result)
    }

    async fn get_days(&self, page: u32, page_size: u32) -> Result<Vec<TimelineDay>, PipelineError> {
        let query = QueryBuilder::new()
            .sort_desc("day_date")
            .page(page, page_size)
            .build();
        let days_page = self
            .query(query)
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(days_page.items)
    }
}
