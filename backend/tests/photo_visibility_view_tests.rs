use nimble_photos::repositories::photo::{PhotoRepository, PostgresPhotoRepository};
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    Some(pool)
}

async fn ensure_tag_schema_and_view(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id BIGSERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            name_norm TEXT NOT NULL,
            visibility SMALLINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS ux_tags_name_norm ON tags (name_norm)")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS photo_tags (
            photo_id UUID NOT NULL REFERENCES photos (id) ON DELETE CASCADE,
            tag_id BIGINT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_by_user_id UUID NULL REFERENCES users (id) ON DELETE SET NULL,
            PRIMARY KEY (photo_id, tag_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE VIEW photos_public_visible AS
        SELECT p.*
        FROM photos p
        WHERE NOT EXISTS (
            SELECT 1
            FROM photo_tags pt
            JOIN tags t ON t.id = pt.tag_id
            WHERE pt.photo_id = p.id
              AND t.visibility = 1
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::test]
async fn photo_visibility_view_hides_and_unhides_with_admin_only_tag() {
    let Some(pool) = setup_pool().await else {
        return;
    };
    ensure_tag_schema_and_view(&pool)
        .await
        .expect("schema/view setup failed");

    let repo = PostgresPhotoRepository::new(pool.clone());
    let photo_id = Uuid::new_v4();
    let tag_name = format!("admin-only-{}", Uuid::new_v4());

    sqlx::query(
        r#"
        INSERT INTO photos (id, path, name, created_at)
        VALUES ($1, $2, $3, NOW())
        "#,
    )
    .bind(photo_id)
    .bind(format!("/tmp/{}.jpg", photo_id))
    .bind(format!("{}.jpg", photo_id))
    .execute(&pool)
    .await
    .expect("insert photo failed");

    let visible_before = repo
        .get_by_ids(&[photo_id], false)
        .await
        .expect("query visible before tag failed");
    assert_eq!(visible_before.len(), 1);

    let tag_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO tags (name, name_norm, visibility, created_at)
        VALUES ($1, $2, 1, NOW())
        ON CONFLICT (name_norm) DO UPDATE SET visibility = 1
        RETURNING id
        "#,
    )
    .bind(&tag_name)
    .bind(tag_name.to_lowercase())
    .fetch_one(&pool)
    .await
    .expect("upsert admin_only tag failed");

    sqlx::query(
        "INSERT INTO photo_tags (photo_id, tag_id, created_at) VALUES ($1, $2, NOW()) ON CONFLICT DO NOTHING",
    )
    .bind(photo_id)
    .bind(tag_id)
    .execute(&pool)
    .await
    .expect("attach tag failed");

    let visible_after = repo
        .get_by_ids(&[photo_id], false)
        .await
        .expect("query non-admin after tag failed");
    assert!(visible_after.is_empty());

    let admin_visible = repo
        .get_by_ids(&[photo_id], true)
        .await
        .expect("query admin after tag failed");
    assert_eq!(admin_visible.len(), 1);

    sqlx::query("DELETE FROM photo_tags WHERE photo_id = $1 AND tag_id = $2")
        .bind(photo_id)
        .bind(tag_id)
        .execute(&pool)
        .await
        .expect("remove tag failed");

    let visible_again = repo
        .get_by_ids(&[photo_id], false)
        .await
        .expect("query non-admin after remove failed");
    assert_eq!(visible_again.len(), 1);

    let _ = sqlx::query("DELETE FROM photo_tags WHERE photo_id = $1")
        .bind(photo_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM photos WHERE id = $1")
        .bind(photo_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id)
        .execute(&pool)
        .await;
}

