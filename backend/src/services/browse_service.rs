use crate::prelude::*;
use anyhow::{Result, anyhow};
use sqlx::{PgPool, Row};

use crate::entities::photo_browse::{BrowseNodeType, BrowseOptions, BrowsePhoto, BrowseResponse, StorageFolder};
use crate::entities::photo_cursor::PhotoCursor;
use crate::models::browse_dimension_sql_adapter::{BrowseDimensionSqlAdapter, SqlParam};
#[cfg(feature = "postgres")]
use crate::repositories::postgres_extensions::PostgresExtensions;

pub struct BrowseService {
    pool: Arc<PgPool>,
}

impl BrowseService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn browse(
        &self,
        storage_id: &Uuid,
        path_segments: &[String],
        options: &BrowseOptions,
        page_size: i64,
        cursor: Option<PhotoCursor>,
    ) -> Result<BrowseResponse> {
        log::info!(
            "Browsing storage_id={}, path_segments={:?}, options={:?}, page_size={}, cursor={:?}",
            storage_id,
            path_segments,
            options,
            page_size,
            cursor
        );
        let depth = path_segments.len();
        if depth > options.dimensions.len() {
            return Err(anyhow!("invalid browse path depth"));
        }

        if depth < options.dimensions.len() {
            return self.browse_folders(storage_id, path_segments, options, depth).await;
        }

        self.browse_photos(storage_id, path_segments, options, page_size, cursor).await
    }

    async fn browse_folders(
        &self,
        storage_id: &Uuid,
        path_segments: &[String],
        options: &BrowseOptions,
        depth: usize,
    ) -> Result<BrowseResponse> {
        let mut where_clauses = vec!["p.storage_id = $1".to_string()];
        let mut params: Vec<SqlParam> = Vec::new();
        let mut param_index = 2usize;

        for (index, segment) in path_segments.iter().enumerate() {
            let adapter = BrowseDimensionSqlAdapter::new(options.dimensions[index].clone());
            let param = adapter.parse_segment_value(segment)?;
            where_clauses.push(adapter.filter_clause(param_index));
            params.push(param);
            param_index += 1;
        }

        let target_adapter = BrowseDimensionSqlAdapter::new(options.dimensions[depth].clone());
        let (folder_select, group_expr) = target_adapter.group_select();
        let order_dir = BrowseDimensionSqlAdapter::order_direction(&options.sort_direction);

        let sql = format!(
            "SELECT {folder_select}, COUNT(*)::bigint AS file_count
             FROM photos p
             WHERE {}
             GROUP BY {group_expr}
             ORDER BY {group_expr} {order_dir}",
            where_clauses.join(" AND ")
        );

        log::info!("Executing browse folders SQL: {}", sql);

        let mut query = sqlx::query(&sql).bind(*storage_id);
        for param in params {
            query = match param {
                SqlParam::Int(value) => query.bind(value),
                SqlParam::String(value) => query.bind(value),
            };
        }

        let rows = query.fetch_all(&*self.pool).await?;
        let has_children = depth + 1 < options.dimensions.len();
        let mut folders = Vec::<StorageFolder>::new();

        for row in rows {
            let folder_name = row
                .try_get::<String, _>("folder")
                .or_else(|_| row.try_get::<i32, _>("folder").map(|value| value.to_string()))
                .or_else(|_| row.try_get::<i64, _>("folder").map(|value| value.to_string()))
                .map_err(|_| anyhow!("invalid folder value"))?;

            let full_path = if path_segments.is_empty() {
                folder_name.clone()
            } else {
                format!("{}/{}", path_segments.join("/"), folder_name)
            };

            folders.push(StorageFolder {
                name: folder_name,
                full_path,
                depth: depth + 1,
                file_count: row.try_get::<i64, _>("file_count").unwrap_or(0),
                has_children,
            });
        }

        Ok(BrowseResponse {
            node_type: BrowseNodeType::Folders,
            folders: Some(folders),
            photos: None,
            next_cursor: None,
        })
    }

    async fn browse_photos(
        &self,
        storage_id: &Uuid,
        path_segments: &[String],
        options: &BrowseOptions,
        page_size: i64,
        cursor: Option<PhotoCursor>,
    ) -> Result<BrowseResponse> {
        let mut where_clauses = vec!["p.storage_id = $1".to_string()];
        let mut params: Vec<SqlParam> = Vec::new();
        let mut param_index = 2usize;

        for (index, segment) in path_segments.iter().enumerate() {
            let adapter = BrowseDimensionSqlAdapter::new(options.dimensions[index].clone());
            let param = adapter.parse_segment_value(segment)?;
            where_clauses.push(adapter.filter_clause(param_index));
            params.push(param);
            param_index += 1;
        }

        let order_dir = BrowseDimensionSqlAdapter::order_direction(&options.sort_direction);

        let start = std::time::Instant::now();
        let mut cursor_values: Option<(DateTime<Utc>, Uuid)> = None;
        if let Some(cursor_value) = cursor {
            let condition = if order_dir == "DESC" {
                format!(
                    "(p.sort_date < ${} OR (p.sort_date = ${} AND p.id < ${}))",
                    param_index,
                    param_index,
                    param_index + 1
                )
            } else {
                format!(
                    "(p.sort_date > ${} OR (p.sort_date = ${} AND p.id > ${}))",
                    param_index,
                    param_index,
                    param_index + 1
                )
            };
            where_clauses.push(condition);
            cursor_values = Some((cursor_value.sort_date, cursor_value.id));
            param_index += 2;
        }

        let normalized_size = page_size.clamp(1, 200);
        let sql = format!(
            "SELECT p.id, p.storage_id, p.name AS file_name, p.name, p.format, p.hash, p.size, p.created_at,
                    p.updated_at, p.date_imported, p.date_taken, p.year, p.month_day, p.metadata_extracted,
                    p.artist, p.make, p.model, p.lens_make, p.lens_model, p.exposure_time, p.iso, p.focal_length,
                    p.label, p.rating, p.flagged, p.is_raw, p.width, p.height, p.orientation, p.day_date, p.sort_date
             FROM photos p
             WHERE {}
             ORDER BY p.sort_date {order_dir}, p.id {order_dir}
             LIMIT ${}",
            where_clauses.join(" AND "),
            param_index
        );
        log::info!(
            "Browse photos SQL: {}, storage_id={}, params={:?}, cursor={:?}, limit={}",
            sql,
            storage_id,
            params,
            cursor_values,
            normalized_size + 1
        );

        let mut query = sqlx::query(&sql).bind(*storage_id);
        for param in params {
            query = match param {
                SqlParam::Int(value) => query.bind(value),
                SqlParam::String(value) => query.bind(value),
            };
        }
        if let Some((cursor_date, cursor_id)) = cursor_values {
            query = query.bind(cursor_date).bind(cursor_id);
        }
        query = query.bind(normalized_size + 1);

        let rows = query.fetch_all(&*self.pool).await?;
        log::info!("Browse photos returned {} rows", rows.len());

        let has_next = rows.len() as i64 > normalized_size;
        let rows = if has_next { rows.into_iter().take(normalized_size as usize).collect::<Vec<_>>() } else { rows };

        let mut entries = Vec::<(BrowsePhoto, DateTime<Utc>)>::new();
        for row in rows {
            let sort_date: DateTime<Utc> = row.try_get("sort_date")?;
            let photo = BrowsePhoto {
                id: row.try_get("id")?,
                storage_id: row.try_get("storage_id")?,
                file_name: row.try_get("file_name")?,
                name: row.try_get("name")?,
                format: row.try_get("format")?,
                hash: row.try_get("hash")?,
                size: row.try_get("size")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                date_imported: row.try_get("date_imported")?,
                date_taken: row.try_get("date_taken")?,
                year: row.try_get("year")?,
                month_day: row.try_get("month_day")?,
                metadata_extracted: row.try_get("metadata_extracted")?,
                artist: row.try_get("artist")?,
                make: row.try_get("make")?,
                model: row.try_get("model")?,
                lens_make: row.try_get("lens_make")?,
                lens_model: row.try_get("lens_model")?,
                exposure_time: row.try_get("exposure_time")?,
                iso: PostgresExtensions::optional_i32_as_u32(&row, "iso")?,
                focal_length: row.try_get("focal_length")?,
                label: row.try_get("label")?,
                rating: PostgresExtensions::optional_i32_as_u8(&row, "rating")?,
                flagged: PostgresExtensions::optional_i32_as_i8(&row, "flagged")?,
                is_raw: row.try_get("is_raw")?,
                width: PostgresExtensions::optional_i32_as_u32(&row, "width")?,
                height: PostgresExtensions::optional_i32_as_u32(&row, "height")?,
                orientation: PostgresExtensions::optional_i32_as_u16(&row, "orientation")?,
                day_date: row.try_get("day_date")?,
                sort_date: sort_date.clone(),
            };
            entries.push((photo, sort_date));
        }

        let next_cursor = if has_next {
            entries.last().map(|(photo, sort_date)| PhotoCursor { sort_date: sort_date.clone(), id: photo.id }.encode())
        } else {
            None
        };

        let photos: Vec<BrowsePhoto> = entries.into_iter().map(|(photo, _)| photo).collect();
        log::info!("Photos {} - elapsed: {:?}", photos.len(), start.elapsed());

        Ok(BrowseResponse { node_type: BrowseNodeType::Photos, folders: None, photos: Some(photos), next_cursor })
    }
}
