use nimble_web::app::application::Application;
use nimble_web::*;
use user::User;
use user_settings::UserSettings;

pub mod user;
pub mod user_settings;

pub fn register_entities(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.use_entity::<User>();
    builder.use_entity::<UserSettings>();

    #[cfg(feature = "postgres")]
    {
        use nimble_web::data::postgres::PostgresProvider;
        use nimble_web::data::repository::Repository;
        use sqlx::PgPool;

        builder.register_singleton(|p| {
            let pool = p.resolve::<PgPool>().expect("pool missing");
            let provider = PostgresProvider::<User>::new((*pool).clone());
            Repository::<User>::new(Box::new(provider))
        });

        builder.register_singleton(|p| {
            let pool = p.resolve::<PgPool>().expect("pool missing");
            let provider = PostgresProvider::<UserSettings>::new((*pool).clone());
            Repository::<UserSettings>::new(Box::new(provider))
        });
    }

    builder
}

pub async fn migrate_entities(app: &Application) {
    #[cfg(feature = "postgres")]
    {
        let _ = app.migrate_entity::<User>().await;
        let _ = app.migrate_entity::<UserSettings>().await;
    }
}
