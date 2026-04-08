use async_trait::async_trait;

use crate::prelude::*;

#[async_trait]
pub trait TimelineRepositoryExtensions {
    async fn get_years(&self) -> Result<Vec<i32>, PipelineError>;
    async fn get_yeardays(&self) -> Result<Vec<TimelineYearDays>, PipelineError>;
    async fn get_days(&self, limit: u32, offset: u32) -> Result<Vec<TimelineDay>, PipelineError>;
    async fn sync(&self) -> Result<(), PipelineError>;
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

    async fn sync(&self) -> Result<(), PipelineError> {
        self.execute("DELETE FROM timeline_days", &[])
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to clear timeline days: {:?}", e))
            })?;

        let sql = r#"
            INSERT INTO timeline_days (
                id,
                day_date,
                year,
                month,
                total_count,
                min_sort_date,
                max_sort_date
            )
            SELECT
                gen_random_uuid() AS id,
                p.day_date,
                EXTRACT(YEAR FROM p.day_date)::int,
                EXTRACT(MONTH FROM p.day_date)::int,
                COUNT(*)::int,
                MIN(p.sort_date),
                MAX(p.sort_date)
            FROM photos p
            WHERE p.day_date IS NOT NULL
            GROUP BY p.day_date
            ORDER BY p.day_date;
        "#;
        self.execute(sql, &[]).await.map_err(|e| {
            PipelineError::message(&format!("failed to sync timeline days: {:?}", e))
        })?;

        Ok(())
    }
}
