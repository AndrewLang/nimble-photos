use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Deserialize;
use std::path::Path;
use uuid::Uuid;

use crate::dtos::photo_dtos::{PhotoGroup, PhotoLoc, TimelineGroup};
use crate::entities::AlbumPhoto;
use crate::entities::ExifModel;
use crate::entities::Photo;
use crate::entities::PhotoComment;
use crate::entities::PhotoViewModel;
use crate::entities::storage_location::StorageLocation;
use crate::models::setting_consts::SettingConsts;
use crate::services::FileService;

use nimble_web::Page;
use nimble_web::PipelineError;
use nimble_web::QueryBuilder;
use nimble_web::Repository;
use nimble_web::data::query::{FilterOperator, Value};
use nimble_web::{DataProvider, HttpContext};

#[async_trait]
pub trait PhotoRepositoryExtensions {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Photo>, PipelineError>;

    async fn photos_in_album(
        &self,
        album_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Photo>, PipelineError>;

    async fn delete_photo(
        &self,
        context: &HttpContext,
        photo: &Photo,
    ) -> Result<u32, PipelineError>;

    async fn delete_file(&self, photo: &Photo, context: &HttpContext) -> Result<(), PipelineError>;

    async fn delete_records(
        &self,
        photo: &Photo,
        context: &HttpContext,
    ) -> Result<(), PipelineError>;

    async fn get_years(&self) -> Result<Vec<String>, PipelineError>;

    async fn get_year_offset(&self, year: &str) -> Result<u32, PipelineError>;

    async fn photos_with_gps(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PhotoLoc>, PipelineError>;

    async fn photos_for_days(&self, days: Vec<String>)
    -> Result<Vec<TimelineGroup>, PipelineError>;

    async fn build_timeline(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<TimelineGroup>, PipelineError>;
}

#[async_trait]
impl PhotoRepositoryExtensions for Repository<Photo> {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Photo>, PipelineError> {
        self.get_by("hash", Value::String(hash.to_string()))
            .await
            .map_err(|_| PipelineError::message("failed to load photo by hash"))
    }

    async fn photos_in_album(
        &self,
        album_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Photo>, PipelineError> {
        let query = QueryBuilder::<Photo>::new()
            .join::<AlbumPhoto>("photo_id", "id")
            .filter("album_id", FilterOperator::Eq, Value::Uuid(album_id))
            .page(page, page_size)
            .build();

        self.query(query)
            .await
            .map_err(|_| PipelineError::message("failed to load photos in album"))
    }

    async fn delete_photo(
        &self,
        context: &HttpContext,
        photo: &Photo,
    ) -> Result<u32, PipelineError> {
        self.delete_file(photo, context).await?;
        self.delete_records(photo, context).await?;

        Ok(1)
    }

    async fn delete_records(
        &self,
        photo: &Photo,
        context: &HttpContext,
    ) -> Result<(), PipelineError> {
        let photo_repo = context.service::<Repository<Photo>>()?;
        let album_photo_repo = context.service::<Repository<AlbumPhoto>>()?;
        let exif_repo = context.service::<Repository<ExifModel>>()?;
        let photo_comment_repo = context.service::<Repository<PhotoComment>>()?;

        photo_repo.delete(&photo.id).await.map_err(|e| {
            PipelineError::message(&format!("failed to delete photo record: {:?}", e))
        })?;
        exif_repo
            .delete_by("image_id", Value::Uuid(photo.id))
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to delete exif record: {:?}", e))
            })?;
        photo_comment_repo
            .delete_by("photo_id", Value::Uuid(photo.id))
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to delete photo comments: {:?}", e))
            })?;
        album_photo_repo
            .delete_by("photo_id", Value::Uuid(photo.id))
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to delete album_photo records: {:?}", e))
            })?;

        Ok(())
    }

    async fn delete_file(&self, photo: &Photo, context: &HttpContext) -> Result<(), PipelineError> {
        let file_service = context.service::<FileService>()?;
        let storage_repo = context.service::<Repository<StorageLocation>>()?;
        let hash = photo
            .hash
            .as_ref()
            .ok_or_else(|| PipelineError::message("Photo hash is missing"))?;

        let storage = storage_repo
            .get(&photo.storage_id)
            .await
            .map_err(|_| PipelineError::message("Storage location not found"))?
            .ok_or_else(|| PipelineError::message("Storage is not found"))?;

        let root = Path::new(&storage.path);

        let thumbnail_path = file_service.path_for_hash(
            root.join(SettingConsts::THUMBNAIL_FOLDER),
            &hash,
            SettingConsts::THUMBNAIL_FORMAT,
        );
        let _ = file_service.remove_file(&thumbnail_path);

        let preview_path = file_service.path_for_hash(
            root.join(SettingConsts::PREVIEW_FOLDER),
            &hash,
            SettingConsts::PREVIEW_FORMAT,
        );
        let _ = file_service.remove_file(&preview_path);

        Ok(())
    }

    async fn get_years(&self) -> Result<Vec<String>, PipelineError> {
        #[derive(Deserialize)]
        struct YearRow {
            year: String,
        }

        let sql = format!(
            r#"
            SELECT DISTINCT to_char(p.day_date, 'YYYY') as year
            FROM photos p
            ORDER BY year DESC
        "#
        );

        let rows = self
            .raw_query::<YearRow>(&sql, &[])
            .await
            .map_err(|e| PipelineError::message(&format!("failed to load years: {:?}", e)))?;

        Ok(rows.into_iter().map(|row| row.year).collect())
    }

    async fn get_year_offset(&self, year: &str) -> Result<u32, PipelineError> {
        #[derive(Deserialize)]
        struct OffsetRow {
            offset: i64,
        }

        let sql = format!(
            r#"
            WITH day_groups AS (
                SELECT DISTINCT to_char(p.day_date, 'YYYY-MM-DD') as day
                FROM photos p
            )
            SELECT count(*) as offset
            FROM day_groups
            WHERE day > $1
        "#
        );

        let search_start = format!("{}-12-31", year);
        let rows = self
            .raw_query::<OffsetRow>(&sql, &[Value::String(search_start)])
            .await
            .map_err(|e| PipelineError::message(&format!("failed to load year offset: {:?}", e)))?;
        let offset = rows.first().map(|row| row.offset).unwrap_or(0);
        Ok(offset.max(0) as u32)
    }

    async fn photos_with_gps(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PhotoLoc>, PipelineError> {
        let sql = format!(
            r#"
            SELECT
                p.id, p.storage_id, p.path, p.name, p.format, p.hash, p.size,
                p.created_at, p.updated_at, p.date_imported, p.date_taken,
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
                p.day_date,
                p.sort_date,
                e.gps_latitude as lat,
                e.gps_longitude as lon
            FROM photos p
            JOIN exifs e ON p.id = e.image_id
            WHERE
                e.gps_latitude IS NOT NULL
                AND e.gps_longitude IS NOT NULL
                AND e.gps_latitude <> 0
                AND e.gps_longitude <> 0
            ORDER BY p.sort_date DESC
            LIMIT $1 OFFSET $2
        "#
        );

        let rows = self
            .raw_query::<PhotoLoc>(&sql, &[Value::Int(limit as i64), Value::Int(offset as i64)])
            .await
            .map_err(|e| {
                PipelineError::message(&format!("failed to load photos with GPS: {:?}", e))
            })?;

        Ok(rows)
    }

    async fn build_timeline(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<TimelineGroup>, PipelineError> {
        let sql = format!(
            r#"
            WITH target_days AS (
                SELECT DISTINCT
                    p.day_date
                FROM photos p
                ORDER BY p.day_date DESC
                LIMIT $1 OFFSET $2
            )
            SELECT
                to_char(td.day_date, 'YYYY-MM-DD') AS day,
                p_agg.totalCount,
                p_agg.photosPayload
            FROM target_days td
            LEFT JOIN LATERAL (
                SELECT
                    count(*) AS totalCount,
                    json_agg(
                        json_build_object(
                            'id', dp.id,
                            'hash', COALESCE(dp.hash, ''),
                            'width', dp.width,
                            'height', dp.height,
                            'name', dp.name
                        )
                    ) AS photosPayload
                FROM (
                    SELECT p.id, p.hash, p.width, p.height, p.name
                    FROM photos p
                    WHERE p.day_date = td.day_date
                    ORDER BY p.sort_date DESC
                ) dp
            ) p_agg ON true
            ORDER BY td.day_date DESC;
        "#
        );

        let groups = self
            .raw_query::<PhotoGroup>(&sql, &[Value::Int(limit as i64), Value::Int(offset as i64)])
            .await
            .map_err(|e| PipelineError::message(&format!("failed to load timeline: {:?}", e)))?;

        let mut timeline = Vec::new();
        for group in groups {
            timeline.push(TimelineGroup {
                title: group.day,
                photos: Page::new(
                    group.photos_payload,
                    group.total_count as u64,
                    1,
                    group.total_count as u32,
                ),
            });
        }

        Ok(timeline)
    }

    async fn photos_for_days(
        &self,
        days: Vec<String>,
    ) -> Result<Vec<TimelineGroup>, PipelineError> {
        log::info!("Loading photos for days: {:?}", days.clone());
        let day_dates: Vec<NaiveDate> = days
            .iter()
            .map(|d| {
                NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|e| PipelineError::message(&format!("invalid day '{}': {}", d, e)))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let query = QueryBuilder::<Photo>::new()
            .filter(
                "day_date",
                FilterOperator::In,
                Value::List(day_dates.into_iter().map(Value::Date).collect()),
            )
            .sort_desc("sort_date")
            .build();
        log::info!("Query: {:?}", query);

        let photos = self.all(query).await.map_err(|e| {
            PipelineError::message(&format!("failed to load photos for days: {:?}", e))
        })?;

        let mut groups: Vec<TimelineGroup> = Vec::new();

        for day in days {
            let day_photos: Vec<Photo> = photos
                .iter()
                .filter(|p| p.day_date.format("%Y-%m-%d").to_string() == day)
                .cloned()
                .collect();
            let length = day_photos.len();

            let group = TimelineGroup {
                title: day.clone(),
                photos: Page::new(
                    day_photos
                        .into_iter()
                        .map(|p| PhotoViewModel {
                            id: p.id,
                            hash: p.hash.unwrap_or_default(),
                            width: p.width,
                            height: p.height,
                            name: p.name,
                        })
                        .collect(),
                    length as u64,
                    1,
                    length as u32,
                ),
            };
            groups.push(group);
        }

        Ok(groups)
    }
}
