use regex::Regex;
use crate::sessions::SessionData;

pub fn reject_sensitive_text(input: &str) -> bool {
    let ssn = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap();
    let cc = Regex::new(r"\b\d{16}\b").unwrap();
    let phone = Regex::new(r"\b\d{3}[- ]?\d{3}[- ]?\d{4}\b").unwrap();

    !(ssn.is_match(input) || cc.is_match(input) || phone.is_match(input))
}

pub fn user_may_access(session: &SessionData, target_username: &str) -> bool {
    session.username == target_username
}