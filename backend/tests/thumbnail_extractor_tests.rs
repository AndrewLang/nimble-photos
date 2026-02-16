use image::{ImageBuffer, ImageReader, Rgb};
use nimble_photos::services::ThumbnailExtractor;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct ThumbnailExtractorTestContext {
    root: PathBuf,
}

impl ThumbnailExtractorTestContext {
    const SOURCE_WIDTH: u32 = 3000;
    const SOURCE_HEIGHT: u32 = 2000;
    const CUSTOM_THUMBNAIL_SIZE: u32 = 128;
    const DEFAULT_THUMBNAIL_FILE_NAME: &'static str = "thumbnail.webp";
    const CUSTOM_THUMBNAIL_FILE_NAME: &'static str = "thumbnail_custom.webp";

    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nimble_photos_thumbnail_extractor_{}_{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&path).expect("failed to create thumbnail extractor test directory");
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

impl Drop for ThumbnailExtractorTestContext {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn thumbnail_extractor_writes_to_requested_location() {
    let context = ThumbnailExtractorTestContext::new();
    context.create_source_image();
    let extractor = ThumbnailExtractor::new();
    let output = context.output_path(ThumbnailExtractorTestContext::DEFAULT_THUMBNAIL_FILE_NAME);

    extractor
        .extract_to(context.source_image_path(), &output)
        .expect("thumbnail extraction failed");

    assert!(output.exists());
}

#[test]
fn thumbnail_extractor_respects_custom_thumbnail_size() {
    let context = ThumbnailExtractorTestContext::new();
    context.create_source_image();
    let extractor = ThumbnailExtractor::new()
        .with_max_border(ThumbnailExtractorTestContext::CUSTOM_THUMBNAIL_SIZE);
    let output = context.output_path(ThumbnailExtractorTestContext::CUSTOM_THUMBNAIL_FILE_NAME);

    extractor
        .extract_to(context.source_image_path(), &output)
        .expect("custom thumbnail extraction failed");

    let dimensions = ThumbnailExtractorTestContext::image_dimensions(&output);
    assert!(dimensions.0 <= ThumbnailExtractorTestContext::CUSTOM_THUMBNAIL_SIZE);
    assert!(dimensions.1 <= ThumbnailExtractorTestContext::CUSTOM_THUMBNAIL_SIZE);
}
