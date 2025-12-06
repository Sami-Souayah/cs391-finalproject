// sessions.rs
use rocket::http::{Cookie, CookieJar};
use serde::{Serialize, Deserialize};
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, OsRng, AeadCore},
    Nonce, 
};
use base64::{engine::general_purpose, Engine as _};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub username: String,
}

pub struct SessionManager {
    key: aes_gcm::Key<Aes256Gcm>,
}

impl SessionManager {
    // Generate a fresh random key at startup
    pub fn new() -> Self {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        SessionManager { key }
    }

    // Encrypt plaintext; prefix nonce to ciphertext so we can decrypt later
fn encrypt_raw(&self, plaintext: &[u8]) -> Option<Vec<u8>> {
    let cipher = Aes256Gcm::new(&self.key);

    // This now works because AeadCore is in scope
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96 bits

    let mut out = nonce.to_vec();
    match cipher.encrypt(&nonce, plaintext) {
        Ok(mut ct) => {
            out.append(&mut ct);
            Some(out)
        }
        Err(_) => None,
    }
}


fn decrypt_raw(&self, data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 12 {
        return None;
    }

    let cipher = Aes256Gcm::new(&self.key);
    let (nonce_bytes, ct) = data.split_at(12);

    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ct).ok()
}



    pub fn set_session_cookie(&self, cookies: &CookieJar<'_>, session: &SessionData) {
        let json = serde_json::to_string(session).expect("serialize session");
        if let Some(cipher_bytes) = self.encrypt_raw(json.as_bytes()) {
            let value = general_purpose::STANDARD.encode(cipher_bytes);
            cookies.add(Cookie::new("session", value));
        }
    }

    pub fn get_session_from_cookies(&self, cookies: &CookieJar<'_>) -> Option<SessionData> {
        let cookie = cookies.get("session")?;
        let bytes = general_purpose::STANDARD.decode(cookie.value()).ok()?;
        let plain = self.decrypt_raw(&bytes)?;
        serde_json::from_slice(&plain).ok()
    }

    /// High-level API for app code
    pub fn encrypt_for_session(&self, _session: &SessionData, plaintext: &[u8]) -> Option<Vec<u8>> {
        self.encrypt_raw(plaintext)
    }

    pub fn decrypt_for_session(&self, _session: &SessionData, cipher_bytes: &[u8]) -> Option<String> {
        self.decrypt_raw(cipher_bytes)
            .and_then(|b| String::from_utf8(b).ok())
    }
}
