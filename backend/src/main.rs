#![allow(dead_code)]

mod dtos;
mod entities;

use entities::register_entities;

use nimble_web::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logging();

    log::info!("Start building application...");
    let mut builder = AppBuilder::new();

    register_entities(&mut builder);

    builder.router_mut().log_routes();

    log::info!("Starting application...");
    let app = builder.build();

    app.start().await?;

    Ok(())
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(log::LevelFilter::Debug);
    let _ = builder.try_init();
}
