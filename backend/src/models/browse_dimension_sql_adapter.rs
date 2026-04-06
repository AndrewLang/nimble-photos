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
            BrowseDimension::Year => ("p.year AS folder", "p.year"),
            BrowseDimension::Date => (
                "concat(p.year::text, '-', p.month_day) AS folder",
                "concat(p.year::text, '-', p.month_day)",
            ),
            BrowseDimension::Month => (
                "concat(p.year::text, '-', split_part(p.month_day, '-', 1)) AS folder",
                "concat(p.year::text, '-', split_part(p.month_day, '-', 1))",
            ),
            BrowseDimension::Camera => ("COALESCE(p.model, p.make) AS folder", "COALESCE(p.model, p.make)"),
            BrowseDimension::Rating => ("p.rating AS folder", "p.rating"),
        }
    }

    pub fn filter_clause(&self, param_index: usize) -> String {
        match self.dimension {
            BrowseDimension::Year => format!("p.year = ${}", param_index),
            BrowseDimension::Date => {
                format!("concat(p.year::text, '-', p.month_day) = ${}", param_index)
            }
            BrowseDimension::Month => {
                format!(
                    "concat(p.year::text, '-', split_part(p.month_day, '-', 1)) = ${}",
                    param_index
                )
            }
            BrowseDimension::Camera => format!("COALESCE(p.model, p.make) = ${}", param_index),
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
