use regex::Regex;
use base64::{engine::general_purpose, Engine as _};

use crate::sessions::{SessionData, SessionManager};

/// Policy 1: user can only access their own data.
pub fn user_may_access(session: &SessionData, target_username: &str) -> bool {
    session.username == target_username
}

/// Policy 2: reject payloads that look like SSNs / CC numbers / phone numbers.
pub fn reject_sensitive_text(input: &str) -> bool {
    let ssn = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap();
    let cc = Regex::new(r"\b\d{13,19}\b").unwrap();
    let phone = Regex::new(r"\b\d{3}[- ]?\d{3}[- ]?\d{4}\b").unwrap();

    !(ssn.is_match(input) || cc.is_match(input) || phone.is_match(input))
}

/// Policy 3: only authenticated users can submit / access.
pub fn require_authenticated(session: Option<&SessionData>) -> bool {
    session.is_some()
}

pub fn owner_decrypt_if_allowed(
    sessions: &SessionManager,
    session: &SessionData,
    encrypted_b64: &str,
) -> Result<String, &'static str> {
    if !user_may_access(session, &session.username) {
        return Err("not allowed to read this user's data");
    }

    let bytes = general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|_| "invalid base64")?;

    sessions
        .decrypt_for_session(session, &bytes)
        .ok_or("decryption failed")
}
