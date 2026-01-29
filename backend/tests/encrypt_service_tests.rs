use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use nimble_web::config::Configuration;

use nimble_photos::services::EncryptService;

#[test]
fn encrypt_decrypt_roundtrip() {
    let key = vec![0u8; 32];
    let mut values = HashMap::new();
    values.insert("Encryption.Key".to_string(), STANDARD.encode(&key));
    let config = Configuration::from_values(values);

    let svc = EncryptService::new(&config).unwrap();
    let plain = "hello world";
    let ct = svc.encrypt(plain).unwrap();
    let pt = svc.decrypt(&ct).unwrap();
    assert_eq!(pt, plain);
}
