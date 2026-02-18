use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use crate::entities::photo_browse::{
    BrowseNodeType, BrowseOptions, BrowsePhoto, BrowseResponse, StorageFolder,
};
use crate::entities::photo_cursor::PhotoCursor;
use crate::models::browse_dimension_sql_adapter::{BrowseDimensionSqlAdapter, SqlParam};

pub struct BrowseService {
    pool: Arc<PgPool>,
}

impl BrowseService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn browse(
        &self,
        storage_id: &str,
        path_segments: &[String],
        options: &BrowseOptions,
        page_size: i64,
        cursor: Option<PhotoCursor>,
    ) -> Result<BrowseResponse> {
        let depth = path_segments.len();
        if depth > options.dimensions.len() {
            return Err(anyhow!("invalid browse path depth"));
        }

        if depth < options.dimensions.len() {
            return self
                .browse_folders(storage_id, path_segments, options, depth)
                .await;
        }

        self.browse_photos(storage_id, path_segments, options, page_size, cursor)
            .await
    }

    async fn browse_folders(
        &self,
        storage_id: &str,
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

        let mut query = sqlx::query(&sql).bind(storage_id.to_string());
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
        storage_id: &str,
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

        let mut cursor_values: Option<(DateTime<Utc>, Uuid)> = None;
        if let Some(cursor_value) = cursor {
            let condition = if order_dir == "DESC" {
                format!(
                    "(p.date_taken < ${} OR (p.date_taken = ${} AND p.id < ${}))",
                    param_index,
                    param_index,
                    param_index + 1
                )
            } else {
                format!(
                    "(p.date_taken > ${} OR (p.date_taken = ${} AND p.id > ${}))",
                    param_index,
                    param_index,
                    param_index + 1
                )
            };
            where_clauses.push(condition);
            cursor_values = Some((cursor_value.date_taken, cursor_value.id));
            param_index += 2;
        }

        let normalized_size = page_size.clamp(1, 200);
        let sql = format!(
            "SELECT p.id, p.name AS file_name, COALESCE(p.hash, '') AS hash, p.date_taken, p.width, p.height
             FROM photos p
             WHERE {}
             ORDER BY p.date_taken {order_dir}, p.id {order_dir}
             LIMIT ${}",
            where_clauses.join(" AND "),
            param_index
        );

        let mut query = sqlx::query(&sql).bind(storage_id.to_string());
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
        let has_next = rows.len() as i64 > normalized_size;
        let rows = if has_next {
            rows.into_iter().take(normalized_size as usize).collect::<Vec<_>>()
        } else {
            rows
        };

        let mut photos = Vec::<BrowsePhoto>::new();
        for row in rows {
            photos.push(BrowsePhoto {
                id: row.try_get("id")?,
                file_name: row.try_get("file_name")?,
                hash: row.try_get("hash")?,
                date_taken: row.try_get("date_taken")?,
                width: row.try_get("width")?,
                height: row.try_get("height")?,
            });
        }

        let next_cursor = if has_next {
            photos
                .last()
                .and_then(|photo| photo.date_taken.map(|date| (date, photo.id)))
                .map(|(date_taken, id)| PhotoCursor { date_taken, id }.encode())
        } else {
            None
        };

        Ok(BrowseResponse {
            node_type: BrowseNodeType::Photos,
            folders: None,
            photos: Some(photos),
            next_cursor,
        })
    }
}
