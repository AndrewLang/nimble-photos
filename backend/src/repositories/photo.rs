use crate::dtos::photo_dtos::{PhotoLoc, TimelineGroup};
use crate::entities::photo::Photo;
use async_trait::async_trait;
use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataError, DataResult};
use serde_json;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait PhotoRepository: Send + Sync {
    async fn get_timeline(&self, limit: u32, offset: u32) -> DataResult<Vec<TimelineGroup>>;
    async fn get_years(&self) -> DataResult<Vec<String>>;
    async fn get_year_offset(&self, year: &str) -> DataResult<u32>;
    async fn get_by_ids(&self, ids: &[Uuid]) -> DataResult<Vec<Photo>>;
    async fn get_with_gps(&self, limit: u32, offset: u32) -> DataResult<Vec<PhotoLoc>>;
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
    async fn get_timeline(&self, limit: u32, offset: u32) -> DataResult<Vec<TimelineGroup>> {
        let sql = r#"
            WITH target_days AS (
                SELECT DISTINCT DATE(COALESCE(date_taken, created_at) AT TIME ZONE 'UTC') as day_date
                FROM photos
                ORDER BY day_date DESC NULLS LAST
                LIMIT $1 OFFSET $2
            )
            SELECT
                COALESCE(to_char(td.day_date, 'YYYY-MM-DD'), 'xxxx') as day,
                p_agg.total_count,
                p_agg.photos_json
            FROM target_days td
            LEFT JOIN LATERAL (
                SELECT
                    count(*) as total_count,
                    json_agg(json_build_object(
                        'id', pd.id,
                        'path', pd.path,
                        'name', pd.name,
                        'format', pd.format,
                        'hash', pd.hash,
                        'size', pd.size,
                        'created_at', pd.created_at,
                        'updated_at', pd.updated_at,
                        'date_imported', pd.date_imported,
                        'date_taken', pd.date_taken,
                        'thumbnail_path', pd.thumbnail_path,
                        'thumbnail_optimized', pd.thumbnail_optimized,
                        'metadata_extracted', pd.metadata_extracted,
                        'is_raw', pd.is_raw,
                        'tags', CASE WHEN pd.tags IS NULL OR pd.tags = '' THEN '[]'::json ELSE pd.tags::json END,
                        'width', pd.width,
                        'height', pd.height,
                        'thumbnail_width', pd.thumbnail_width,
                        'thumbnail_height', pd.thumbnail_height
                    ) ORDER BY pd.sort_date DESC) as photos_json
                FROM (
                    SELECT
                        p.id, p.path, p.name, p.format, p.hash, p.size, p.created_at, p.updated_at,
                        p.date_imported, p.date_taken, p.thumbnail_path, p.thumbnail_optimized,
                        p.metadata_extracted, p.is_raw, p.tags,
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
                        p.thumbnail_width, p.thumbnail_height,
                        COALESCE(p.date_taken, p.created_at) as sort_date
                    FROM photos p
                    LEFT JOIN exifs e ON p.id = e.image_id
                    WHERE DATE(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC') = td.day_date
                ) pd
            ) p_agg ON true
            ORDER BY td.day_date DESC NULLS LAST
        "#;

        let rows = sqlx::query(sql)
            .bind(limit as i32)
            .bind(offset as i32)
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

    async fn get_years(&self) -> DataResult<Vec<String>> {
        let sql = r#"
            SELECT DISTINCT to_char(COALESCE(date_taken, created_at) AT TIME ZONE 'UTC', 'YYYY') as year
            FROM photos
            ORDER BY year DESC
        "#;

        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        let mut years = Vec::new();
        for row in rows {
            use sqlx::Row;
            let year: String = row.try_get("year").unwrap_or_else(|_| "xxxx".to_string());
            years.push(year);
        }

        Ok(years)
    }

    async fn get_year_offset(&self, year: &str) -> DataResult<u32> {
        let sql = r#"
            WITH day_groups AS (
            SELECT DISTINCT to_char(COALESCE(date_taken, created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD') as day
                FROM photos
            )
            SELECT count(*) as offset
            FROM day_groups
            WHERE day > $1
        "#;

        let search_start = format!("{}-12-31", year);
        let row = sqlx::query(sql)
            .bind(search_start)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        use sqlx::Row;
        let offset: i64 = row.try_get("offset").unwrap_or(0);
        Ok(offset as u32)
    }

    async fn get_by_ids(&self, ids: &[uuid::Uuid]) -> DataResult<Vec<Photo>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let sql = r#"
            SELECT
                p.id, p.path, p.name, p.format, p.hash, p.size, p.created_at, p.updated_at,
                p.date_imported, p.date_taken, p.thumbnail_path, p.thumbnail_optimized,
                p.metadata_extracted, p.is_raw, p.tags,
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
            FROM (SELECT * FROM unnest($1)) as ids(id)
            JOIN photos p ON p.id = ids.id
            LEFT JOIN exifs e ON p.id = e.image_id
            ORDER BY COALESCE(p.date_taken, p.created_at) DESC
        "#;

        let photos = sqlx::query_as::<_, Photo>(sql)
            .bind(ids)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        Ok(photos)
    }

    async fn get_with_gps(&self, limit: u32, offset: u32) -> DataResult<Vec<PhotoLoc>> {
        let sql = r#"
            SELECT
                p.id, p.path, p.name, p.format, p.hash, p.size, p.created_at, p.updated_at,
                p.date_imported, p.date_taken, p.thumbnail_path, p.thumbnail_optimized,
                p.metadata_extracted, p.is_raw, p.tags,

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

                p.thumbnail_width,
                p.thumbnail_height,

                e.gps_latitude as lat,
                e.gps_longitude as lon

            FROM photos p
            JOIN exifs e ON p.id = e.image_id

            WHERE
                e.gps_latitude IS NOT NULL
                AND e.gps_longitude IS NOT NULL
                AND e.gps_latitude <> 0
                AND e.gps_longitude <> 0

            ORDER BY COALESCE(p.date_taken, p.created_at) DESC
            LIMIT $1 OFFSET $2
        "#;

        let photos = sqlx::query_as::<_, PhotoLoc>(sql)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        Ok(photos)
    }
}
