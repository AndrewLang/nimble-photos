use album::Album;
use album_comment::AlbumComment;
use exif::ExifModel;
#[cfg(not(feature = "postgres"))]
use nimble_web::data::memory_repository::MemoryRepository;
use nimble_web::*;
use photo::Photo;
use photo_comment::PhotoComment;
use setting::Setting;
use user::User;
use user_settings::UserSettings;
use uuid_id::EnsureUuidIdHooks;

use crate::entities::album_hooks::AlbumHooks;
use crate::repositories::photo::PhotoRepository;

pub use storage_location::ImageStorageLocation;

pub mod album;
pub mod album_comment;
pub mod album_hooks;
pub mod exif;
pub mod photo;
pub mod photo_comment;
pub mod setting;
pub mod storage_location;
pub mod tag;
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
        AlbumHooks::new(),
        &[
            EntityOperation::List,
            EntityOperation::Get,
            EntityOperation::Create,
            EntityOperation::Update,
        ],
        Policy::Authenticated,
    );
    builder.use_entity_with_hooks_and_policy(
        AlbumHooks::new(),
        &[EntityOperation::Delete],
        Policy::InRole("admin".to_string()),
    );
    builder.use_entity_with_hooks(
        EnsureUuidIdHooks::<ExifModel>::new(),
        &[EntityOperation::Get],
    );
    builder.use_entity_with_hooks(
        EnsureUuidIdHooks::<PhotoComment>::new(),
        &[
            EntityOperation::List,
            EntityOperation::Get,
            EntityOperation::Create,
        ],
    );
    builder.use_entity_with_hooks(
        EnsureUuidIdHooks::<AlbumComment>::new(),
        &[
            EntityOperation::List,
            EntityOperation::Get,
            EntityOperation::Create,
            EntityOperation::Update,
        ],
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
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<PhotoComment>::new();
            Repository::<PhotoComment>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<Setting>::new();
            Repository::<Setting>::new(Box::new(provider))
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
        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<PhotoComment>::new((*pool).clone());
            Repository::<PhotoComment>::new(Box::new(provider))
        });
        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<AlbumComment>::new((*pool).clone());
            Repository::<AlbumComment>::new(Box::new(provider))
        });
        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<Setting>::new((*pool).clone());
            Repository::<Setting>::new(Box::new(provider))
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
        let _ = app.migrate_entity::<PhotoComment>().await;
        let _ = app.migrate_entity::<AlbumComment>().await;
        let _ = app.migrate_entity::<Setting>().await;

        log::info!("Creating additional indices for performance...");
        let sqls = [
            "CREATE INDEX IF NOT EXISTS idx_photos_date_taken ON photos (date_taken)",
            "CREATE INDEX IF NOT EXISTS idx_photos_created_at ON photos (created_at)",
            "CREATE INDEX IF NOT EXISTS idx_photos_sort_date_v2 ON photos ((DATE(COALESCE(date_taken, created_at) AT TIME ZONE 'UTC')))",
            "CREATE INDEX IF NOT EXISTS idx_exifs_image_id ON exifs (image_id)",
            "CREATE INDEX IF NOT EXISTS idx_photo_comments_photo_id ON photo_comments (photo_id)",
            "CREATE INDEX IF NOT EXISTS idx_album_comments_album_id ON album_comments (album_id)",
            "CREATE TABLE IF NOT EXISTS tags (id BIGSERIAL PRIMARY KEY, name TEXT NOT NULL, name_norm TEXT NOT NULL, visibility SMALLINT NOT NULL DEFAULT 0, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), CONSTRAINT ck_tags_visibility CHECK (visibility IN (0, 1)))",
            "CREATE UNIQUE INDEX IF NOT EXISTS ux_tags_name_norm ON tags (name_norm)",
            "CREATE TABLE IF NOT EXISTS photo_tags (photo_id UUID NOT NULL REFERENCES photos (id) ON DELETE CASCADE, tag_id BIGINT NOT NULL REFERENCES tags (id) ON DELETE CASCADE, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), created_by_user_id UUID NULL REFERENCES users (id) ON DELETE SET NULL, PRIMARY KEY (photo_id, tag_id))",
            "CREATE TABLE IF NOT EXISTS album_tags (album_id UUID NOT NULL REFERENCES albums (id) ON DELETE CASCADE, tag_id BIGINT NOT NULL REFERENCES tags (id) ON DELETE CASCADE, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), created_by_user_id UUID NULL REFERENCES users (id) ON DELETE SET NULL, PRIMARY KEY (album_id, tag_id))",
            "CREATE INDEX IF NOT EXISTS idx_photo_tags_tag_id_photo_id ON photo_tags (tag_id, photo_id)",
            "CREATE INDEX IF NOT EXISTS idx_album_tags_tag_id_album_id ON album_tags (tag_id, album_id)",
            "CREATE OR REPLACE VIEW photos_public_visible AS SELECT p.* FROM photos p WHERE NOT EXISTS (SELECT 1 FROM photo_tags pt JOIN tags t ON t.id = pt.tag_id WHERE pt.photo_id = p.id AND t.visibility = 1)",
        ];

        for sql in sqls {
            if let Err(e) = sqlx::query(sql).execute(&*pool).await {
                log::error!("Failed to execute SQL {}: {}", sql, e);
            }
        }
    }
}
