use anyhow::Result;
use image::ImageReader;
use image::imageops::FilterType;
use rawthumb::{ExportConfig, ThumbnailExporter};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ImageProcessService;

impl ImageProcessService {
    const THUMBNAIL_MAX_BORDER: u32 = 400;
    const PREVIEW_MAX_BORDER: u32 = 2048;
    const OUTPUT_FORMAT_EXTENSION: &'static str = "jpg";
    const RAW_EXTENSIONS: [&'static str; 10] = [
        "cr2", "cr3", "nef", "arw", "dng", "orf", "raf", "rw2", "pef", "srw",
    ];

    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_thumbnail_from_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> Result<()> {
        self.generate_from_file(input_path.as_ref(), output_path.as_ref(), Self::THUMBNAIL_MAX_BORDER)
    }

    pub fn generate_preview_from_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> Result<()> {
        self.generate_from_file(input_path.as_ref(), output_path.as_ref(), Self::PREVIEW_MAX_BORDER)
    }

    fn generate_from_file(&self, input_path: &Path, output_path: &Path, max_border: u32) -> Result<()> {
        Self::ensure_parent_directory(output_path)?;

        if self.is_raw_file(input_path) {
            return self.generate_raw_image(input_path, output_path, max_border);
        }

        self.generate_standard_image(input_path, output_path, max_border)
    }

    fn generate_raw_image(&self, input_path: &Path, output_path: &Path, max_border: u32) -> Result<()> {
        let exporter_config = ExportConfig::default()
            .with_auto_rotate(true)
            .with_max_border(Some(max_border));
        let exporter = ThumbnailExporter::new_with_config(exporter_config);
        let thumbnail = exporter.export(input_path.to_string_lossy().as_ref())?;
        fs::write(output_path, thumbnail.jpeg.as_ref())?;
        Ok(())
    }

    fn generate_standard_image(
        &self,
        input_path: &Path,
        output_path: &Path,
        max_border: u32,
    ) -> Result<()> {
        let image = ImageReader::open(input_path)?
            .with_guessed_format()?
            .decode()?;
        let resized = image.resize(max_border, max_border, FilterType::Lanczos3);
        resized.save_with_format(output_path, image::ImageFormat::Jpeg)?;
        Ok(())
    }

    fn is_raw_file(&self, input_path: &Path) -> bool {
        let extension = input_path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());

        match extension {
            Some(value) => Self::RAW_EXTENSIONS.contains(&value.as_str()),
            None => false,
        }
    }

    fn ensure_parent_directory(output_path: &Path) -> Result<()> {
        let parent_directory = output_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        fs::create_dir_all(parent_directory)?;
        Ok(())
    }

    pub fn output_format_extension(&self) -> &'static str {
        Self::OUTPUT_FORMAT_EXTENSION
    }
}
