use chrono::{TimeZone, Utc};
use nimble_photos::models::property_map::PropertyMap;
use nimble_photos::services::image_categorizer::{
    CategorizeRequest, ImageCategorizer, TemplateCategorizer,
};
use std::fs;
use std::path::{Path, PathBuf};

const WORKING_DIRECTORY: &str = "working_directory";
const HASH: &str = "hash";
const EXIF_DATE_TAKEN: &str = "exif_date_taken";

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nimble_photos_template_categorizer_tests_{}_{}_{}",
        std::process::id(),
        name,
        nanos
    ))
}

fn write_test_file(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    fs::write(path, contents).expect("failed to write test file");
}

#[test]
fn template_categorizer_supports_hash_shortcut() {
    let root = unique_temp_dir("hash-shortcut");
    let source = root.join("incoming.jpg");
    let working_dir = root.join("storage");
    write_test_file(&source, b"photo");

    let mut properties = PropertyMap::new();
    properties
        .insert::<PathBuf>(working_dir.clone())
        .alias(WORKING_DIRECTORY);
    properties
        .insert::<String>("abcdef1234567890".to_string())
        .alias(HASH);
    properties
        .insert::<Option<chrono::DateTime<Utc>>>(None)
        .alias(EXIF_DATE_TAKEN);

    let request = CategorizeRequest::new(&source, &properties);
    let categorizer = TemplateCategorizer::new("hash");
    let result = categorizer.categorize(&request).expect("categorize should pass");

    let expected = working_dir.join("ab").join("cd").join("incoming.jpg");
    assert_eq!(result.final_path, expected);
    assert!(result.final_path.exists());
    assert!(!source.exists());
}

#[test]
fn template_categorizer_supports_date_shortcut_and_custom_template() {
    let root = unique_temp_dir("date-custom");
    let source_date = root.join("incoming-date.jpg");
    let source_custom = root.join("incoming-custom.jpg");
    let working_dir = root.join("storage");
    write_test_file(&source_date, b"date");
    write_test_file(&source_custom, b"custom");

    let date_taken = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();

    let mut date_properties = PropertyMap::new();
    date_properties
        .insert::<PathBuf>(working_dir.clone())
        .alias(WORKING_DIRECTORY);
    date_properties
        .insert::<Option<chrono::DateTime<Utc>>>(Some(date_taken))
        .alias(EXIF_DATE_TAKEN);

    let date_result = TemplateCategorizer::new("date")
        .categorize(&CategorizeRequest::new(&source_date, &date_properties))
        .expect("date shortcut should pass");
    assert_eq!(
        date_result.final_path,
        working_dir.join("2024-01-02").join("incoming-date.jpg")
    );

    let mut custom_properties = PropertyMap::new();
    custom_properties
        .insert::<PathBuf>(working_dir.clone())
        .alias(WORKING_DIRECTORY);
    custom_properties
        .insert::<Option<chrono::DateTime<Utc>>>(Some(date_taken))
        .alias(EXIF_DATE_TAKEN);
    custom_properties
        .insert::<String>("abcdef1234567890".to_string())
        .alias(HASH);

    let custom_result =
        TemplateCategorizer::new("{year}/{hash:0:2}/{fileStem}-{hash:0:6}.{extension}")
            .categorize(&CategorizeRequest::new(&source_custom, &custom_properties))
            .expect("custom template should pass");
    assert_eq!(
        custom_result.final_path,
        working_dir
            .join("2024")
            .join("ab")
            .join("incoming-custom-abcdef.jpg")
    );
}

#[test]
fn template_categorizer_requires_working_directory() {
    let root = unique_temp_dir("missing-workdir");
    let source = root.join("incoming.jpg");
    write_test_file(&source, b"photo");

    let mut properties = PropertyMap::new();
    properties
        .insert::<Option<chrono::DateTime<Utc>>>(Some(Utc::now()))
        .alias(EXIF_DATE_TAKEN);

    let result =
        TemplateCategorizer::new("{year}/{fileName}").categorize(&CategorizeRequest::new(
            &source,
            &properties,
        ));

    assert!(result.is_err());
}
