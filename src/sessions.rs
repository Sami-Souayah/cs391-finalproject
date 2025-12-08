use rocket::http::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};

use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit, OsRng, AeadCore};
use base64::{engine::general_purpose, Engine as _};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub username: String,
}

pub struct SessionManager {
    key: aes_gcm::Key<Aes256Gcm>,
}

impl SessionManager {
    pub fn new() -> Self {
        // For a real app, you'd load this from env; for demo,
        // we generate a fresh key each run.
        let key = Aes256Gcm::generate_key(&mut OsRng);
        SessionManager { key }
    }

    fn cipher(&self) -> Aes256Gcm {
        Aes256Gcm::new(&self.key)
    }

    fn encrypt_bytes(&self, plaintext: &[u8]) -> Option<Vec<u8>> {
        let cipher = self.cipher();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); 
        let mut out = nonce.to_vec(); 
        let mut ct = cipher.encrypt(&nonce, plaintext).ok()?;
        out.append(&mut ct);
        Some(out)
    }

    fn decrypt_bytes(&self, data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < 12 {
            return None;
        }
        let (nonce_bytes, ct) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = self.cipher();
        cipher.decrypt(nonce, ct).ok()
    }

    /// Create an authenticated session cookie (Policy 3).
    pub fn create_session(&self, cookies: &CookieJar<'_>, username: &str) {
        let session = SessionData {
            username: username.to_owned(),
        };

        let json = serde_json::to_vec(&session).expect("serialize session");
        if let Some(encrypted) = self.encrypt_bytes(&json) {
            let encoded = general_purpose::STANDARD.encode(encrypted);

            let cookie = Cookie::build(("session", encoded))
                .path("/")
                .http_only(true)
                .finish();

            cookies.add(cookie);
        }
    }

    /// Read / decrypt the current session, if any.
    pub fn get_session(&self, cookies: &CookieJar<'_>) -> Option<SessionData> {
        let cookie = cookies.get("session")?;
        let decoded = general_purpose::STANDARD.decode(cookie.value()).ok()?;
        let decrypted = self.decrypt_bytes(&decoded)?;
        serde_json::from_slice(&decrypted).ok()
    }

    /// Encrypt arbitrary data "for this session".
    pub fn encrypt_for_session(
        &self,
        _session: &SessionData,
        plaintext: &[u8],
    ) -> Option<Vec<u8>> {
        self.encrypt_bytes(plaintext)
    }

    /// Decrypt arbitrary data "for this session".
    pub fn decrypt_for_session(
        &self,
        _session: &SessionData,
        ciphertext: &[u8],
    ) -> Option<String> {
        let bytes = self.decrypt_bytes(ciphertext)?;
        String::from_utf8(bytes).ok()
    }
}
