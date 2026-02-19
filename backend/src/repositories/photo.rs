use crate::dtos::photo_dtos::{PhotoLoc, TimelineGroup};
use crate::entities::photo::{Photo, PhotoViewModel};
use crate::entities::tag::Tag;
use async_trait::async_trait;
use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataError, DataResult};
use serde_json;
use sqlx::FromRow;
use sqlx::PgPool;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

const TAG_VISIBILITY_ADMIN_ONLY: i16 = 1;

#[derive(Debug, Clone)]
pub enum TagRef {
    Id(i64),
    Name(String),
}

#[async_trait]
pub trait PhotoRepository: Send + Sync {
    async fn get_timeline(
        &self,
        limit: u32,
        offset: u32,
        is_admin: bool,
    ) -> DataResult<Vec<TimelineGroup>>;
    async fn get_years(&self, is_admin: bool) -> DataResult<Vec<String>>;
    async fn get_year_offset(&self, year: &str, is_admin: bool) -> DataResult<u32>;
    async fn get_by_ids(&self, ids: &[Uuid], is_admin: bool) -> DataResult<Vec<Photo>>;
    async fn get_with_gps(
        &self,
        limit: u32,
        offset: u32,
        is_admin: bool,
    ) -> DataResult<Vec<PhotoLoc>>;

    async fn list_all_tags(&self, is_admin: bool) -> DataResult<Vec<Tag>>;
    async fn get_tags_by_ids(&self, ids: &[i64], is_admin: bool) -> DataResult<Vec<Tag>>;
    async fn upsert_tag(&self, name: &str, visibility: Option<i16>) -> DataResult<Tag>;
    async fn set_photo_tags(
        &self,
        photo_id: Uuid,
        tag_refs: &[TagRef],
        _created_by_user_id: Option<Uuid>,
    ) -> DataResult<()>;
    async fn add_photo_tag(&self, photo_id: Uuid, tag_name: &str) -> DataResult<()>;
    async fn remove_photo_tag(&self, photo_id: Uuid, tag_name_or_id: &str) -> DataResult<bool>;
    async fn get_photo_tags(&self, photo_id: Uuid, is_admin: bool) -> DataResult<Vec<Tag>>;
    async fn get_photo_tag_name_map(
        &self,
        photo_ids: &[Uuid],
        is_admin: bool,
    ) -> DataResult<HashMap<Uuid, Vec<String>>>;
    async fn filter_photos_by_tags(
        &self,
        tag_names: &[String],
        match_all: bool,
        is_admin: bool,
        page: u32,
        page_size: u32,
    ) -> DataResult<Page<Photo>>;
    async fn get_photos_page(
        &self,
        page: u32,
        page_size: u32,
        is_admin: bool,
    ) -> DataResult<Page<Photo>>;

    async fn set_album_tags(
        &self,
        album_id: Uuid,
        tag_refs: &[TagRef],
        created_by_user_id: Option<Uuid>,
    ) -> DataResult<()>;
    async fn get_album_tags(&self, album_id: Uuid, is_admin: bool) -> DataResult<Vec<Tag>>;
}

pub struct PostgresPhotoRepository {
    pool: PgPool,
}

impl PostgresPhotoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PostgresPhotoRepository {
    fn photos_relation(is_admin: bool) -> &'static str {
        if is_admin {
            "photos"
        } else {
            "photos_public_visible"
        }
    }

    fn tag_visibility_filter(tag_alias: &str, is_admin_param: &str) -> String {
        format!(
            "({is_admin_param} OR {tag_alias}.visibility <> {admin_only})",
            admin_only = TAG_VISIBILITY_ADMIN_ONLY
        )
    }

    fn normalize_tag_name(raw: &str) -> Option<(String, String)> {
        let name = raw.trim();
        if name.is_empty() {
            return None;
        }
        Some((name.to_string(), name.to_lowercase()))
    }

    fn normalize_tag_names(raw_tags: &[String]) -> Vec<(String, String)> {
        let mut dedup = BTreeMap::<String, String>::new();
        for raw in raw_tags {
            if let Some((name, name_norm)) = Self::normalize_tag_name(raw) {
                dedup.entry(name_norm).or_insert(name);
            }
        }
        dedup.into_iter().map(|(norm, name)| (name, norm)).collect()
    }

    async fn resolve_tag_ids(
        &self,
        refs: &[TagRef],
        default_visibility: i16,
    ) -> DataResult<Vec<i64>> {
        let mut ids = Vec::<i64>::new();
        let mut names = Vec::<String>::new();

        for item in refs {
            match item {
                TagRef::Id(id) => ids.push(*id),
                TagRef::Name(name) => names.push(name.clone()),
            }
        }

        let normalized = Self::normalize_tag_names(&names);

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        for (name, name_norm) in normalized {
            let tag_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO tags (name, name_norm, visibility, created_at)
                VALUES ($1, $2, $3, NOW())
                ON CONFLICT (name_norm) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                "#,
            )
            .bind(name)
            .bind(name_norm)
            .bind(default_visibility)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;
            ids.push(tag_id);
        }

        tx.commit()
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        ids.sort_unstable();
        ids.dedup();
        Ok(ids)
    }
}

#[async_trait]
impl PhotoRepository for PostgresPhotoRepository {
    async fn get_timeline(
        &self,
        limit: u32,
        offset: u32,
        is_admin: bool,
    ) -> DataResult<Vec<TimelineGroup>> {
        let photos = Self::photos_relation(is_admin);
        let sql = format!(
            r#"
            WITH target_days AS (
                SELECT DISTINCT
                    DATE(date_taken AT TIME ZONE 'UTC') AS day_date
                FROM {photos}
                ORDER BY day_date DESC NULLS LAST
                LIMIT $1 OFFSET $2
            )
            SELECT
                to_char(td.day_date, 'YYYY-MM-DD') AS day,
                p_agg.total_count,
                p_agg.photos_json
            FROM target_days td
            LEFT JOIN LATERAL (
                SELECT
                    count(*) AS total_count,
                    json_agg(
                        json_build_object(
                            'id', p.id,
                            'hash', p.hash,
                            'width', p.width,
                            'height', p.height,
                            'name', p.name
                        )
                        ORDER BY COALESCE(p.date_taken, p.created_at) DESC
                    ) AS photos_json
                FROM {photos} p
                WHERE DATE(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC') = td.day_date
            ) p_agg ON true
            ORDER BY td.day_date DESC NULLS LAST;
        "#
        );

        let rows = sqlx::query(&sql)
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

            let photos: Vec<PhotoViewModel> = serde_json::from_value(photos_json)
                .map_err(|e| DataError::Provider(format!("Failed to deserialize photos: {}", e)))?;

            let count = total_count as u64;
            timeline.push(TimelineGroup {
                title: day,
                photos: Page::new(photos, count, 1, count as u32),
            });
        }

        Ok(timeline)
    }

    async fn get_years(&self, is_admin: bool) -> DataResult<Vec<String>> {
        let photos = Self::photos_relation(is_admin);
        let sql = format!(
            r#"
            SELECT DISTINCT to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY') as year
            FROM {photos} p
            ORDER BY year DESC
        "#
        );

        let rows = sqlx::query(&sql)
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

    async fn get_year_offset(&self, year: &str, is_admin: bool) -> DataResult<u32> {
        let photos = Self::photos_relation(is_admin);
        let sql = format!(
            r#"
            WITH day_groups AS (
                SELECT DISTINCT to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD') as day
                FROM {photos} p
            )
            SELECT count(*) as offset
            FROM day_groups
            WHERE day > $1
        "#
        );

        let search_start = format!("{}-12-31", year);
        let row = sqlx::query(&sql)
            .bind(search_start)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        use sqlx::Row;
        let offset: i64 = row.try_get("offset").unwrap_or(0);
        Ok(offset as u32)
    }

    async fn get_by_ids(&self, ids: &[uuid::Uuid], is_admin: bool) -> DataResult<Vec<Photo>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let photos = Self::photos_relation(is_admin);
        let sql = format!(
            r#"
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
            FROM (SELECT * FROM unnest($1)) as ids(id)
            JOIN {photos} p ON p.id = ids.id
            LEFT JOIN exifs e ON p.id = e.image_id
            ORDER BY COALESCE(p.date_taken, p.created_at) DESC
        "#
        );

        let photos = sqlx::query_as::<_, Photo>(&sql)
            .bind(ids)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        Ok(photos)
    }

    async fn get_with_gps(
        &self,
        limit: u32,
        offset: u32,
        is_admin: bool,
    ) -> DataResult<Vec<PhotoLoc>> {
        let photos = Self::photos_relation(is_admin);
        let sql = format!(
            r#"
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
                p.thumbnail_width,
                p.thumbnail_height,
                e.gps_latitude as lat,
                e.gps_longitude as lon
            FROM {photos} p
            JOIN exifs e ON p.id = e.image_id
            WHERE
                e.gps_latitude IS NOT NULL
                AND e.gps_longitude IS NOT NULL
                AND e.gps_latitude <> 0
                AND e.gps_longitude <> 0
            ORDER BY COALESCE(p.date_taken, p.created_at) DESC
            LIMIT $1 OFFSET $2
        "#
        );

        let photos = sqlx::query_as::<_, PhotoLoc>(&sql)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        Ok(photos)
    }

    async fn list_all_tags(&self, is_admin: bool) -> DataResult<Vec<Tag>> {
        let vis = Self::tag_visibility_filter("t", "$1::boolean");
        let sql = format!(
            r#"
            SELECT id, name, visibility, created_at
            FROM tags t
            WHERE {vis}
            ORDER BY name ASC
        "#
        );

        sqlx::query_as::<_, Tag>(&sql)
            .bind(is_admin)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))
    }

    async fn get_tags_by_ids(&self, ids: &[i64], is_admin: bool) -> DataResult<Vec<Tag>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let vis = Self::tag_visibility_filter("t", "$2::boolean");
        let sql = format!(
            r#"
            SELECT t.id, t.name, t.visibility, t.created_at
            FROM tags t
            WHERE t.id = ANY($1::bigint[])
              AND {vis}
            ORDER BY t.name ASC
        "#
        );

        sqlx::query_as::<_, Tag>(&sql)
            .bind(ids)
            .bind(is_admin)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))
    }

    async fn upsert_tag(&self, name: &str, visibility: Option<i16>) -> DataResult<Tag> {
        let (clean_name, name_norm) = Self::normalize_tag_name(name)
            .ok_or_else(|| DataError::Provider("tag name cannot be empty".to_string()))?;
        let visibility = visibility.unwrap_or(0);

        let sql = r#"
            INSERT INTO tags (name, name_norm, visibility, created_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (name_norm) DO UPDATE SET name = EXCLUDED.name, visibility = EXCLUDED.visibility
            RETURNING id, name, visibility, created_at
        "#;

        sqlx::query_as::<_, Tag>(sql)
            .bind(clean_name)
            .bind(name_norm)
            .bind(visibility)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))
    }

    async fn set_photo_tags(
        &self,
        photo_id: Uuid,
        tag_refs: &[TagRef],
        _created_by_user_id: Option<Uuid>,
    ) -> DataResult<()> {
        let ids = self.resolve_tag_ids(tag_refs, 0).await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        sqlx::query("DELETE FROM photo_tags WHERE photo_id = $1")
            .bind(photo_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        for tag_id in ids {
            sqlx::query(
                "INSERT INTO photo_tags (photo_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(photo_id)
            .bind(tag_id as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;
        Ok(())
    }

    async fn add_photo_tag(&self, photo_id: Uuid, tag_name: &str) -> DataResult<()> {
        let tag = self.upsert_tag(tag_name, None).await?;
        sqlx::query(
            "INSERT INTO photo_tags (photo_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(photo_id)
        .bind(tag.id)
        .execute(&self.pool)
        .await
        .map_err(|e| DataError::Provider(e.to_string()))?;
        Ok(())
    }

    async fn remove_photo_tag(&self, photo_id: Uuid, tag_name_or_id: &str) -> DataResult<bool> {
        let done = if let Ok(tag_id) = tag_name_or_id.trim().parse::<i64>() {
            let result = sqlx::query("DELETE FROM photo_tags WHERE photo_id = $1 AND tag_id = $2")
                .bind(photo_id)
                .bind(tag_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DataError::Provider(e.to_string()))?;
            result.rows_affected() > 0
        } else if let Some((_, name_norm)) = Self::normalize_tag_name(tag_name_or_id) {
            let result = sqlx::query(
                r#"
                DELETE FROM photo_tags pt
                USING tags t
                WHERE pt.photo_id = $1
                  AND pt.tag_id = t.id
                  AND t.name_norm = $2
                "#,
            )
            .bind(photo_id)
            .bind(name_norm)
            .execute(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;
            result.rows_affected() > 0
        } else {
            false
        };

        Ok(done)
    }

    async fn get_photo_tags(&self, photo_id: Uuid, is_admin: bool) -> DataResult<Vec<Tag>> {
        let vis = Self::tag_visibility_filter("t", "$2::boolean");
        let photos = Self::photos_relation(is_admin);
        let sql = format!(
            r#"
            SELECT t.id, t.name, t.visibility, t.created_at
            FROM {photos} p
            JOIN photo_tags pt ON pt.photo_id = p.id
            JOIN tags t ON t.id = pt.tag_id
            WHERE p.id = $1
              AND {vis}
            ORDER BY t.name ASC
        "#
        );

        sqlx::query_as::<_, Tag>(&sql)
            .bind(photo_id)
            .bind(is_admin)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))
    }

    async fn get_photo_tag_name_map(
        &self,
        photo_ids: &[Uuid],
        is_admin: bool,
    ) -> DataResult<HashMap<Uuid, Vec<String>>> {
        let mut map = HashMap::new();
        if photo_ids.is_empty() {
            return Ok(map);
        }

        let vis = Self::tag_visibility_filter("t", "$2::boolean");
        let sql = format!(
            r#"
            SELECT pt.photo_id, t.name
            FROM photo_tags pt
            JOIN tags t ON t.id = pt.tag_id
            WHERE pt.photo_id = ANY($1::uuid[])
              AND {vis}
            ORDER BY t.name ASC
        "#
        );

        let rows = sqlx::query(&sql)
            .bind(photo_ids)
            .bind(is_admin)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        for row in rows {
            use sqlx::Row;
            let photo_id: Uuid = row
                .try_get("photo_id")
                .map_err(|e| DataError::Provider(e.to_string()))?;
            let tag_name: String = row
                .try_get("name")
                .map_err(|e| DataError::Provider(e.to_string()))?;
            map.entry(photo_id)
                .or_insert_with(Vec::new)
                .push(tag_name);
        }

        Ok(map)
    }

    async fn filter_photos_by_tags(
        &self,
        tag_names: &[String],
        match_all: bool,
        is_admin: bool,
        page: u32,
        page_size: u32,
    ) -> DataResult<Page<Photo>> {
        let normalized = Self::normalize_tag_names(tag_names);
        let normalized_values: Vec<String> = normalized.into_iter().map(|(_, norm)| norm).collect();
        if match_all && !normalized_values.is_empty() {
            let existing_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE name_norm = ANY($1::text[])")
                    .bind(&normalized_values)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| DataError::Provider(e.to_string()))?;

            if existing_count < normalized_values.len() as i64 {
                return Ok(Page::new(Vec::new(), 0, page.max(1), page_size.max(1)));
            }
        }

        let photos = Self::photos_relation(is_admin);
        let limit = page_size.max(1);
        let page_value = page.max(1);
        let offset = (page_value - 1) * limit;
        let sql = format!(
            r#"
            WITH selected_tag_ids AS (
                SELECT id
                FROM tags
                WHERE name_norm = ANY($1::text[])
            ),
            filtered_photos AS (
                SELECT p.id
                FROM {photos} p
                WHERE (
                    COALESCE(array_length($1::text[], 1), 0) = 0
                    OR
                    CASE
                      WHEN $2::boolean THEN (
                        SELECT COUNT(DISTINCT pt.tag_id)
                        FROM photo_tags pt
                        JOIN selected_tag_ids st ON st.id = pt.tag_id
                        WHERE pt.photo_id = p.id
                      ) = (SELECT COUNT(*) FROM selected_tag_ids)
                      ELSE EXISTS (
                        SELECT 1
                        FROM photo_tags pt
                        JOIN selected_tag_ids st ON st.id = pt.tag_id
                        WHERE pt.photo_id = p.id
                      )
                    END
                  )
            ),
            filtered_count AS (
                SELECT COUNT(*)::bigint AS total FROM filtered_photos
            )
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
                p.thumbnail_width, p.thumbnail_height,
                (SELECT total FROM filtered_count) AS _total_count
            FROM filtered_photos fp
            JOIN photos p ON p.id = fp.id
            LEFT JOIN exifs e ON p.id = e.image_id
            ORDER BY COALESCE(p.date_taken, p.created_at) DESC
            LIMIT $3 OFFSET $4
        "#
        );

        let rows = sqlx::query(&sql)
            .bind(&normalized_values)
            .bind(match_all)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        let mut items = Vec::<Photo>::new();
        let mut total = 0u64;
        for row in rows {
            use sqlx::Row;
            total = row.try_get::<i64, _>("_total_count").unwrap_or(0).max(0) as u64;
            items.push(Photo::from_row(&row).map_err(|e| DataError::Provider(e.to_string()))?);
        }

        Ok(Page::new(items, total, page_value, limit))
    }

    async fn get_photos_page(
        &self,
        page: u32,
        page_size: u32,
        is_admin: bool,
    ) -> DataResult<Page<Photo>> {
        let photos = Self::photos_relation(is_admin);
        let limit = page_size.max(1);
        let page_value = page.max(1);
        let offset = (page_value - 1) * limit;
        let count_sql = format!("SELECT COUNT(*)::bigint FROM {photos} p");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;
        let sql = format!(
            r#"
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
            FROM {photos} p
            LEFT JOIN exifs e ON p.id = e.image_id
            ORDER BY COALESCE(p.date_taken, p.created_at) DESC
            LIMIT $1 OFFSET $2
        "#
        );

        let items = sqlx::query_as::<_, Photo>(&sql)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        Ok(Page::new(items, total.max(0) as u64, page_value, limit))
    }

    async fn set_album_tags(
        &self,
        album_id: Uuid,
        tag_refs: &[TagRef],
        created_by_user_id: Option<Uuid>,
    ) -> DataResult<()> {
        let ids = self.resolve_tag_ids(tag_refs, 0).await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        sqlx::query("DELETE FROM album_tags WHERE album_id = $1")
            .bind(album_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;

        for tag_id in ids {
            sqlx::query(
                "INSERT INTO album_tags (album_id, tag_id, created_at, created_by_user_id) VALUES ($1, $2, NOW(), $3) ON CONFLICT DO NOTHING",
            )
            .bind(album_id)
            .bind(tag_id as i64)
            .bind(created_by_user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DataError::Provider(e.to_string()))?;
        Ok(())
    }

    async fn get_album_tags(&self, album_id: Uuid, is_admin: bool) -> DataResult<Vec<Tag>> {
        let vis = Self::tag_visibility_filter("t", "$2::boolean");
        let sql = format!(
            r#"
            SELECT t.id, t.name, t.visibility, t.created_at
            FROM album_tags at
            JOIN tags t ON t.id = at.tag_id
            WHERE at.album_id = $1
              AND {vis}
            ORDER BY t.name ASC
        "#
        );

        sqlx::query_as::<_, Tag>(&sql)
            .bind(album_id)
            .bind(is_admin)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataError::Provider(e.to_string()))
    }
}
