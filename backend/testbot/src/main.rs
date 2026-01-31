use anyhow::Result;
use env_logger;
use std::env;
use std::process::Command;
use std::time::{Duration, Instant};

use nimble_web::testbot::TestBot;
use tokio::time::sleep;
mod auth;
use auth::AuthScenario;

const DEFAULT_PORT: u16 = 7878;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    env::set_var("RUST_LOG", "debug");
    env::set_var("Nimble_Photo_Url", format!("0.0.0.0:{}", DEFAULT_PORT));
    let start = Instant::now();

    let host_url =
        env::var("Nimble_Photo_Url").unwrap_or_else(|_| format!("0.0.0.0:{}", DEFAULT_PORT));
    log::info!("Starting hosting application at {}", host_url);

    start_hosting_application().await?;

    let bound_address = wait_for_bot_address()
        .await
        .unwrap_or_else(|_| format!("localhost:{}", DEFAULT_PORT));
    let base_url = format!("http://{bound_address}");
    log::info!("Start testing endpoints at URL: {}", base_url);

    let mut bot = TestBot::connect(base_url).await?;

    let scenario_result = bot.run_scenario(AuthScenario::new()).await;

    scenario_result?;

    env::remove_var("Nimble_Photo_Url");
    log::info!("Testbot finished in {:?}", start.elapsed());

    Ok(())
}

async fn wait_for_bot_address() -> Result<String> {
    sleep(Duration::from_millis(1000 * 5)).await;
    Ok(format!("localhost:{}", DEFAULT_PORT))
}

async fn start_hosting_application() -> Result<()> {
    Command::new("cargo")
        .args(&["run", "--bin", "nimble-photos"])
        .current_dir("..")
        .spawn()?;
    Ok(())
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder
        .filter_level(log::LevelFilter::Off)
        .filter_module("nimble_web::testbot", log::LevelFilter::Debug);

    let _ = builder.try_init();
}
