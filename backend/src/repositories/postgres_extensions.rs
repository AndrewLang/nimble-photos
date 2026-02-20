pub struct PostgresExtensions;

#[cfg(feature = "postgres")]
use sqlx::Row;
#[cfg(feature = "postgres")]
use sqlx::postgres::PgRow;

#[cfg(feature = "postgres")]
impl PostgresExtensions {
    pub fn optional_i32_as_u32(row: &PgRow, column: &str) -> sqlx::Result<Option<u32>> {
        row.try_get::<Option<i32>, _>(column)
            .map(|opt| opt.map(|value| value as u32))
    }
}
