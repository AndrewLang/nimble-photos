use image::{ImageBuffer, Rgb};
use nimble_photos::services::{PreviewExtractor, ThumbnailExtractor};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

struct ImageExtractorTestContext {
    root: PathBuf,
}

impl ImageExtractorTestContext {
    const SOURCE_WIDTH: u32 = 3000;
    const SOURCE_HEIGHT: u32 = 2000;
    const THUMBNAIL_MAX_BORDER: u32 = 400;
    const PREVIEW_MAX_BORDER: u32 = 2048;
    const PARALLEL_TASK_COUNT: usize = 6;

    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nimble_photos_image_process_service_{}_{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&path).expect("failed to create test root directory");
        Self { root: path }
    }

    fn source_image_path(&self) -> PathBuf {
        self.root.join("source.png")
    }

    fn raw_image_path(&self) -> PathBuf {
        self.root.join("source.cr3")
    }

    fn thumbnail_output_path(&self) -> PathBuf {
        self.root.join("thumbnail.webp")
    }

    fn preview_output_path(&self) -> PathBuf {
        self.root.join("preview.jpg")
    }

    fn parallel_thumbnail_output_path(&self, index: usize) -> PathBuf {
        self.root.join(format!("thumbnail_{index}.webp"))
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

    fn create_invalid_raw_image(&self) {
        fs::write(self.raw_image_path(), b"invalid-raw-content")
            .expect("failed to write raw test file");
    }

    fn image_dimensions(path: &Path) -> (u32, u32) {
        image::ImageReader::open(path)
            .expect("failed to open image")
            .decode()
            .expect("failed to decode image")
            .into_rgb8()
            .dimensions()
    }
}

impl Drop for ImageExtractorTestContext {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn thumbnail_extractor_creates_webp_with_thumbnail_size() {
    let context = ImageExtractorTestContext::new();
    context.create_source_image();
    let extractor = ThumbnailExtractor::new();
    let output = context.thumbnail_output_path();

    extractor
        .extract_to(context.source_image_path(), &output)
        .expect("thumbnail generation failed");

    assert!(output.exists());
    let dimensions = ImageExtractorTestContext::image_dimensions(&output);
    assert!(dimensions.0 <= ImageExtractorTestContext::THUMBNAIL_MAX_BORDER);
    assert!(dimensions.1 <= ImageExtractorTestContext::THUMBNAIL_MAX_BORDER);
}

#[test]
fn preview_extractor_creates_jpeg_with_preview_size() {
    let context = ImageExtractorTestContext::new();
    context.create_source_image();
    let extractor = PreviewExtractor::new();
    let output = context.preview_output_path();

    extractor
        .extract_to(context.source_image_path(), &output)
        .expect("preview generation failed");

    assert!(output.exists());
    let dimensions = ImageExtractorTestContext::image_dimensions(&output);
    assert!(dimensions.0 <= ImageExtractorTestContext::PREVIEW_MAX_BORDER);
    assert!(dimensions.1 <= ImageExtractorTestContext::PREVIEW_MAX_BORDER);
}

#[test]
fn thumbnail_extractor_returns_error_for_invalid_raw_content() {
    let context = ImageExtractorTestContext::new();
    context.create_invalid_raw_image();
    let extractor = ThumbnailExtractor::new();
    let output = context.thumbnail_output_path();

    let result = extractor.extract_to(context.raw_image_path(), &output);

    assert!(result.is_err());
}

#[test]
fn thumbnail_extractor_supports_parallel_generation() {
    let context = Arc::new(ImageExtractorTestContext::new());
    context.create_source_image();
    let extractor = Arc::new(ThumbnailExtractor::new());

    let handles: Vec<_> = (0..ImageExtractorTestContext::PARALLEL_TASK_COUNT)
        .map(|index| {
            let extractor_clone = Arc::clone(&extractor);
            let context_clone = Arc::clone(&context);
            thread::spawn(move || {
                let output = context_clone.parallel_thumbnail_output_path(index);
                extractor_clone
                    .extract_to(context_clone.source_image_path(), &output)
                    .expect("parallel thumbnail generation failed");
                output
            })
        })
        .collect();

    for handle in handles {
        let output = handle.join().expect("parallel task panicked");
        assert!(output.exists());
    }
}
