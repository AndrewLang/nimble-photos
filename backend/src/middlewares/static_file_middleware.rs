use async_trait::async_trait;
use nimble_web::http::context::HttpContext;
use nimble_web::pipeline::middleware::Middleware;
use nimble_web::pipeline::next::Next;
use nimble_web::pipeline::pipeline::PipelineError;
use nimble_web::result::{FileResponse, IntoResponse};
use std::path::{Component, Path, PathBuf};

pub struct StaticFileMiddleware {
    root: PathBuf,
}

impl StaticFileMiddleware {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    fn resolve_default_root() -> PathBuf {
        let mut candidates = vec![PathBuf::from("www"), PathBuf::from("backend/www")];

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("www"));
            }
        }

        for candidate in candidates {
            if candidate.join("index.html").exists() {
                log::info!("Using static root: {}", candidate.display());
                return candidate;
            }
        }

        PathBuf::from("www")
    }

    fn normalize_request_path(path: &str) -> Option<PathBuf> {
        let trimmed = path.trim_start_matches('/');
        if trimmed.is_empty() {
            return Some(PathBuf::from("index.html"));
        }

        let mut normalized = PathBuf::new();
        for component in Path::new(trimmed).components() {
            match component {
                Component::Normal(segment) => normalized.push(segment),
                Component::CurDir => {}
                _ => return None,
            }
        }

        Some(normalized)
    }

    fn should_try_spa_fallback(path: &str) -> bool {
        let leaf = path.rsplit('/').next().unwrap_or_default();
        !leaf.contains('.')
    }
}

impl Default for StaticFileMiddleware {
    fn default() -> Self {
        Self::new(Self::resolve_default_root())
    }
}

#[async_trait]
impl Middleware for StaticFileMiddleware {
    async fn handle(&self, context: &mut HttpContext, next: Next<'_>) -> Result<(), PipelineError> {
        let method = context.request().method();
        if method != "GET" && method != "HEAD" {
            return next.run(context).await;
        }

        let request_path = context.request().path();
        if request_path.starts_with("/api") {
            return next.run(context).await;
        }

        let Some(normalized_path) = Self::normalize_request_path(request_path) else {
            context.response_mut().set_status(400);
            return Ok(());
        };

        let mut file_path = self.root.join(normalized_path);
        if file_path.is_dir() {
            file_path = file_path.join("index.html");
        }

        if !file_path.exists() {
            if Self::should_try_spa_fallback(request_path) {
                file_path = self.root.join("index.html");
            } else {
                return next.run(context).await;
            }
        }

        if !file_path.exists() {
            return next.run(context).await;
        }

        FileResponse::from_path(file_path).into_response(context);
        Ok(())
    }
}
