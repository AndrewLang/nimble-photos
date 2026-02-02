use crate::dtos::photo_dtos::TimelineGroup;
use crate::entities::photo::Photo;
use async_trait::async_trait;
use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataError, DataResult};
use serde_json;
use sqlx::PgPool;

#[async_trait]
pub trait PhotoRepository: Send + Sync {
    async fn get_timeline(&self, limit: u32) -> DataResult<Vec<TimelineGroup>>;
}

pub struct PostgresPhotoRepository {
    pool: PgPool,
}

impl PostgresPhotoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PhotoRepository for PostgresPhotoRepository {
    async fn get_timeline(&self, limit: u32) -> DataResult<Vec<TimelineGroup>> {
        // Group by day (YYYY-MM-DD)
        // We use COALESCE to fallback to created_at if date_taken is NULL
        // We order by day descending and take the first `limit` groups.

        let sql = r#"
            WITH photo_data AS (
                SELECT 
                    p.id, p.path, p.name, p.format, p.hash, p.size, p.created_at, p.updated_at, 
                    p.date_imported, p.date_taken, p.thumbnail_path, p.thumbnail_optimized, 
                    p.metadata_extracted, p.is_raw,
                    CASE 
                        WHEN e.orientation IN (5, 6, 7, 8) THEN
                            COALESCE(NULLIF(p.height, 0), NULLIF(e.pixel_y_dimension, 0), NULLIF(e.image_length, 0))
                        ELSE
                            COALESCE(NULLIF(p.width, 0), NULLIF(e.pixel_x_dimension, 0), NULLIF(e.image_width, 0))
                    END as width,
                    CASE 
                        WHEN e.orientation IN (5, 6, 7, 8) THEN
                            COALESCE(NULLIF(p.width, 0), NULLIF(e.pixel_x_dimension, 0), NULLIF(e.image_width, 0))
                        ELSE
                            COALESCE(NULLIF(p.height, 0), NULLIF(e.pixel_y_dimension, 0), NULLIF(e.image_length, 0))
                    END as height,
                    p.thumbnail_width, p.thumbnail_height
                FROM photos p
                LEFT JOIN exifs e ON p.id = e.image_id
            )
            SELECT 
                COALESCE(to_char(COALESCE(date_taken, created_at), 'YYYY-MM-DD'), 'Unknown') as day,
                count(*) as total_count,
                json_agg(pd.* ORDER BY COALESCE(date_taken, created_at) DESC) as photos_json
            FROM photo_data pd
            GROUP BY day
            ORDER BY day DESC
            LIMIT $1
        "#;

        let rows = sqlx::query(sql)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        let mut timeline = Vec::new();

        for row in rows {
            use sqlx::Row;
            let day: String = row
                .try_get("day")
                .map_err(|e| DataError::Provider(e.to_string()))?;
            let total_count: i64 = row
                .try_get("total_count")
                .map_err(|e| DataError::Provider(e.to_string()))?;
            let photos_json: serde_json::Value = row
                .try_get("photos_json")
                .map_err(|e| DataError::Provider(e.to_string()))?;

            let photos: Vec<Photo> = serde_json::from_value(photos_json)
                .map_err(|e| DataError::Provider(format!("Failed to deserialize photos: {}", e)))?;

            let count = total_count as u64;
            timeline.push(TimelineGroup {
                title: day,
                photos: Page::new(photos, count, 1, count as u32),
            });
        }

        Ok(timeline)
    }
}
