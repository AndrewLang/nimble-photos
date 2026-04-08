use crate::prelude::*;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::{PostgresEntity, value_builder::PostgresValueBuilder},
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::{FromRow, Row},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineDay {
    pub id: Uuid,
    pub day_date: NaiveDate,

    pub year: i32,
    pub month: i32,

    pub total_count: i32,

    pub min_sort_date: Option<DateTime<Utc>>,
    pub max_sort_date: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
}

impl Entity for TimelineDay {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name() -> &'static str {
        "timeline_day"
    }
}

#[cfg(feature = "postgres")]
impl<'r> FromRow<'r, sqlx::postgres::PgRow> for TimelineDay {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            day_date: row.try_get("day_date")?,
            year: row.try_get("year")?,
            month: row.try_get("month")?,
            total_count: row.try_get("total_count")?,
            min_sort_date: row.try_get("min_sort_date")?,
            max_sort_date: row.try_get("max_sort_date")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for TimelineDay {
    fn id_column() -> &'static str {
        "id"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::Uuid(*id)
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "id",
            "day_date",
            "year",
            "month",
            "total_count",
            "min_sort_date",
            "max_sort_date",
            "created_at",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::Uuid(self.id),
            Value::Date(self.day_date),
            Value::Int(self.year as i64),
            Value::Int(self.month as i64),
            Value::Int(self.total_count as i64),
            PostgresValueBuilder::optional_datetime(&self.min_sort_date),
            PostgresValueBuilder::optional_datetime(&self.max_sort_date),
            Value::DateTime(self.created_at),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "day_date",
            "year",
            "month",
            "total_count",
            "min_sort_date",
            "max_sort_date",
            "created_at",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            Value::Date(self.day_date),
            Value::Int(self.year as i64),
            Value::Int(self.month as i64),
            Value::Int(self.total_count as i64),
            PostgresValueBuilder::optional_datetime(&self.min_sort_date),
            PostgresValueBuilder::optional_datetime(&self.max_sort_date),
            Value::DateTime(self.created_at),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Uuid).primary_key(),
            ColumnDef::new("day_date", ColumnType::Custom("DATE")).not_null(),
            ColumnDef::new("year", ColumnType::Integer).not_null(),
            ColumnDef::new("month", ColumnType::Integer).not_null(),
            ColumnDef::new("total_count", ColumnType::Integer).not_null(),
            ColumnDef::new("min_sort_date", ColumnType::Timestamp),
            ColumnDef::new("max_sort_date", ColumnType::Timestamp),
            ColumnDef::new("created_at", ColumnType::Timestamp)
                .not_null()
                .default("NOW()"),
        ]
    }
}
