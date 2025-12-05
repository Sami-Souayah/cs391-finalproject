use cocoon::Cocoon;
use rocket::http::CookieJar;
use serde::{Serialize, Deserialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct SessionData {
    pub username: String,
}

pub struct SessionManager {
    pub cocoon: Cocoon,
}

impl SessionManager {
    pub fn new() -> Self {
        let key_str = env::var("COCOON_KEY")
            .expect("COCOON_KEY must be set in .env")
            .replace("base64:", "");

        let key = base64::decode(key_str).expect("Invalid key");

        SessionManager {
            cocoon: Cocoon::new(&key),
        }
    }

    pub fn create_session(&self, cookies: &CookieJar<'_>, username: &str) {
        let data = SessionData {
            username: username.to_string(),
        };

        let serialized = serde_json::to_vec(&data).unwrap();
        let encrypted = self.cocoon.wrap(&serialized).unwrap();

        cookies.add(rocket::http::Cookie::new(
            "session",
            base64::encode(&encrypted),
        ));
    }

    pub fn get_session(&self, cookies: &CookieJar<'_>) -> Option<SessionData> {
        let c = cookies.get("session")?;
        let decoded = base64::decode(c.value()).ok()?;
        let decrypted = self.cocoon.unwrap(&decoded).ok()?;

        serde_json::from_slice(&decrypted).ok()
    }
}