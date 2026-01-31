use anyhow::Result;
use env_logger;
use std::env;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use nimble_web::testbot::TestBot;
use tokio::time::sleep;
mod auth;
mod photo;
use auth::AuthScenario;
use photo::PhotoScenario;

const DEFAULT_PORT: u16 = 7878;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    configure_env();

    let start = Instant::now();
    log::info!(
        "Starting hosting application at {}",
        env::var("Nimble_Photo_Url").unwrap()
    );

    let mut host_process = start_hosting_application().await?;
    let scenario_result = execute_testbot().await;
    shutdown_host(&mut host_process);

    cleanup_env();
    log::info!("Testbot finished in {:?}", start.elapsed());

    scenario_result?;
    Ok(())
}

fn configure_env() {
    env::set_var("Nimble_Photo_Url", format!("0.0.0.0:{}", DEFAULT_PORT));
}

fn cleanup_env() {
    env::remove_var("Nimble_Photo_Url");
}

async fn execute_testbot() -> Result<()> {
    let bound_address = wait_for_bot_address()
        .await
        .unwrap_or_else(|_| format!("localhost:{}", DEFAULT_PORT));
    let base_url = format!("http://{bound_address}");

    log::info!("Start testing endpoints at URL: {}", base_url);

    let mut bot = TestBot::connect(base_url).await?;
    bot.add_scenario(AuthScenario::new());
    bot.add_scenario(PhotoScenario::new());

    bot.run().await?;
    Ok(())
}

async fn wait_for_bot_address() -> Result<String> {
    sleep(Duration::from_millis(1000 * 5)).await;
    Ok(format!("localhost:{}", DEFAULT_PORT))
}

async fn start_hosting_application() -> Result<Child> {
    let child = Command::new("cargo")
        .args(&["run", "--bin", "nimble-photos"])
        .current_dir("..")
        .env("RUST_LOG", "off")
        .spawn()?;

    Ok(child)
}

fn shutdown_host(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_default_env();
    builder
        .filter(None, log::LevelFilter::Off)
        .filter_module("testbot", log::LevelFilter::Debug)
        .filter_module("nimble_web::testbot", log::LevelFilter::Debug);

    let _ = builder.try_init();
}
