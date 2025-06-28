#[path = "common/common.rs"]
pub mod common;

mod miscellaneous_tests {
    #[cfg(feature = "2024_11_05")]
    use rust_mcp_schema::mcp_2024_11_05::schema_utils::*;
    #[cfg(feature = "2025_03_26")]
    use rust_mcp_schema::mcp_2025_03_26::schema_utils::*;
    #[cfg(feature = "draft")]
    use rust_mcp_schema::mcp_draft::schema_utils::*;
    #[cfg(feature = "latest")]
    use rust_mcp_schema::schema_utils::*;

    #[test]
    fn test_display_request() {
        assert_eq!(MessageTypes::Request.to_string(), "Request");
    }

    #[test]
    fn test_display_response() {
        assert_eq!(MessageTypes::Response.to_string(), "Response");
    }

    #[test]
    fn test_display_notification() {
        assert_eq!(MessageTypes::Notification.to_string(), "Notification");
    }

    #[test]
    fn test_display_error() {
        assert_eq!(MessageTypes::Error.to_string(), "Error");
    }
}
