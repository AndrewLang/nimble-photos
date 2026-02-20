use chrono::{DateTime, NaiveDate, Utc};
use nimble_photos::models::property_map::PropertyMap;
use nimble_photos::models::template::PropertyMapTemplateContext;
use nimble_photos::models::template::{TemplateContext, TemplateEngine};
use std::any::Any;

struct TestTemplateContext {
    file_name: String,
    camera: String,
    rating: i32,
    effective_date: DateTime<Utc>,
    hash: Option<String>,
}

impl TestTemplateContext {
    fn new(file_name: &str, effective_date: DateTime<Utc>, hash: Option<&str>) -> Self {
        Self {
            file_name: file_name.to_string(),
            camera: "Canon".to_string(),
            rating: 5,
            effective_date,
            hash: hash.map(|v| v.to_string()),
        }
    }
}

impl TemplateContext for TestTemplateContext {
    fn get_property<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T> {
        match key {
            "file_name" => (&self.file_name as &dyn Any).downcast_ref::<T>(),
            "camera" => (&self.camera as &dyn Any).downcast_ref::<T>(),
            "rating" => (&self.rating as &dyn Any).downcast_ref::<T>(),
            "effective_date" => (&self.effective_date as &dyn Any).downcast_ref::<T>(),
            "hash" => self
                .hash
                .as_ref()
                .and_then(|value| (value as &dyn Any).downcast_ref::<T>()),
            _ => None,
        }
    }
}

fn test_date() -> DateTime<Utc> {
    let naive = NaiveDate::from_ymd_opt(2025, 1, 4)
        .expect("valid date")
        .and_hms_opt(10, 20, 30)
        .expect("valid time");
    DateTime::from_naive_utc_and_offset(naive, Utc)
}

#[test]
fn template_patterns_render_expected_paths() {
    let ctx = TestTemplateContext::new(
        "holiday.snapshot.jpg",
        test_date(),
        Some("abcdef1234567890"),
    );

    let cases = [
        (
            "{year}/{fileStem}-{hash:0:6}.{extension}",
            "2025/holiday.snapshot-abcdef.jpg",
        ),
        ("{year}/{date:%Y-%m-%d}", "2025/2025-01-04"),
        (
            "{year}/{date:%Y-%m-%d}/{fileName}",
            "2025/2025-01-04/holiday.snapshot.jpg",
        ),
        ("{year}/{fileName}", "2025/holiday.snapshot.jpg"),
        ("{date:%Y-%m}/{fileName}", "2025-01/holiday.snapshot.jpg"),
        ("{hash:0:2}/{hash:2:2}/{hash}", "ab/cd/abcdef1234567890"),
        (
            "{hash:0:2}/{hash:2:2}/{fileStem}-{hash:0:6}.{extension}",
            "ab/cd/holiday.snapshot-abcdef.jpg",
        ),
        (
            "{date:%Y-%m-%d}/{fileStem}-{hash:0:8}.{extension}",
            "2025-01-04/holiday.snapshot-abcdef12.jpg",
        ),
        (
            "{year}/{month}/{day}/{hash:0:2}/{fileName}",
            "2025/01/04/ab/holiday.snapshot.jpg",
        ),
    ];

    for (template, expected) in cases {
        let compiled = TemplateEngine::compile(template).expect("template should compile");
        let rendered = compiled.render(&ctx).expect("template should render");
        assert_eq!(rendered, expected, "template: {template}");
    }
}

#[test]
fn requires_hash_detects_hash_tokens() {
    let with_hash = TemplateEngine::compile("{year}/{hash:0:2}/{fileName}")
        .expect("hash template should compile");
    let without_hash =
        TemplateEngine::compile("{year}/{date:%Y-%m-%d}/{fileName}").expect("compile should pass");

    assert!(with_hash.requires_hash());
    assert!(!without_hash.requires_hash());
}

#[test]
fn property_map_template_context_renders_template() {
    let mut properties = PropertyMap::new();
    properties
        .insert::<String>("holiday.snapshot.jpg".to_string())
        .alias("file_name");
    properties
        .insert::<DateTime<Utc>>(test_date())
        .alias("effective_date");
    properties
        .insert::<String>("abcdef1234567890".to_string())
        .alias("hash");

    let ctx = PropertyMapTemplateContext::new(properties);
    let template = TemplateEngine::compile("{year}/{fileStem}-{hash:0:6}.{extension}")
        .expect("template should compile");

    let rendered = template.render(&ctx).expect("template should render");
    assert_eq!(rendered, "2025/holiday.snapshot-abcdef.jpg");
}

#[test]
fn property_map_template_context_requires_effective_date() {
    let mut properties = PropertyMap::new();
    properties
        .insert::<String>("holiday.snapshot.jpg".to_string())
        .alias("file_name");
    properties
        .insert::<String>("abcdef1234567890".to_string())
        .alias("hash");

    let ctx = PropertyMapTemplateContext::new(properties);
    let template = TemplateEngine::compile("{year}/{fileName}").expect("template should compile");

    let result = template.render(&ctx);
    assert!(result.is_err());
}
