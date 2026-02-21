use crate::entities::exif::ExifModel;
use crate::services::image_process_constants::ImageProcessKeys;

use exif::{Reader, Tag};
use once_cell::sync::Lazy;
use quickraw::{Export, Input};
use std::collections::HashMap;
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
        let path_ref = path.as_ref();
        let file = match File::open(path_ref) {
            Ok(file) => file,
            Err(_) => return ExifModel::default(),
        };

        let mut reader = BufReader::new(file);
        let mut metadata = self.extract_from_reader(&mut reader);

        if self.is_raw(path_ref) {
            let raw_metadata = self.extract_raw_metadata(path_ref);
            metadata.extend(raw_metadata);
        }

        let model = self.build_exif(&metadata);
        model
    }

    pub fn extract_from_bytes(&self, bytes: &[u8]) -> ExifModel {
        let mut reader = Cursor::new(bytes);
        let fields = self.extract_from_reader(&mut reader);
        let model = self.build_exif(&fields);

        model
    }

    fn extract_from_reader<R: BufRead + Seek>(&self, reader: &mut R) -> HashMap<String, String> {
        let exif = match Reader::new().read_from_container(reader) {
            Ok(exif) => exif,
            Err(_) => return HashMap::new(),
        };

        exif.fields()
            .map(|field| (field.tag.to_string(), field.display_value().to_string()))
            .collect()
    }

    fn build_exif(&self, fields: &HashMap<String, String>) -> ExifModel {
        ExifModel {
            make: self.text_from_field(fields, Tag::Make.to_string()),
            model: self.text_from_field(fields, Tag::Model.to_string()),
            lens_make: self.text_from_field(fields, Tag::LensMake.to_string()),
            lens_model: self.text_from_field(fields, Tag::LensModel.to_string()),
            lens_serial_number: self.text_from_field(fields, Tag::LensSerialNumber.to_string()),
            body_serial_number: self.text_from_field(fields, Tag::BodySerialNumber.to_string()),
            exposure_time: self.text_from_field(fields, Tag::ExposureTime.to_string()),
            f_number: self.f32_from_field(fields, Tag::FNumber.to_string()),
            aperture_value: self.f32_from_field(fields, Tag::ApertureValue.to_string()),
            max_aperture_value: self.f32_from_field(fields, Tag::MaxApertureValue.to_string()),
            brightness_value: self.f32_from_field(fields, Tag::BrightnessValue.to_string()),
            shutter_speed_value: self.f32_from_field(fields, Tag::ShutterSpeedValue.to_string()),
            focal_length: self.f32_from_field(fields, Tag::FocalLength.to_string()),
            image_width: self.u32_from_field(fields, Tag::ImageWidth.to_string()),
            image_length: self.u32_from_field(fields, Tag::ImageLength.to_string()),
            pixel_x_dimension: self.u32_from_field(fields, Tag::PixelXDimension.to_string()),
            pixel_y_dimension: self.u32_from_field(fields, Tag::PixelYDimension.to_string()),
            orientation: self.u16_from_field(fields, Tag::Orientation.to_string()),
            datetime: self.text_from_field(fields, Tag::DateTime.to_string()),
            datetime_original: self.text_from_field(fields, Tag::DateTimeOriginal.to_string()),
            datetime_digitized: self.text_from_field(fields, Tag::DateTimeDigitized.to_string()),
            gps_latitude: self.gps_coordinate(
                fields,
                Tag::GPSLatitude.to_string(),
                Tag::GPSLatitudeRef.to_string(),
            ),
            gps_longitude: self.gps_coordinate(
                fields,
                Tag::GPSLongitude.to_string(),
                Tag::GPSLongitudeRef.to_string(),
            ),
            gps_altitude: self.gps_altitude(fields),
            gps_altitude_ref: self.text_from_field(fields, Tag::GPSAltitudeRef.to_string()),
            gps_latitude_ref: self.text_from_field(fields, Tag::GPSLatitudeRef.to_string()),
            gps_longitude_ref: self.text_from_field(fields, Tag::GPSLongitudeRef.to_string()),
            software: self.text_from_field(fields, Tag::Software.to_string()),
            artist: self.text_from_field(fields, Tag::Artist.to_string()),
            copyright: self.text_from_field(fields, Tag::Copyright.to_string()),
            ..ExifModel::default()
        }
    }

    fn field<'a>(&self, fields: &'a HashMap<String, String>, tag: String) -> Option<&'a String> {
        fields.get(&tag)
    }

    fn text_from_field(&self, fields: &HashMap<String, String>, tag: String) -> Option<String> {
        let field = self.field(fields, tag)?;
        self.non_empty_string(field.clone())
    }

    fn u32_from_field(&self, fields: &HashMap<String, String>, tag: String) -> Option<u32> {
        let field = self.field(fields, tag)?;
        Self::parse_f64_token(field).and_then(|value| {
            if value.is_finite() && value >= 0.0 {
                Some(value as u32)
            } else {
                None
            }
        })
    }

    fn u16_from_field(&self, fields: &HashMap<String, String>, tag: String) -> Option<u16> {
        let field = self.field(fields, tag)?;
        Self::parse_f64_token(field).and_then(|value| {
            if value.is_finite() && value >= 0.0 {
                Some(value as u16)
            } else {
                None
            }
        })
    }

    fn f32_from_field(&self, fields: &HashMap<String, String>, tag: String) -> Option<f32> {
        let field = self.field(fields, tag)?;
        Self::parse_f64_token(field).map(|value| value as f32)
    }

    fn gps_coordinate(
        &self,
        fields: &HashMap<String, String>,
        coordinate_tag: String,
        reference_tag: String,
    ) -> Option<f64> {
        let field = self.field(fields, coordinate_tag)?;
        let mut values = Self::extract_numeric_values(field).into_iter();
        let degrees = values.next()?;
        let minutes = values.next().unwrap_or(0.0);
        let seconds = values.next().unwrap_or(0.0);
        let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

        let reference = self.text_from_field(fields, reference_tag)?;
        let reference_upper = reference.to_ascii_uppercase();
        if reference_upper == "S" || reference_upper == "W" {
            decimal = -decimal;
        }

        Some(decimal)
    }

    fn gps_altitude(&self, fields: &HashMap<String, String>) -> Option<f64> {
        let altitude = self
            .field(fields, Tag::GPSAltitude.to_string())
            .and_then(|value| Self::parse_f64_token(value))?;

        let reference = self
            .field(fields, Tag::GPSAltitudeRef.to_string())
            .and_then(|value| Self::parse_f64_token(value))
            .map(|value| value as u8)
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

    fn extract_numeric_values(text: &str) -> Vec<f64> {
        text.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '/' || c == '-'))
            .filter(|token| !token.is_empty())
            .filter_map(Self::parse_f64_token)
            .collect()
    }

    fn parse_f64_token(value: &str) -> Option<f64> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some((num, den)) = trimmed.split_once('/') {
            let numerator = num.trim().parse::<f64>().ok()?;
            let denominator = den.trim().parse::<f64>().ok()?;
            if denominator == 0.0 {
                None
            } else {
                Some(numerator / denominator)
            }
        } else {
            trimmed.parse::<f64>().ok()
        }
    }

    fn is_raw<P: AsRef<Path>>(&self, path: P) -> bool {
        let extension = match path.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some(ext) => ext,
            None => return false,
        };

        ImageProcessKeys::RAW_EXTENSIONS
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&extension))
    }

    fn exif_tag_name_map(&self) -> &'static HashMap<&'static str, &'static str> {
        static MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
            HashMap::from([
                ("make", "Make"),
                ("model", "Model"),
                ("orientation", "Orientation"),
                ("bps", "BitsPerSample"),
                ("compression", "Compression"),
                ("cfa_pattern", "CFAPattern"),
                ("black_level", "BlackLevel"),
                ("height", "ImageLength"),
                ("width", "ImageWidth"),
                ("crop_height", "DefaultCropHeight"),
                ("crop_width", "DefaultCropWidth"),
                ("crop_left", "DefaultCropLeft"),
                ("crop_top", "DefaultCropTop"),
                ("strip", "StripOffsets"),
                ("strip_len", "StripByteCounts"),
                ("maker_notes", "MakerNote"),
                ("endianness", "ByteOrder"),
                ("contrast_curve_offset", "ContrastCurveOffset"),
                ("white_balance_r", "WhiteBalanceRed"),
                ("white_balance_g", "WhiteBalanceGreen"),
                ("white_balance_b", "WhiteBalanceBlue"),
                ("linear_table_len", "LinearTableLen"),
                ("contrast_curve_len", "ContrastCurveLen"),
                ("linear_table_offset", "LinearTableOffset"),
            ])
        });
        &MAP
    }

    fn extract_meaningful_value(&self, value: &str) -> String {
        let first_part = value.split('/').next().unwrap_or(value).trim();
        first_part
            .trim_matches(|c: char| c == '[' || c == ']' || c.is_whitespace())
            .to_string()
    }

    fn parse_quickraw_exif(&self, raw_text: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let tag_name_map = self.exif_tag_name_map();

        for line in raw_text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key_trimmed = key.trim();

                let mapped_name = tag_name_map
                    .get(key_trimmed)
                    .cloned()
                    .unwrap_or_else(|| key_trimmed);

                let raw_value = value.trim();
                let cleaned_value = self.extract_meaningful_value(raw_value);
                map.insert(mapped_name.to_string(), cleaned_value);
            }
        }
        map
    }

    fn extract_raw_metadata<P: AsRef<Path>>(&self, path: P) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        let bytes = std::fs::read(&path).unwrap_or_default();
        let input_source = Input::ByBuffer(bytes.to_vec());
        let exif = match Export::export_exif_info(input_source) {
            Ok(info) => {
                let content = info.stringify_all().unwrap_or_default();
                let mut map = HashMap::new();
                for (k, v) in self.parse_quickraw_exif(&content) {
                    map.insert(k, v);
                }
                Ok(map)
            }
            Err(e) => Err(e),
        };
        for (key, value) in exif.unwrap_or_default() {
            metadata.insert(key, value);
        }
        metadata
    }
}
