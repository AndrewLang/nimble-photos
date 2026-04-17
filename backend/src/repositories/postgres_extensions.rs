pub struct PostgresExtensions;

#[cfg(feature = "postgres")]
use sqlx::Row;
#[cfg(feature = "postgres")]
use sqlx::postgres::PgRow;

#[cfg(feature = "postgres")]
impl PostgresExtensions {
    pub fn optional_i32_as_u32(row: &PgRow, column: &str) -> sqlx::Result<Option<u32>> {
        row.try_get::<Option<i32>, _>(column).map(|opt| opt.map(|value| value as u32))
    }

    pub fn optional_i32_as_i32(row: &PgRow, column: &str) -> sqlx::Result<Option<i32>> {
        row.try_get(column)
    }

    pub fn optional_i32_as_u16(row: &PgRow, column: &str) -> sqlx::Result<Option<u16>> {
        row.try_get::<Option<i32>, _>(column).map(|opt| opt.map(|value| value as u16))
    }

    pub fn optional_i32_as_u8(row: &PgRow, column: &str) -> sqlx::Result<Option<u8>> {
        row.try_get::<Option<i32>, _>(column).map(|opt| opt.map(|value| value as u8))
    }

    pub fn optional_i32_as_i8(row: &PgRow, column: &str) -> sqlx::Result<Option<i8>> {
        row.try_get::<Option<i32>, _>(column).map(|opt| opt.map(|value| value as i8))
    }
}
