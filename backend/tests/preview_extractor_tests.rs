use image::{ImageBuffer, ImageReader, Rgb};
use nimble_photos::services::PreviewExtractor;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct PreviewExtractorTestContext {
    root: PathBuf,
}

impl PreviewExtractorTestContext {
    const SOURCE_WIDTH: u32 = 3000;
    const SOURCE_HEIGHT: u32 = 2000;
    const DEFAULT_PREVIEW_FILE_NAME: &'static str = "preview.jpg";
    const CUSTOM_PREVIEW_FILE_NAME: &'static str = "preview_custom.jpg";
    const CUSTOM_PREVIEW_SIZE: u32 = 1024;

    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nimble_photos_preview_extractor_{}_{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&path).expect("failed to create preview extractor test directory");
        Self { root: path }
    }

    fn source_image_path(&self) -> PathBuf {
        self.root.join("source.png")
    }

    fn output_path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn create_source_image(&self) {
        let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_fn(
            Self::SOURCE_WIDTH,
            Self::SOURCE_HEIGHT,
            |x, y| {
                let red = (x % 255) as u8;
                let green = (y % 255) as u8;
                let blue = ((x + y) % 255) as u8;
                Rgb([red, green, blue])
            },
        );
        image
            .save(self.source_image_path())
            .expect("failed to save source image");
    }

    fn image_dimensions(path: &Path) -> (u32, u32) {
        ImageReader::open(path)
            .expect("failed to open image")
            .decode()
            .expect("failed to decode image")
            .into_rgb8()
            .dimensions()
    }
}

impl Drop for PreviewExtractorTestContext {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn preview_extractor_extract_to_writes_to_requested_location() {
    let context = PreviewExtractorTestContext::new();
    context.create_source_image();
    let extractor = PreviewExtractor::new();
    let output = context.output_path(PreviewExtractorTestContext::DEFAULT_PREVIEW_FILE_NAME);

    extractor
        .extract_to(context.source_image_path(), &output)
        .expect("preview extraction failed");

    assert!(output.exists());
}

#[test]
fn preview_extractor_respects_custom_preview_size() {
    let context = PreviewExtractorTestContext::new();
    context.create_source_image();
    let extractor =
        PreviewExtractor::new().with_max_border(PreviewExtractorTestContext::CUSTOM_PREVIEW_SIZE);
    let output = context.output_path(PreviewExtractorTestContext::CUSTOM_PREVIEW_FILE_NAME);

    extractor
        .extract_to(context.source_image_path(), &output)
        .expect("custom preview extraction failed");

    let dimensions = PreviewExtractorTestContext::image_dimensions(&output);
    assert!(dimensions.0 <= PreviewExtractorTestContext::CUSTOM_PREVIEW_SIZE);
    assert!(dimensions.1 <= PreviewExtractorTestContext::CUSTOM_PREVIEW_SIZE);
}

#[test]
fn preview_extractor_extract_uses_configured_output_path() {
    let context = PreviewExtractorTestContext::new();
    context.create_source_image();
    let output = context.output_path(PreviewExtractorTestContext::DEFAULT_PREVIEW_FILE_NAME);
    let extractor = PreviewExtractor::new().with_output_path(&output);

    let generated_path = extractor
        .extract(context.source_image_path())
        .expect("preview extraction with configured output failed");

    assert_eq!(generated_path, output);
    assert!(generated_path.exists());
}
