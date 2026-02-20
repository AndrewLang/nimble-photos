#![cfg(feature = "postgres")]

use nimble_photos::entities::ensure_supporting_schema;
use sqlx::PgPool;

async fn setup_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&url).await.ok()
}

async fn table_exists(pool: &PgPool, name: &str) -> bool {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_name = $1
        )
        "#,
    )
    .bind(name)
    .fetch_one(pool)
    .await
    .expect("table existence query failed")
}

#[tokio::test]
async fn ensure_supporting_schema_creates_tag_tables() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    ensure_supporting_schema(&pool)
        .await
        .expect("supporting schema migration failed");

    assert!(table_exists(&pool, "tags").await, "tags table missing");
    assert!(
        table_exists(&pool, "photo_tags").await,
        "photo_tags table missing"
    );
}
