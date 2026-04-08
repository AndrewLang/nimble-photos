use crate::prelude::*;

struct TimelineYearsHandler;

#[async_trait]
#[get("/api/timeline/years")]
impl HttpHandler for TimelineYearsHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<TimelineDay>>()?;

        let years = repository
            .get_years()
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(years))
    }
}

struct TimelineYearDaysHandler;

#[async_trait]
#[get("/api/timeline/yeardays")]
impl HttpHandler for TimelineYearDaysHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<TimelineDay>>()?;

        let years = repository
            .get_yeardays()
            .await
            .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;

        Ok(ResponseValue::json(years))
    }
}

struct TimelineHandler;

#[async_trait]
#[get("/api/timeline/{page}/{pageSize}")]
impl HttpHandler for TimelineHandler {
    async fn invoke(&self, context: &mut HttpContext) -> Result<ResponseValue, PipelineError> {
        let repository = context.service::<Repository<TimelineDay>>()?;
        let photo_repository = context.service::<Repository<Photo>>()?;
        let page: u32 = context.page().unwrap_or(1);
        let page_size: u32 = context.page_size().unwrap_or(10);

        let days: Vec<String> = repository
            .get_days(page, page_size)
            .await?
            .into_iter()
            .map(|d| d.day_date.format("%Y-%m-%d").to_string())
            .collect();

        let groups = photo_repository.photos_for_days(days).await.map_err(|e| {
            PipelineError::message(&format!("failed to load photos for days: {:?}", e))
        })?;

        Ok(ResponseValue::json(groups))
    }
}
