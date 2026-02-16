use nimble_photos::services::ExifService;

#[test]
fn extract_from_invalid_bytes_returns_empty_model() {
    let service = ExifService::new();
    let result = service.extract_from_bytes(b"not-an-image");

    assert!(result.make.is_none());
    assert!(result.model.is_none());
    assert!(result.datetime.is_none());
    assert!(result.gps_latitude.is_none());
}

#[test]
fn extract_from_missing_path_returns_empty_model() {
    let service = ExifService::new();
    let result = service.extract_from_path("./does-not-exist-image-file.jpg");

    assert!(result.make.is_none());
    assert!(result.model.is_none());
    assert!(result.datetime.is_none());
    assert!(result.gps_longitude.is_none());
}
