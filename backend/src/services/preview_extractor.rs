use anyhow::{Result, anyhow};
use image::{ImageFormat, ImageReader, imageops::FilterType};
use rawthumb::{ExportConfig, ThumbnailExporter};
use std::fs;
use std::path::{Path, PathBuf};

use super::image_process_constants::{PREVIEW_FORMAT_EXTENSION, RAW_EXTENSIONS};

const PREVIEW_MAX_BORDER: u32 = 2048;

#[derive(Clone, Debug)]
pub struct PreviewExtractor {
    max_border: u32,
    output_path: Option<PathBuf>,
}

impl PreviewExtractor {
    pub fn new() -> Self {
        Self {
            max_border: PREVIEW_MAX_BORDER,
            output_path: None,
        }
    }

    pub fn with_max_border(mut self, max_border: u32) -> Self {
        self.max_border = max_border;
        self
    }

    pub fn with_output_path<P: AsRef<Path>>(mut self, output_path: P) -> Self {
        self.output_path = Some(output_path.as_ref().to_path_buf());
        self
    }

    pub fn extract<P: AsRef<Path>>(&self, input_path: P) -> Result<PathBuf> {
        let destination = self
            .output_path
            .as_ref()
            .ok_or_else(|| anyhow!("preview output path is not configured"))?
            .to_path_buf();
        self.extract_to(input_path, destination)
    }

    pub fn extract_to<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> Result<PathBuf> {
        let destination = output_path.as_ref().to_path_buf();
        self.generate_to_file(input_path.as_ref(), &destination)?;
        Ok(destination)
    }

    pub fn preview_size(&self) -> u32 {
        self.max_border
    }

    pub fn output_format_extension() -> &'static str {
        PREVIEW_FORMAT_EXTENSION
    }

    fn generate_to_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        Self::ensure_parent_directory(output_path)?;

        if Self::is_raw_file(input_path) {
            return self.generate_raw_image(input_path, output_path);
        }

        self.generate_standard_image(input_path, output_path)
    }

    fn ensure_parent_directory(output_path: &Path) -> Result<()> {
        let parent_directory = output_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        fs::create_dir_all(parent_directory)?;
        Ok(())
    }

    fn is_raw_file(input_path: &Path) -> bool {
        input_path
            .extension()
            .and_then(|value| value.to_str())
            .map(Self::is_raw_extension)
            .unwrap_or(false)
    }

    fn is_raw_extension(extension: &str) -> bool {
        RAW_EXTENSIONS
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(extension))
    }

    fn generate_raw_image(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let exporter_config = ExportConfig::default()
            .with_auto_rotate(true)
            .with_max_border(Some(self.max_border));
        let exporter = ThumbnailExporter::new_with_config(exporter_config);
        let thumbnail = exporter.export(input_path.to_string_lossy().as_ref())?;
        fs::write(output_path, thumbnail.jpeg.as_ref())?;
        Ok(())
    }

    fn generate_standard_image(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let image = ImageReader::open(input_path)?
            .with_guessed_format()?
            .decode()?;
        let resized = image.resize(self.max_border, self.max_border, FilterType::Lanczos3);
        resized.save_with_format(output_path, ImageFormat::Jpeg)?;
        Ok(())
    }
}
