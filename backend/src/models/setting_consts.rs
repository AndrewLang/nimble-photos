pub struct SettingConsts;

impl SettingConsts {
    pub const THUMBNAIL_FOLDER: &'static str = ".thumbnails";
    pub const THUMBNAIL_CONTENT_TYPE: &'static str = "image/webp";
    pub const THUMBNAIL_FORMAT: &'static str = "webp";

    pub const PREVIEW_FOLDER: &'static str = ".previews";
    pub const PREVIEW_FORMAT: &'static str = "jpg";
    pub const PREVIEW_CONTENT_TYPE: &'static str = "image/jpeg";

    pub const DEFAULT_HTTP_IMAGE_CACHE_HEADER: &'static str = "public, max-age=31536000, immutable";
}
