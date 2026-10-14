use uuid::Uuid;

pub fn uuid_to_string(uuid: Uuid) -> String {
    uuid.to_string()
}

pub fn string_to_uuid(uuid_str: &str) -> Uuid {
    Uuid::parse_str(uuid_str).unwrap_or_else(|_| Uuid::nil())
}

pub fn parse_uuid(uuid_str: &str) -> Result<Uuid, uuid::Error> {
    Uuid::parse_str(uuid_str)
}

pub fn parse_uuids(uuid_strings: &[String]) -> Result<Vec<Uuid>, uuid::Error> {
    uuid_strings.iter().map(|value| parse_uuid(value)).collect()
}

/// Lowercase hexadecimal SHA-256 of `data` (sha2 0.11 digests no longer implement `LowerHex`).
pub fn sha256_hex(data: impl AsRef<[u8]>) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;
    Sha256::digest(data).iter().fold(String::with_capacity(64), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}
