use nimble_photos::services::image_pipeline::{
    preview_output_path_from_hash, thumbnail_output_path_from_hash,
};

#[test]
fn thumbnail_output_path_uses_hash_segments() {
    let thumbnail_root = std::env::temp_dir().join("thumb-root-test");
    let output = thumbnail_output_path_from_hash(&thumbnail_root, Some("abcd1234"), "webp")
        .expect("hash should enable thumbnail path");
    let expected = thumbnail_root
        .join("ab")
        .join("cd")
        .join("abcd1234.webp");
    assert_eq!(output, expected);
}

#[test]
fn preview_output_path_requires_hash() {
    let preview_root = std::env::temp_dir().join("preview-root-test");
    assert!(preview_output_path_from_hash(&preview_root, None, "webp").is_err());

    let output = preview_output_path_from_hash(&preview_root, Some("beefcafe"), "webp")
        .expect("hash should enable preview path");
    let expected = preview_root.join("be").join("ef").join("beefcafe.webp");
    assert_eq!(output, expected);
}
