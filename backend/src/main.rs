#![allow(dead_code)]

mod controllers;
mod dtos;
mod entities;
mod services;

use controllers::register_controllers;
use entities::{migrate_entities, register_entities};
use nimble_web::AppBuilder;
use nimble_web::app::application::AppError;
use services::register_services;

#[tokio::main]
async fn main() -> std::result::Result<(), AppError> {
    init_logging();

    log::info!("Start building application...");
    let mut builder = AppBuilder::new();
    builder
        .use_config("web.config.json")
        .use_postgres()
        .use_authentication();

    register_services(&mut builder);
    register_controllers(&mut builder);
    register_entities(&mut builder);

    log::info!("Starting application...");
    let app = builder.build();

    app.log_routes();

    log::info!("Migrating database...");
    migrate_entities(&app).await;

    app.start().await?;

    Ok(())
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder
        .filter_level(log::LevelFilter::Debug)
        .filter_module("sqlx", log::LevelFilter::Debug);
    let _ = builder.try_init();
}
