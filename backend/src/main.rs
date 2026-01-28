#![allow(dead_code)]

mod controllers;
mod dtos;
mod entities;

use entities::register_entities;
use nimble_web::AppBuilder;
use nimble_web::app::application::AppError;

#[tokio::main]
async fn main() -> std::result::Result<(), AppError> {
    init_logging();

    log::info!("Start building application...");
    let mut builder = AppBuilder::new();

    register_entities(&mut builder);
    // enable authentication and register the AuthController
    builder.use_authentication();
    builder.use_controller::<controllers::auth_controller::AuthController>();

    log::info!("Starting application...");
    let app = builder.build();
    app.log_routes();

    app.start().await?;

    Ok(())
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(log::LevelFilter::Debug);
    let _ = builder.try_init();
}
