use crate::entities::exif::ExifModel;
use exif::{Exif, Field, Rational, Reader, SRational, Tag, Value};
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Seek};
use std::path::Path;

#[derive(Debug)]
pub struct ExifService;

impl ExifService {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_from_path<P: AsRef<Path>>(&self, path: P) -> ExifModel {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return ExifModel::default(),
        };

        let mut reader = BufReader::new(file);
        self.extract_from_reader(&mut reader)
    }

    pub fn extract_from_bytes(&self, bytes: &[u8]) -> ExifModel {
        let mut reader = Cursor::new(bytes);
        self.extract_from_reader(&mut reader)
    }

    fn extract_from_reader<R: BufRead + Seek>(&self, reader: &mut R) -> ExifModel {
        let exif = match Reader::new().read_from_container(reader) {
            Ok(exif) => exif,
            Err(_) => return ExifModel::default(),
        };

        self.map_model(&exif)
    }

    fn map_model(&self, exif: &Exif) -> ExifModel {
        ExifModel {
            make: self.text_from_field(exif, Tag::Make),
            model: self.text_from_field(exif, Tag::Model),
            lens_make: self.text_from_field(exif, Tag::LensMake),
            lens_model: self.text_from_field(exif, Tag::LensModel),
            lens_serial_number: self.text_from_field(exif, Tag::LensSerialNumber),
            body_serial_number: self.text_from_field(exif, Tag::BodySerialNumber),
            exposure_time: self.text_from_field(exif, Tag::ExposureTime),
            f_number: self.f32_from_field(exif, Tag::FNumber),
            aperture_value: self.f32_from_field(exif, Tag::ApertureValue),
            max_aperture_value: self.f32_from_field(exif, Tag::MaxApertureValue),
            brightness_value: self.f32_from_field(exif, Tag::BrightnessValue),
            shutter_speed_value: self.f32_from_field(exif, Tag::ShutterSpeedValue),
            focal_length: self.f32_from_field(exif, Tag::FocalLength),
            image_width: self.u32_from_field(exif, Tag::ImageWidth),
            image_length: self.u32_from_field(exif, Tag::ImageLength),
            pixel_x_dimension: self.u32_from_field(exif, Tag::PixelXDimension),
            pixel_y_dimension: self.u32_from_field(exif, Tag::PixelYDimension),
            orientation: self.u16_from_field(exif, Tag::Orientation),
            datetime: self.text_from_field(exif, Tag::DateTime),
            datetime_original: self.text_from_field(exif, Tag::DateTimeOriginal),
            datetime_digitized: self.text_from_field(exif, Tag::DateTimeDigitized),
            gps_latitude: self.gps_coordinate(exif, Tag::GPSLatitude, Tag::GPSLatitudeRef),
            gps_longitude: self.gps_coordinate(exif, Tag::GPSLongitude, Tag::GPSLongitudeRef),
            gps_altitude: self.gps_altitude(exif),
            gps_altitude_ref: self.text_from_field(exif, Tag::GPSAltitudeRef),
            gps_latitude_ref: self.text_from_field(exif, Tag::GPSLatitudeRef),
            gps_longitude_ref: self.text_from_field(exif, Tag::GPSLongitudeRef),
            software: self.text_from_field(exif, Tag::Software),
            artist: self.text_from_field(exif, Tag::Artist),
            copyright: self.text_from_field(exif, Tag::Copyright),
            ..ExifModel::default()
        }
    }

    fn field<'a>(&self, exif: &'a Exif, tag: Tag) -> Option<&'a Field> {
        exif.fields().find(|field| field.tag == tag)
    }

    fn text_from_field(&self, exif: &Exif, tag: Tag) -> Option<String> {
        let field = self.field(exif, tag)?;
        let text = match &field.value {
            Value::Ascii(values) => values
                .iter()
                .find_map(|entry| std::str::from_utf8(entry).ok())
                .map(str::to_string)
                .unwrap_or_default(),
            _ => field.display_value().with_unit(exif).to_string(),
        };
        self.non_empty_string(text)
    }

    fn u32_from_field(&self, exif: &Exif, tag: Tag) -> Option<u32> {
        let field = self.field(exif, tag)?;
        match &field.value {
            Value::Long(values) => values.first().copied(),
            Value::Short(values) => values.first().map(|value| *value as u32),
            Value::Byte(values) => values.first().map(|value| *value as u32),
            Value::SLong(values) => values.first().and_then(|value| (*value).try_into().ok()),
            _ => self
                .text_from_field(exif, tag)
                .and_then(|value| value.parse::<u32>().ok()),
        }
    }

    fn u16_from_field(&self, exif: &Exif, tag: Tag) -> Option<u16> {
        let field = self.field(exif, tag)?;
        match &field.value {
            Value::Short(values) => values.first().copied(),
            Value::Long(values) => values.first().and_then(|value| (*value).try_into().ok()),
            Value::Byte(values) => values.first().map(|value| *value as u16),
            _ => self
                .text_from_field(exif, tag)
                .and_then(|value| value.parse::<u16>().ok()),
        }
    }

    fn f32_from_field(&self, exif: &Exif, tag: Tag) -> Option<f32> {
        let field = self.field(exif, tag)?;
        match &field.value {
            Value::Rational(values) => values
                .first()
                .and_then(|value| Self::rational_to_f64(*value))
                .map(|value| value as f32),
            Value::SRational(values) => values
                .first()
                .and_then(|value| Self::srational_to_f64(*value))
                .map(|value| value as f32),
            Value::Float(values) => values.first().copied(),
            Value::Double(values) => values.first().map(|value| *value as f32),
            Value::Long(values) => values.first().map(|value| *value as f32),
            Value::Short(values) => values.first().map(|value| *value as f32),
            Value::SLong(values) => values.first().map(|value| *value as f32),
            _ => self
                .text_from_field(exif, tag)
                .and_then(|value| value.parse::<f32>().ok()),
        }
    }

    fn gps_coordinate(&self, exif: &Exif, coordinate_tag: Tag, reference_tag: Tag) -> Option<f64> {
        let field = self.field(exif, coordinate_tag)?;
        let values = match &field.value {
            Value::Rational(values) if values.len() >= 3 => values,
            _ => return None,
        };

        let degrees = Self::rational_to_f64(values[0])?;
        let minutes = Self::rational_to_f64(values[1])?;
        let seconds = Self::rational_to_f64(values[2])?;
        let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

        let reference = self.text_from_field(exif, reference_tag)?;
        let reference_upper = reference.to_ascii_uppercase();
        if reference_upper == "S" || reference_upper == "W" {
            decimal = -decimal;
        }

        Some(decimal)
    }

    fn gps_altitude(&self, exif: &Exif) -> Option<f64> {
        let altitude = match self.field(exif, Tag::GPSAltitude).map(|field| &field.value) {
            Some(Value::Rational(values)) => values
                .first()
                .and_then(|value| Self::rational_to_f64(*value))?,
            Some(Value::SRational(values)) => values
                .first()
                .and_then(|value| Self::srational_to_f64(*value))?,
            _ => return None,
        };

        let reference = self
            .field(exif, Tag::GPSAltitudeRef)
            .and_then(|field| match &field.value {
                Value::Byte(values) => values.first().copied(),
                Value::Short(values) => values.first().map(|value| *value as u8),
                _ => None,
            })
            .unwrap_or(0);

        if reference == 1 {
            Some(-altitude)
        } else {
            Some(altitude)
        }
    }

    fn non_empty_string(&self, value: String) -> Option<String> {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    fn rational_to_f64(value: Rational) -> Option<f64> {
        if value.denom == 0 {
            None
        } else {
            Some(value.num as f64 / value.denom as f64)
        }
    }

    fn srational_to_f64(value: SRational) -> Option<f64> {
        if value.denom == 0 {
            None
        } else {
            Some(value.num as f64 / value.denom as f64)
        }
    }
}
