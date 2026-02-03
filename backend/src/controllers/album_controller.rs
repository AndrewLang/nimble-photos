use crate::entities::album::Album;
use crate::repositories::photo::PhotoRepository;
use async_trait::async_trait;
use nimble_web::controller::controller::Controller;
use nimble_web::data::provider::DataProvider;
use nimble_web::data::repository::Repository;
use nimble_web::endpoint::http_handler::HttpHandler;
use nimble_web::endpoint::route::EndpointRoute;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::Json;
use nimble_web::result::into_response::ResponseValue;
use serde::Deserialize;
use uuid::Uuid;

use sqlx::PgPool;

pub struct AlbumController;

impl Controller for AlbumController {
    fn routes() -> Vec<EndpointRoute> {
        vec![
            EndpointRoute::get(
                "/api/albums/{id}/photos/{page}/{pageSize}",
                AlbumPhotosHandler,
            )
            .build(),
            EndpointRoute::get("/api/albums/{page}/{pageSize}", ListAlbumsHandler).build(),
        ]
    }
}

struct AlbumPhotosHandler;

#[derive(Deserialize)]
struct AlbumRules {
    #[serde(rename = "photoIds")]
    photo_ids: Vec<Uuid>,
}

#[async_trait]
impl HttpHandler for AlbumPhotosHandler {
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
        let route_params = context.route().map(|r| r.params());

        let id_str = route_params
            .and_then(|p| p.get("id"))
            .ok_or_else(|| PipelineError::message("id parameter missing"))?;

        let page: u32 = route_params
            .and_then(|p| p.get("page"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let page_size: u32 = route_params
            .and_then(|p| p.get("pageSize"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        let id = Uuid::parse_str(id_str).map_err(|_| PipelineError::message("invalid album id"))?;

        let album_repo = context.service::<Repository<Album>>()?;

        let album = album_repo
            .get(&id)
            .await
            .map_err(|e| PipelineError::message(&format!("Failed to fetch album: {:?}", e)))?
            .ok_or_else(|| PipelineError::message("Album not found"))?;

        let mut photos = Vec::new();
        let mut total = 0u64;

        if let Some(rules_json) = album.rules_json {
            if let Ok(rules) = serde_json::from_str::<AlbumRules>(&rules_json) {
                total = rules.photo_ids.len() as u64;

                let start = ((page - 1) * page_size) as usize;
                let mut end = start + page_size as usize;
                if end > rules.photo_ids.len() {
                    end = rules.photo_ids.len();
                }

                if start < rules.photo_ids.len() {
                    let slice = &rules.photo_ids[start..end];
                    let photo_repo = context.service::<Box<dyn PhotoRepository>>()?;

                    photos = photo_repo
                        .get_by_ids(slice)
                        .await
                        .map_err(|e| PipelineError::message(&format!("{:?}", e)))?;
                }
            }
        }

        let response = serde_json::json!({
            "page": page,
            "page_size": page_size,
            "total": total,
            "items": photos
        });

        Ok(ResponseValue::new(Json(response)))
    }
}

struct ListAlbumsHandler;

#[async_trait]
impl HttpHandler for ListAlbumsHandler {
    async fn invoke(
        &self,
        context: &mut HttpContext,
    ) -> std::result::Result<ResponseValue, PipelineError> {
        let pool = context.service::<PgPool>()?;
        let route_params = context.route().map(|r| r.params());

        let page: u32 = route_params
            .and_then(|p| p.get("page"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let page_size: u32 = route_params
            .and_then(|p| p.get("pageSize"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        let offset = if page > 0 { (page - 1) * page_size } else { 0 };

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM albums")
            .fetch_one(&*pool)
            .await
            .map_err(|e| PipelineError::message(&format!("Failed to count albums: {:?}", e)))?;

        let albums = sqlx::query_as::<_, Album>(
            "SELECT * FROM albums ORDER BY create_date DESC LIMIT $1 OFFSET $2",
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&*pool)
        .await
        .map_err(|e| PipelineError::message(&format!("Failed to fetch albums: {:?}", e)))?;

        let response = serde_json::json!({
            "page": page,
            "pageSize": page_size,
            "total": total,
            "items": albums
        });

        Ok(ResponseValue::new(Json(response)))
    }
}
