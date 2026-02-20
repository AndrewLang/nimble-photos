use album::Album;
use album_comment::AlbumComment;
use client::Client;
use client_storage::ClientStorage;
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
use anyhow::{Result, anyhow};
#[cfg(feature = "postgres")]
use nimble_web::data::postgres::PostgresEntity;

pub use storage_location::StorageLocation;

pub mod album;
pub mod album_comment;
pub mod album_hooks;
pub mod client;
pub mod client_storage;
pub mod exif;
pub mod permission;
pub mod photo;
pub mod photo_browse;
pub mod photo_comment;
pub mod photo_cursor;
pub mod photo_tag;
pub mod setting;
pub mod storage_location;
pub mod tag;
pub mod user;
pub mod user_settings;
pub mod uuid_id;

pub fn register_entities(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.use_entity_with_operations::<StorageLocation>(&EntityOperation::all());
    builder.use_entity_with_operations_and_policy::<User>(
        &[EntityOperation::Get, EntityOperation::List],
        Policy::Authenticated,
    );
    builder.use_entity_with_operations_and_policy::<Client>(
        &[EntityOperation::Get, EntityOperation::List],
        Policy::Authenticated,
    );
    builder.use_entity_with_operations_and_policy::<ClientStorage>(
        &[EntityOperation::Get, EntityOperation::List],
        Policy::Authenticated,
    );
    builder.use_entity_with_operations::<UserSettings>(&[
        EntityOperation::Get,
        EntityOperation::Update,
    ]);
    builder.use_entity_with_operations::<Photo>(&EntityOperation::all());
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
            let provider = MemoryRepository::<Client>::new();
            Repository::<Client>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<ClientStorage>::new();
            Repository::<ClientStorage>::new(Box::new(provider))
        });
        builder.register_singleton(|_| {
            let provider = MemoryRepository::<StorageLocation>::new();
            Repository::<StorageLocation>::new(Box::new(provider))
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
            let provider = PostgresProvider::<Client>::new((*pool).clone());
            Repository::<Client>::new(Box::new(provider))
        });
        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<ClientStorage>::new((*pool).clone());
            Repository::<ClientStorage>::new(Box::new(provider))
        });
        builder.register_singleton(|p| {
            let pool = p.get::<PgPool>();
            let provider = PostgresProvider::<StorageLocation>::new((*pool).clone());
            Repository::<StorageLocation>::new(Box::new(provider))
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

pub async fn migrate_entities(app: &Application) -> Result<()> {
    #[cfg(not(feature = "postgres"))]
    {
        let _ = app;
        return Ok(());
    }

    #[cfg(feature = "postgres")]
    {
        migrate_entity::<User>(app).await?;
        migrate_entity::<Client>(app).await?;
        migrate_entity::<ClientStorage>(app).await?;
        migrate_entity::<StorageLocation>(app).await?;
        migrate_entity::<UserSettings>(app).await?;
        migrate_entity::<Photo>(app).await?;
        migrate_entity::<Album>(app).await?;
        migrate_entity::<ExifModel>(app).await?;
        migrate_entity::<PhotoComment>(app).await?;
        migrate_entity::<AlbumComment>(app).await?;
        migrate_entity::<Setting>(app).await?;

        let pool = app
            .services()
            .resolve::<sqlx::PgPool>()
            .ok_or_else(|| anyhow!("PgPool not found in service provider"))?;

        log::info!("Creating additional indices for performance...");
        ensure_supporting_schema(pool.as_ref()).await?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(feature = "postgres")]
async fn migrate_entity<E>(app: &Application) -> Result<()>
where
    E: PostgresEntity,
{
    log::info!("Migrating entity [{}] ...", E::plural_name());
    app.migrate_entity::<E>()
        .await
        .map_err(|err| anyhow!("Failed to migrate {}: {:?}", E::plural_name(), err))?;
    Ok(())
}

#[cfg(feature = "postgres")]
pub async fn ensure_supporting_schema(pool: &sqlx::PgPool) -> Result<()> {
    let sqls = [
        "CREATE EXTENSION IF NOT EXISTS \"pgcrypto\"",
        "ALTER TABLE clientstorages ADD COLUMN IF NOT EXISTS id UUID",
        r#"DO $$ BEGIN
                IF EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'clientstorages'
                    AND column_name = 'id'
                    AND data_type <> 'uuid'
                ) THEN
                    EXECUTE 'ALTER TABLE clientstorages ALTER COLUMN id TYPE UUID USING gen_random_uuid()';
                END IF;
            END $$;"#,
        "ALTER TABLE clientstorages ALTER COLUMN id SET DEFAULT gen_random_uuid()",
        "UPDATE clientstorages SET id = gen_random_uuid() WHERE id IS NULL",
        "ALTER TABLE clientstorages DROP CONSTRAINT IF EXISTS clientstorages_pkey",
        "ALTER TABLE clientstorages ADD CONSTRAINT clientstorages_pkey PRIMARY KEY (id)",
        "CREATE UNIQUE INDEX IF NOT EXISTS ux_clientstorages_client_storage ON clientstorages (client_id, storage_id)",
        "CREATE INDEX IF NOT EXISTS idx_photos_day_taken ON photos (day_date DESC, date_taken DESC)",
        "CREATE INDEX IF NOT EXISTS idx_photos_hash ON photos(hash)",
        "CREATE INDEX IF NOT EXISTS idx_photos_storage ON photos(storage_id)",
        "CREATE INDEX IF NOT EXISTS idx_exifs_image_id ON exifs (image_id)",
        "CREATE INDEX IF NOT EXISTS idx_photo_comments_photo_id ON photo_comments (photo_id)",
        "CREATE INDEX IF NOT EXISTS idx_album_comments_album_id ON album_comments (album_id)",
        "CREATE TABLE IF NOT EXISTS tags (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name TEXT NOT NULL, name_norm TEXT NOT NULL, visibility SMALLINT NOT NULL DEFAULT 0, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), CONSTRAINT ck_tags_visibility CHECK (visibility IN (0, 1)))",
        "CREATE UNIQUE INDEX IF NOT EXISTS ux_tags_name_norm ON tags (name_norm)",
        "CREATE INDEX IF NOT EXISTS idx_tags_name ON tags (name)",
        "CREATE TABLE IF NOT EXISTS photo_tags (photo_id UUID NOT NULL REFERENCES photos (id) ON DELETE CASCADE, tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE, PRIMARY KEY (photo_id, tag_id))",
        "CREATE TABLE IF NOT EXISTS album_tags (album_id UUID NOT NULL REFERENCES albums (id) ON DELETE CASCADE, tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), created_by_user_id UUID NULL REFERENCES users (id) ON DELETE SET NULL, PRIMARY KEY (album_id, tag_id))",
        "CREATE INDEX IF NOT EXISTS idx_photo_tags_photo ON photo_tags (photo_id)",
        "CREATE INDEX IF NOT EXISTS idx_photo_tags_tag ON photo_tags (tag_id)",
        "CREATE INDEX IF NOT EXISTS idx_album_tags_tag_id_album_id ON album_tags (tag_id, album_id)",
        "CREATE OR REPLACE VIEW photos_public_visible AS SELECT p.* FROM photos p WHERE NOT EXISTS (SELECT 1 FROM photo_tags pt JOIN tags t ON t.id = pt.tag_id WHERE pt.photo_id = p.id AND t.visibility = 1)",
    ];

    for sql in sqls {
        sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|err| anyhow!("Failed to execute SQL '{}': {}", sql, err))?;
    }

    Ok(())
}
