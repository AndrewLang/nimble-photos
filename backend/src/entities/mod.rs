use album::Album;
use exif::ExifModel;
#[cfg(not(feature = "postgres"))]
use nimble_web::data::memory_repository::MemoryRepository;
use nimble_web::*;
use photo::Photo;
use user::User;
use user_settings::UserSettings;
use uuid_id::EnsureUuidIdHooks;

use crate::repositories::photo::PhotoRepository;

pub mod album;
pub mod album_hooks;
pub mod exif;
pub mod photo;
pub mod user;
pub mod user_settings;
pub mod uuid_id;

pub fn register_entities(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.use_entity_with_operations_and_policy::<User>(
        &[EntityOperation::Get, EntityOperation::List],
        Policy::Authenticated,
    );
    builder.use_entity_with_operations::<UserSettings>(&[
        EntityOperation::Get,
        EntityOperation::Update,
    ]);
    builder.use_entity_with_hooks(EnsureUuidIdHooks::<Photo>::new(), EntityOperation::all());
    builder.use_entity_with_hooks_and_policy(
        album_hooks::AlbumHooks::new(),
        &[
            EntityOperation::List,
            EntityOperation::Get,
            EntityOperation::Create,
            EntityOperation::Update,
        ],
        Policy::Authenticated,
    );
    builder.use_entity_with_hooks_and_policy(
        album_hooks::AlbumHooks::new(),
        &[EntityOperation::Delete],
        Policy::InRole("admin".to_string()),
    );
    builder.use_entity_with_hooks(
        EnsureUuidIdHooks::<ExifModel>::new(),
        &[EntityOperation::Get],
    );

    #[cfg(not(feature = "postgres"))]
    {
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<Photo>::new();
            Repository::<Photo>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<ExifModel>::new();
            Repository::<ExifModel>::new(Box::new(provider))
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
            let repo: Box<dyn PhotoRepository> = Box::new(
                crate::repositories::photo::PostgresPhotoRepository::new((*pool).clone()),
            );
            repo
        });

        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<ExifModel>::new((*pool).clone());
            Repository::<ExifModel>::new(Box::new(provider))
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
        let pool = app
            .services()
            .resolve::<sqlx::PgPool>()
            .expect("PgPool not found");

        let _ = app.migrate_entity::<User>().await;
        let _ = app.migrate_entity::<UserSettings>().await;
        let _ = app.migrate_entity::<Photo>().await;
        let _ = app.migrate_entity::<Album>().await;
        let _ = app.migrate_entity::<ExifModel>().await;

        log::info!("Creating additional indices for performance...");
        let sqls = [
            "CREATE INDEX IF NOT EXISTS idx_photos_date_taken ON photos (date_taken)",
            "CREATE INDEX IF NOT EXISTS idx_photos_created_at ON photos (created_at)",
            "CREATE INDEX IF NOT EXISTS idx_photos_sort_date_v2 ON photos ((DATE(COALESCE(date_taken, created_at) AT TIME ZONE 'UTC')))",
            "CREATE INDEX IF NOT EXISTS idx_exifs_image_id ON exifs (image_id)",
        ];

        for sql in sqls {
            if let Err(e) = sqlx::query(sql).execute(&*pool).await {
                log::error!("Failed to execute SQL {}: {}", sql, e);
            }
        }
    }
}
