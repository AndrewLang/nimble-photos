use album::Album;
use exif::Exif;
use nimble_web::app::application::Application;
#[cfg(not(feature = "postgres"))]
use nimble_web::data::memory_repository::MemoryRepository;
use nimble_web::data::repository::Repository;
use nimble_web::entity::operation::EntityOperation;
// use nimble_web::security::policy::Policy;
use nimble_web::*;
use photo::Photo;
use user::User;
use user_settings::UserSettings;

pub mod album;
pub mod exif;
pub mod photo;
pub mod user;
pub mod user_settings;

pub fn register_entities(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.use_entity_with_operations_and_policy::<User>(
        &[EntityOperation::Get, EntityOperation::List],
        Policy::Authenticated,
    );
    builder.use_entity_with_operations::<UserSettings>(&[
        EntityOperation::Get,
        EntityOperation::Update,
    ]);
    builder.use_entity::<Photo>();
    builder.use_entity::<Album>();
    builder.use_entity_with_operations::<Exif>(&[EntityOperation::Get]);

    #[cfg(not(feature = "postgres"))]
    {
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<Photo>::new();
            Repository::<Photo>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<Exif>::new();
            Repository::<Exif>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<User>::new();
            Repository::<User>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<UserSettings>::new();
            Repository::<UserSettings>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<Album>::new();
            Repository::<Album>::new(Box::new(provider))
        });
    }

    #[cfg(feature = "postgres")]
    {
        use sqlx::PgPool;

        log::debug!("Registering Postgres repositories for entities...");
        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<Photo>::new((*pool).clone());
            Repository::<Photo>::new(Box::new(provider))
        });

        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<Exif>::new((*pool).clone());
            Repository::<Exif>::new(Box::new(provider))
        });

        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<User>::new((*pool).clone());
            Repository::<User>::new(Box::new(provider))
        });

        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<UserSettings>::new((*pool).clone());
            Repository::<UserSettings>::new(Box::new(provider))
        });

        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<Album>::new((*pool).clone());
            Repository::<Album>::new(Box::new(provider))
        });
    }

    builder
}

pub async fn migrate_entities(app: &Application) {
    #[cfg(not(feature = "postgres"))]
    {
        let _ = app;
    }
    #[cfg(feature = "postgres")]
    {
        let _ = app.migrate_entity::<User>().await;
        let _ = app.migrate_entity::<UserSettings>().await;
        let _ = app.migrate_entity::<Photo>().await;
        let _ = app.migrate_entity::<Album>().await;
        let _ = app.migrate_entity::<Exif>().await;
    }
}
