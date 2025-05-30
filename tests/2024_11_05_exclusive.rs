//! Test cases applicable only to the specific version(s) of the schema (currently 2024_11_05)
#[path = "common/common.rs"]
pub mod common;

#[cfg(feature = "2024_11_05")]
mod test_2024_11_05_exclusive {
    use rust_mcp_schema::mcp_2024_11_05::schema_utils::*;
    use rust_mcp_schema::mcp_2024_11_05::*;

    use super::common::{get_message, re_serialize};

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
                    size: None,
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

    #[test]
    fn test_client_list_resource_templates_request() {
        // create a ClientMessage
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::ListResourceTemplatesRequest(
                ListResourceTemplatesRequest::new(None),
            )),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListResourceTemplatesRequest(_)))
        ));
    }

    #[test]
    fn test_server_list_resource_templates_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::ListResourceTemplatesResult(ListResourceTemplatesResult {
                meta: None,
                next_cursor: None,
                resource_templates: vec![],
            })),
        ));

        let message: ServerMessage = re_serialize(message);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListResourceTemplatesResult(_)))
        ));
    }

    #[test]
    fn test_server_list_resource_templates_result_sample() {
        let message = get_message("res_template_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListResourceTemplatesResult(_)))
        ));
    }

    #[test]
    fn test_client_list_resource_templates_request_sample() {
        let message = get_message("req_template_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListResourceTemplatesRequest(_)))
        ));
    }
}
