#![allow(dead_code)]

mod controllers;
mod dtos;
mod entities;

use controllers::auth_controller::AuthController;
use entities::{migrate_entities, register_entities};
use nimble_web::AppBuilder;
use nimble_web::app::application::AppError;

#[tokio::main]
async fn main() -> std::result::Result<(), AppError> {
    init_logging();
    log::info!("Start building application...");
    let mut builder = AppBuilder::new();
    builder
        .use_config("web.config.json")
        .use_postgres()
        .use_authentication()
        .use_controller::<AuthController>();
    register_entities(&mut builder);
    log::info!("Starting application...");
    let app = builder.build();
    app.log_routes();
    migrate_entities(&app);
    app.start().await?;
    Ok(())
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(log::LevelFilter::Debug);
    let _ = builder.try_init();
}
