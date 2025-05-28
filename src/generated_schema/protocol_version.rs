use std::fmt::Display;
#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolVersion {
    V2024_11_05,
    V2025_03_26,
    Draft,
}
impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolVersion::V2024_11_05 => write!(f, "2024-11-05"),
            ProtocolVersion::V2025_03_26 => write!(f, "2025-03-26"),
            ProtocolVersion::Draft => write!(f, "DRAFT-2025-v2"),
        }
    }
}
#[derive(Debug)]
pub struct ParseProtocolVersionError {
    details: String,
}
impl std::fmt::Display for ParseProtocolVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Protocol version parse error: {}", self.details)
    }
}
impl std::error::Error for ParseProtocolVersionError {}
impl TryFrom<&str> for ProtocolVersion {
    type Error = ParseProtocolVersionError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "2024-11-05" => Ok(ProtocolVersion::V2024_11_05),
            "2025-03-26" => Ok(ProtocolVersion::V2025_03_26),
            "DRAFT-2025-v2" => Ok(ProtocolVersion::Draft),
            other => Err(ParseProtocolVersionError {
                details: other.to_string(),
            }),
        }
    }
}
