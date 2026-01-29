use nimble_web::app::application::Application;
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

pub async fn migrate_entities(app: &Application) {
    #[cfg(feature = "postgres")]
    {
        let _ = app.migrate_entity::<User>().await;
        let _ = app.migrate_entity::<UserSettings>().await;
    }
}
