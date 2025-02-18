#[path = "common/common.rs"]
pub mod common;

mod miscellaneous_tests {
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
