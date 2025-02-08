//! Test cases applicable only to the draft version of the schema
#[path = "common/common.rs"]
pub mod common;

#[cfg(feature = "draft")]
mod test_draft_exclusive {
    use super::common::re_serialize;
    use rust_mcp_schema::schema_utils::*;
    use rust_mcp_schema::*;

    #[test]
    fn test_server_list_resources_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::ListResourcesResult(ListResourcesResult {
                meta: None,
                next_cursor: None,
                resources: vec![Resource {
                    annotations: None,
                    description: None,
                    mime_type: None,
                    name: "Resource 1".to_string(),
                    uri: "test://static/resource/1".to_string(),
                }],
            })),
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListResourcesResult(_)))
        ));
    }
}
