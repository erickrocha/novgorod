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
