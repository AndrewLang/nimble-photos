pub mod album_controller;
pub mod auth_controller;
pub mod dashboard_controller;
pub mod photo_controller;

use album_controller::AlbumController;
use auth_controller::AuthController;
use dashboard_controller::DashboardController;
use nimble_web::*;
use photo_controller::PhotoController;

pub fn register_controllers(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder
        .use_controller::<AuthController>()
        .use_controller::<PhotoController>()
        .use_controller::<DashboardController>()
        .use_controller::<AlbumController>();

    builder
}
