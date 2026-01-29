pub mod auth_controller;

use auth_controller::AuthController;
use nimble_web::*;

pub fn register_controllers(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.use_controller::<AuthController>();
    builder
}
