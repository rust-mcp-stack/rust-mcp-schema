use std::fmt::Display;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtocolVersion {
    V2024_11_05,
    V2025_03_26,
    V2025_06_18,
    Draft,
}
impl ProtocolVersion {
    pub fn supported_versions(include_draft: bool) -> Vec<ProtocolVersion> {
        let mut versions = vec![
            ProtocolVersion::V2024_11_05,
            ProtocolVersion::V2025_03_26,
            ProtocolVersion::V2025_06_18,
        ];
        if include_draft {
            versions.push(ProtocolVersion::Draft);
        }
        versions
    }
}
impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolVersion::V2024_11_05 => write!(f, "2024-11-05"),
            ProtocolVersion::V2025_03_26 => write!(f, "2025-03-26"),
            ProtocolVersion::V2025_06_18 => write!(f, "2025-06-18"),
            ProtocolVersion::Draft => write!(f, "DRAFT-2025-v3"),
        }
    }
}
#[derive(Debug)]
pub struct ParseProtocolVersionError {
    details: String,
}
impl std::fmt::Display for ParseProtocolVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Unsupported protocol version : {}. Supported versions: {}",
            self.details,
            ProtocolVersion::supported_versions(false)
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
impl std::error::Error for ParseProtocolVersionError {}
impl TryFrom<&str> for ProtocolVersion {
    type Error = ParseProtocolVersionError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "2024-11-05" => Ok(ProtocolVersion::V2024_11_05),
            "2025-03-26" => Ok(ProtocolVersion::V2025_03_26),
            "2025-06-18" => Ok(ProtocolVersion::V2025_06_18),
            "DRAFT-2025-v3" => Ok(ProtocolVersion::Draft),
            "DRAFT" => Ok(ProtocolVersion::Draft),
            other => Err(ParseProtocolVersionError {
                details: other.to_string(),
            }),
        }
    }
}
