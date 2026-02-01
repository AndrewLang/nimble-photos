use anyhow::{Result, anyhow};
use env_logger;
use std::env;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use nimble_web::testbot::TestBot;
use tokio::time::sleep;
mod album;
mod auth;
mod photo;
use album::AlbumScenario;
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
    let bound_address = wait_for_bot_address().await?;
    let base_url = format!("http://{bound_address}");

    log::info!("Start testing endpoints at URL: {}", base_url);

    let mut bot = TestBot::connect(base_url).await?;
    bot.add_scenario(AuthScenario::new());
    bot.add_scenario(PhotoScenario::new());
    bot.add_scenario(AlbumScenario::new());

    bot.run().await?;
    Ok(())
}
async fn wait_for_bot_address() -> Result<String> {
    let addr = env::var("Nimble_Photo_Url").unwrap_or_else(|_| format!("0.0.0.0:{DEFAULT_PORT}"));

    let mut socket: std::net::SocketAddr = addr
        .parse()
        .map_err(|e| anyhow!("invalid Nimble_Photo_Url '{}': {}", addr, e))?;

    if socket.ip().is_unspecified() {
        socket.set_ip(std::net::IpAddr::from([127, 0, 0, 1]));
    }

    let display = socket.to_string();
    log::info!("🤖 ⏳ waiting for host at {}", display);

    let deadline = Instant::now() + Duration::from_secs(30);

    loop {
        match tokio::net::TcpStream::connect(socket).await {
            Ok(stream) => {
                drop(stream);
                log::info!("🤖 ✅ host ready at {}", display);
                return Ok(display);
            }
            Err(err) if Instant::now() < deadline => {
                log::debug!("🤖 … waiting: {}", err);
                sleep(Duration::from_millis(500)).await;
            }
            Err(err) => {
                return Err(anyhow!(
                    "timed out waiting for host at {}: {}",
                    display,
                    err
                ));
            }
        }
    }
}

async fn start_hosting_application() -> Result<Child> {
    log::info!("Starting hosting application...");
    let child = Command::new("cargo")
        .args(&["run", "--bin", "nimble-photos", "--features", "testbot"])
        .current_dir("..")
        .env("RUST_LOG", "off")
        .spawn()?;

    log::info!("Hosting application started with PID {}", child.id());

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
        .filter_module("nimble_web::testbot", log::LevelFilter::Debug)
        .filter_module("nimble_photos", log::LevelFilter::Error);

    let _ = builder.try_init();
}
