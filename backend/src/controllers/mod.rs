pub mod admin_user_controller;
pub mod album_controller;
pub mod assets_controller;
pub mod auth_controller;
pub mod client_controller;
pub mod dashboard_controller;
pub mod httpcontext_extensions;
pub mod photo_controller;
pub mod storage_controller;
pub mod tag_controller;

use admin_user_controller::AdminUserController;
use album_controller::AlbumController;
use assets_controller::AssetsController;
use auth_controller::AuthController;
use dashboard_controller::DashboardController;
use nimble_web::*;
use photo_controller::PhotoController;
use storage_controller::StorageController;
use tag_controller::TagController;

pub fn register_controllers(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder
        .use_controller::<AdminUserController>()
        .use_controller::<AuthController>()
        .use_controller::<PhotoController>()
        .use_controller::<TagController>()
        .use_controller::<DashboardController>()
        .use_controller::<AlbumController>()
        .use_controller::<AssetsController>()
        .use_controller::<StorageController>();

    builder
}
