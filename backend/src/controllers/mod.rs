pub mod auth_controller;

use nimble_web::*;

use auth_controller::AuthController;

pub fn register_controllers(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.use_controller::<AuthController>();

    builder
}
