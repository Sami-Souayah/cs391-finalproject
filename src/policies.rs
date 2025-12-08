use regex::Regex;
use crate::sessions::{SessionData, SessionManager};
use base64::Engine as _;



pub fn reject_sensitive_text(input: &str) -> bool {
    let ssn = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap();
    let cc  = Regex::new(r"\b\d{13,19}\b").unwrap();
    let phone = Regex::new(r"\b\d{3}[- ]?\d{3}[- ]?\d{4}\b").unwrap();

    !(ssn.is_match(input) || cc.is_match(input) || phone.is_match(input))
}

pub fn owner_decrypt_if_allowed(
    sessions: &SessionManager,
    session: &SessionData,
    encrypted_b64: &str,
) -> Result<String, &'static str> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|_| "Invalid base64")?;

    sessions
        .decrypt_for_session(session, &bytes)
        .ok_or("Access denied or decryption failed")
}
