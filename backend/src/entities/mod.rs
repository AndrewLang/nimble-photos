use nimble_web::*;

use user::User;
use user_settings::UserSettings;

pub mod user;
pub mod user_settings;

pub fn register_entities(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.use_entity::<User>();
    builder.use_entity::<UserSettings>();

    builder
}
