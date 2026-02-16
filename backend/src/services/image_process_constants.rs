pub(crate) const THUMBNAIL_FORMAT_EXTENSION: &str = "webp";
pub(crate) const PREVIEW_FORMAT_EXTENSION: &str = "jpg";
pub(crate) const RAW_EXTENSIONS: [&str; 10] = [
    "cr2", "cr3", "nef", "arw", "dng", "orf", "raf", "rw2", "pef", "srw",
];

pub struct ImageProcessKeys {}

impl ImageProcessKeys {
    pub const RAW_EXTENSIONS: [&'static str; 10] = [
        "cr2", "cr3", "nef", "arw", "dng", "orf", "raf", "rw2", "pef", "srw",
    ];

    pub const THUMBNAIL_FORMAT_EXTENSION: &'static str = "webp";
    pub const THUMBNAIL_PATH: &'static str = "thumbnail_path";
    pub const THUMBNAIL_WIDTH: &'static str = "thumbnail_width";
    pub const THUMBNAIL_HEIGHT: &'static str = "thumbnail_height";
    pub const PREVIEW_FORMAT_EXTENSION: &'static str = "jpg";
    pub const PREVIEW_PATH: &'static str = "preview_path";

    pub const EXIF_METADATA: &'static str = "exif_metadata";
    pub const EXIF_DATE_TAKEN: &'static str = "exif_date_taken";
    pub const CATEGORIZE_DATE_FORMAT: &'static str = "categorize_date_format";
    pub const HASH: &'static str = "hash";
    pub const WORKING_DIRECTORY: &'static str = "working_directory";
    pub const FINAL_PATH: &'static str = "final_path";
}
