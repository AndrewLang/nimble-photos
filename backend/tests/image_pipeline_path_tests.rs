use nimble_photos::services::image_pipeline::ImageProcessContext;

#[test]
fn thumbnail_output_path_uses_hash_segments() {
    let thumbnail_root = std::env::temp_dir().join("thumb-root-test");
    let output = ImageProcessContext::thumbnail_output_path_from_hash(
        &thumbnail_root,
        Some("abcd1234"),
        "webp",
    )
    .expect("hash should enable thumbnail path");
    let expected = thumbnail_root.join("ab").join("cd").join("abcd1234.webp");
    assert_eq!(output, expected);
}

#[test]
fn preview_output_path_requires_hash() {
    let preview_root = std::env::temp_dir().join("preview-root-test");
    assert!(
        ImageProcessContext::preview_output_path_from_hash(&preview_root, None, "webp").is_err()
    );

    let output =
        ImageProcessContext::preview_output_path_from_hash(&preview_root, Some("beefcafe"), "webp")
            .expect("hash should enable preview path");
    let expected = preview_root.join("be").join("ef").join("beefcafe.webp");
    assert_eq!(output, expected);
}

#[test]
fn thumbnail_output_path_zero_pads_short_hashes() {
    let thumbnail_root = std::env::temp_dir().join("thumb-root-short-hash");
    let output =
        ImageProcessContext::thumbnail_output_path_from_hash(&thumbnail_root, Some("9f"), "jpg")
            .expect("short hash should still produce a thumbnail path");

    let expected = thumbnail_root.join("9f").join("00").join("9f.jpg");
    assert_eq!(output, expected);
}
