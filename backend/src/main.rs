#![allow(dead_code)]

use nimble_photos::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), AppError> {
    init_logging();

    log::info!("Start building application...");
    let bind_address = resolve_bind_address();
    let mut builder = AppBuilder::new();
    builder
        .use_config("web.config.json")
        .use_env()
        .use_address(&bind_address)
        .use_postgres()
        .use_middleware(CorsMiddleware::default())
        .use_authentication()
        .use_middleware(PublicAccessMiddleware::new())
        .use_middleware(StaticFileMiddleware::default());

    register_services(&mut builder);
    register_controllers(&mut builder);
    register_entities(&mut builder);

    log::info!("Starting application...");
    let app = builder.build();
    let _ = app.services().get::<PhotoService>();

    app.log_routes();

    log::info!("Migrating database...");
    migrate_entities(&app).await.map_err(|err| AppError::Runtime(format!("migrate entities: {err}")))?;

    app.start().await?;

    Ok(())
}

fn resolve_bind_address() -> String {
    if let Ok(address) = std::env::var("Nimble_Photo_Url") {
        return address;
    }

    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "5151".to_string());
    format!("{host}:{port}")
}

fn init_logging() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");

    let mut builder = env_logger::Builder::from_env(env);

    if std::env::var("RUST_LOG").is_err() {
        builder.filter_level(log::LevelFilter::Debug).filter_module("sqlx", log::LevelFilter::Info);
    }

    let _ = builder.try_init();
}
