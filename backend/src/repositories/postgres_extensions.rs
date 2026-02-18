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

    pub fn optional_string_allow_missing(
        row: &PgRow,
        column: &str,
    ) -> sqlx::Result<Option<String>> {
        match row.try_get::<Option<String>, _>(column) {
            Ok(value) => Ok(value),
            Err(sqlx::Error::ColumnNotFound(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
