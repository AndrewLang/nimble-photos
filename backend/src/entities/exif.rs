use chrono::{DateTime, Utc};
use nimble_web::Entity;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "postgres")]
use {
    nimble_web::data::postgres::PostgresEntity,
    nimble_web::data::query::Value,
    nimble_web::data::schema::{ColumnDef, ColumnType},
    sqlx::FromRow,
};

#[cfg_attr(feature = "postgres", derive(FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exif {
    pub hash: String,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub software: Option<String>,
    pub taken_at: Option<DateTime<Utc>>,
    pub digitized_at: Option<DateTime<Utc>>,
    pub timezone_offset: Option<i16>,
    pub exposure_time: Option<f64>,
    pub f_number: Option<f32>,
    pub iso: Option<i32>,
    pub exposure_bias: Option<f32>,
    pub metering_mode: Option<String>,
    pub flash: Option<bool>,
    pub focal_length: Option<f32>,
    pub focal_length_35mm: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub orientation: Option<i16>,
    pub color_space: Option<String>,
    pub white_balance: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f32>,
    pub raw: Option<JsonValue>,
}

impl Entity for Exif {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.hash
    }

    fn name() -> &'static str {
        "exif"
    }
}

#[cfg(feature = "postgres")]
impl PostgresEntity for Exif {
    fn id_column() -> &'static str {
        "hash"
    }

    fn id_value(id: &Self::Id) -> Value {
        Value::String(id.clone())
    }

    fn insert_columns() -> &'static [&'static str] {
        &[
            "hash",
            "camera_make",
            "camera_model",
            "lens_make",
            "lens_model",
            "software",
            "taken_at",
            "digitized_at",
            "timezone_offset",
            "exposure_time",
            "f_number",
            "iso",
            "exposure_bias",
            "metering_mode",
            "flash",
            "focal_length",
            "focal_length_35mm",
            "width",
            "height",
            "orientation",
            "color_space",
            "white_balance",
            "latitude",
            "longitude",
            "altitude",
            "raw",
        ]
    }

    fn insert_values(&self) -> Vec<Value> {
        vec![
            Value::String(self.hash.clone()),
            opt_string(&self.camera_make),
            opt_string(&self.camera_model),
            opt_string(&self.lens_make),
            opt_string(&self.lens_model),
            opt_string(&self.software),
            opt_datetime(self.taken_at),
            opt_datetime(self.digitized_at),
            opt_int(self.timezone_offset.map(|v| v as i64)),
            opt_float(self.exposure_time.map(|v| v as f64)),
            opt_float(self.f_number.map(|v| v as f64)),
            opt_int(self.iso.map(|v| v as i64)),
            opt_float(self.exposure_bias.map(|v| v as f64)),
            opt_string(&self.metering_mode),
            opt_bool(self.flash),
            opt_float(self.focal_length.map(|v| v as f64)),
            opt_int(self.focal_length_35mm.map(|v| v as i64)),
            opt_int(self.width.map(|v| v as i64)),
            opt_int(self.height.map(|v| v as i64)),
            opt_int(self.orientation.map(|v| v as i64)),
            opt_string(&self.color_space),
            opt_string(&self.white_balance),
            opt_float(self.latitude),
            opt_float(self.longitude),
            opt_float(self.altitude.map(|v| v as f64)),
            opt_json(&self.raw),
        ]
    }

    fn update_columns() -> &'static [&'static str] {
        &[
            "camera_make",
            "camera_model",
            "lens_make",
            "lens_model",
            "software",
            "taken_at",
            "digitized_at",
            "timezone_offset",
            "exposure_time",
            "f_number",
            "iso",
            "exposure_bias",
            "metering_mode",
            "flash",
            "focal_length",
            "focal_length_35mm",
            "width",
            "height",
            "orientation",
            "color_space",
            "white_balance",
            "latitude",
            "longitude",
            "altitude",
            "raw",
        ]
    }

    fn update_values(&self) -> Vec<Value> {
        vec![
            opt_string(&self.camera_make),
            opt_string(&self.camera_model),
            opt_string(&self.lens_make),
            opt_string(&self.lens_model),
            opt_string(&self.software),
            opt_datetime(self.taken_at),
            opt_datetime(self.digitized_at),
            opt_int(self.timezone_offset.map(|v| v as i64)),
            opt_float(self.exposure_time.map(|v| v as f64)),
            opt_float(self.f_number.map(|v| v as f64)),
            opt_int(self.iso.map(|v| v as i64)),
            opt_float(self.exposure_bias.map(|v| v as f64)),
            opt_string(&self.metering_mode),
            opt_bool(self.flash),
            opt_float(self.focal_length.map(|v| v as f64)),
            opt_int(self.focal_length_35mm.map(|v| v as i64)),
            opt_int(self.width.map(|v| v as i64)),
            opt_int(self.height.map(|v| v as i64)),
            opt_int(self.orientation.map(|v| v as i64)),
            opt_string(&self.color_space),
            opt_string(&self.white_balance),
            opt_float(self.latitude),
            opt_float(self.longitude),
            opt_float(self.altitude.map(|v| v as f64)),
            opt_json(&self.raw),
        ]
    }

    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("hash", ColumnType::Text).primary_key(),
            ColumnDef::new("camera_make", ColumnType::Text),
            ColumnDef::new("camera_model", ColumnType::Text),
            ColumnDef::new("lens_make", ColumnType::Text),
            ColumnDef::new("lens_model", ColumnType::Text),
            ColumnDef::new("software", ColumnType::Text),
            ColumnDef::new("taken_at", ColumnType::Timestamp),
            ColumnDef::new("digitized_at", ColumnType::Timestamp),
            ColumnDef::new("timezone_offset", ColumnType::Integer),
            ColumnDef::new("exposure_time", ColumnType::Double),
            ColumnDef::new("f_number", ColumnType::Float),
            ColumnDef::new("iso", ColumnType::Integer),
            ColumnDef::new("exposure_bias", ColumnType::Float),
            ColumnDef::new("metering_mode", ColumnType::Text),
            ColumnDef::new("flash", ColumnType::Boolean),
            ColumnDef::new("focal_length", ColumnType::Float),
            ColumnDef::new("focal_length_35mm", ColumnType::Integer),
            ColumnDef::new("width", ColumnType::Integer),
            ColumnDef::new("height", ColumnType::Integer),
            ColumnDef::new("orientation", ColumnType::Integer),
            ColumnDef::new("color_space", ColumnType::Text),
            ColumnDef::new("white_balance", ColumnType::Text),
            ColumnDef::new("latitude", ColumnType::Double),
            ColumnDef::new("longitude", ColumnType::Double),
            ColumnDef::new("altitude", ColumnType::Float),
            ColumnDef::new("raw", ColumnType::Json),
        ]
    }
}

fn opt_string(value: &Option<String>) -> Value {
    match value {
        Some(v) => Value::String(v.clone()),
        None => Value::Null,
    }
}

fn opt_datetime(value: Option<DateTime<Utc>>) -> Value {
    match value {
        Some(v) => Value::DateTime(v),
        None => Value::Null,
    }
}

fn opt_int(value: Option<i64>) -> Value {
    match value {
        Some(v) => Value::Int(v),
        None => Value::Null,
    }
}

fn opt_float(value: Option<f64>) -> Value {
    match value {
        Some(v) => Value::Float(v),
        None => Value::Null,
    }
}

fn opt_bool(value: Option<bool>) -> Value {
    match value {
        Some(v) => Value::Bool(v),
        None => Value::Null,
    }
}

fn opt_json(value: &Option<JsonValue>) -> Value {
    match value {
        Some(v) => Value::String(v.to_string()),
        None => Value::Null,
    }
}
