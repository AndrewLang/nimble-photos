use anyhow::Result;

use crate::entities::photo_browse::{BrowseDimension, SortDirection};

#[derive(Debug, PartialEq, Eq)]
pub enum SqlParam {
    Int(i32),
    String(String),
}

pub struct BrowseDimensionSqlAdapter {
    dimension: BrowseDimension,
}

impl BrowseDimensionSqlAdapter {
    pub fn new(dimension: BrowseDimension) -> Self {
        Self { dimension }
    }

    pub fn group_select(&self) -> (&'static str, &'static str) {
        match self.dimension {
            BrowseDimension::Year => (
                "EXTRACT(YEAR FROM COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC')::int AS folder",
                "EXTRACT(YEAR FROM COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC')::int",
            ),
            BrowseDimension::Date => (
                "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD') AS folder",
                "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD')",
            ),
            BrowseDimension::Month => (
                "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM') AS folder",
                "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM')",
            ),
            BrowseDimension::Camera => ("p.camera_model AS folder", "p.camera_model"),
            BrowseDimension::Rating => ("p.rating AS folder", "p.rating"),
        }
    }

    pub fn filter_clause(&self, param_index: usize) -> String {
        match self.dimension {
            BrowseDimension::Year => {
                format!(
                    "EXTRACT(YEAR FROM COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC')::int = ${}",
                    param_index
                )
            }
            BrowseDimension::Date => format!(
                "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM-DD') = ${}",
                param_index
            ),
            BrowseDimension::Month => {
                format!(
                    "to_char(COALESCE(p.date_taken, p.created_at) AT TIME ZONE 'UTC', 'YYYY-MM') = ${}",
                    param_index
                )
            }
            BrowseDimension::Camera => format!("p.camera_model = ${}", param_index),
            BrowseDimension::Rating => format!("p.rating = ${}", param_index),
        }
    }

    pub fn parse_segment_value(&self, segment: &str) -> Result<SqlParam> {
        match self.dimension {
            BrowseDimension::Year => {
                let year: i32 = segment.parse()?;
                Ok(SqlParam::Int(year))
            }
            BrowseDimension::Date => {
                chrono::NaiveDate::parse_from_str(segment, "%Y-%m-%d")?;
                Ok(SqlParam::String(segment.to_string()))
            }
            BrowseDimension::Month => {
                chrono::NaiveDate::parse_from_str(&(segment.to_string() + "-01"), "%Y-%m-%d")?;
                Ok(SqlParam::String(segment.to_string()))
            }
            BrowseDimension::Camera => Ok(SqlParam::String(segment.to_string())),
            BrowseDimension::Rating => {
                let rating: i32 = segment.parse()?;
                Ok(SqlParam::Int(rating))
            }
        }
    }

    pub fn order_direction(direction: &SortDirection) -> &'static str {
        match direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}
