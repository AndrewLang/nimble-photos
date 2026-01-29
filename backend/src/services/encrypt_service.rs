use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce, aead::Aead};
use nimble_web::config::Configuration;
use rand::RngCore;

pub struct EncryptService {
    cipher: XChaCha20Poly1305,
}

impl EncryptService {
    pub fn new(config: &Configuration) -> Result<Self> {
        let key_b64 = config
            .get("Encryption.Key")
            .ok_or_else(|| anyhow!("encryption key not configured"))?;
        let key_bytes = STANDARD.decode(key_b64)?;
        if key_bytes.len() != 32 {
            return Err(anyhow!("encryption key must be 32 bytes"));
        }
        let key = Key::from_slice(&key_bytes);
        let cipher = XChaCha20Poly1305::new(key);
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("encryption failed: {}", e))?;
        let mut out = Vec::with_capacity(24 + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);

        Ok(STANDARD.encode(&out))
    }

    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<String> {
        let data = STANDARD.decode(ciphertext_b64)?;
        if data.len() < 24 {
            return Err(anyhow!("ciphertext too short"));
        }
        let (nonce_bytes, ct) = data.split_at(24);
        let nonce = XNonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ct)
            .map_err(|e| anyhow!("decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| anyhow!("invalid utf8: {}", e))
    }
}
