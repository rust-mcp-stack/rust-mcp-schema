#[path = "common/common.rs"]
pub mod common;

mod test_deserialize {
    #[cfg(feature = "2024_11_05")]
    use rust_mcp_schema::mcp_2024_11_05::schema_utils::*;
    #[cfg(feature = "2025_03_26")]
    use rust_mcp_schema::mcp_2025_03_26::schema_utils::*;
    #[cfg(feature = "draft")]
    use rust_mcp_schema::mcp_draft::schema_utils::*;
    #[cfg(any(feature = "latest", feature = "2025_06_18"))]
    use rust_mcp_schema::schema_utils::*;

    #[cfg(feature = "2024_11_05")]
    use rust_mcp_schema::mcp_2024_11_05::*;
    #[cfg(feature = "2025_03_26")]
    use rust_mcp_schema::mcp_2025_03_26::*;
    #[cfg(feature = "draft")]
    use rust_mcp_schema::mcp_draft::*;
    #[cfg(any(feature = "latest", feature = "2025_06_18"))]
    use rust_mcp_schema::*;
    use serde_json::json;

    use super::common::get_message;

    /* ---------------------- CLIENT REQUESTS ---------------------- */
    #[test]
    fn test_client_initialize_request() {
        let message = get_message("req_initialize", LATEST_PROTOCOL_VERSION);
        assert!(matches!(&message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::InitializeRequest(_)))
        ));

        if let ClientMessage::Request(client_message) = message {
            matches!(&client_message.id, &RequestId::Integer(0));
            assert_eq!(client_message.jsonrpc(), JSONRPC_VERSION);
            assert_eq!(client_message.method, "initialize");

            if let RequestFromClient::ClientRequest(ClientRequest::InitializeRequest(request)) = client_message.request {
                assert_eq!(request.method(), "initialize");
                assert_eq!(request.params.protocol_version, LATEST_PROTOCOL_VERSION);
                assert_eq!(request.params.client_info.name, "mcp-inspector");
                assert_eq!(request.params.client_info.version, "0.0.1");
                assert!(request.params.capabilities.roots.is_some());

                if let Some(roots) = request.params.capabilities.roots {
                    assert!(roots.list_changed.is_some());
                    assert!(roots.list_changed.unwrap());
                }
            }
        }
    }

    #[test]
    fn test_client_list_resources_request() {
        let message = get_message("req_resource_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListResourcesRequest(_)))
        ));
    }

    #[test]
    fn test_client_read_resource_request() {
        let message = get_message("req_resource_read", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ReadResourceRequest(_)))
        ));
    }

    #[test]
    fn test_client_list_prompts_request() {
        let message = get_message("req_prompts_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListPromptsRequest(_)))
        ));
    }

    #[test]
    fn test_client_get_prompt_request() {
        let message = get_message("req_prompts_get_1", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::GetPromptRequest(_)))
        ));

        let message = get_message("req_prompts_get_2", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::GetPromptRequest(_)))
        ));
    }

    #[test]
    fn test_client_list_tools_request() {
        let message = get_message("req_tools_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListToolsRequest(_)))
        ));
    }

    #[test]
    fn test_client_call_tool_request() {
        let message = get_message("req_tools_call_1", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::CallToolRequest(_)))
        ));

        let message = get_message("req_tools_call_2", LATEST_PROTOCOL_VERSION);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::CallToolRequest(_)))
        ));

        let message = get_message("req_tools_call_3", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::CallToolRequest(_)))
        ));

        let message = get_message("req_tools_call_4", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::CallToolRequest(_)))
        ));
    }

    #[test]
    fn test_client_ping_request() {
        let message = get_message("req_ping", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::PingRequest(_)))
        ));
    }

    /* ---------------------- CLIENT RESPONSES ---------------------- */
    #[test]
    fn test_list_tools_result() {
        let message = get_message("res_sampling_create_message_2", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Response(client_message)
                if matches!(&client_message.result, ResultFromClient::ClientResult(client_result)
                        if matches!( client_result, ClientResult::CreateMessageResult(_))
                )
        ));
    }
    /* ---------------------- SERVER RESPONSES ---------------------- */

    #[test]
    fn test_server_initialize_result() {
        let message = get_message("res_initialize", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::InitializeResult(_)))
        ));
    }

    #[test]
    fn test_server_list_resources_result() {
        let message = get_message("res_resource_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListResourcesResult(_)))
        ));
    }

    #[test]
    fn test_server_read_resource_result() {
        let message = get_message("res_resource_read", LATEST_PROTOCOL_VERSION);

        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ReadResourceResult(_)))
        ));
    }

    #[test]
    fn test_server_list_prompts_result() {
        let message = get_message("res_prompts_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListPromptsResult(_)))
        ));
    }

    #[test]
    fn test_server_get_prompt_result() {
        let message = get_message("res_prompts_get_1", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::GetPromptResult(_)))
        ));

        let message = get_message("res_prompts_get_2", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::GetPromptResult(_)))
        ));
    }

    #[test]
    fn test_server_list_tools_result() {
        let message = get_message("res_tools_list", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListToolsResult(_)))
        ));
    }

    //TODO: add test case for DRAFT version
    #[cfg(any(feature = "2025_03_26", feature = "2024_11_05"))]
    #[test]
    fn test_server_call_tool_result() {
        let message = get_message("res_tools_call_1", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::CallToolResult(_)))
        ));

        let message = get_message("res_tools_call_2", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::CallToolResult(_)))
        ));

        let message = get_message("res_tools_call_4", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::CallToolResult(_)))
        ));
    }

    #[test]
    fn test_server_ping_result() {
        let message = get_message("res_ping", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(_server_result))
        ));
    }

    /* ---------------------- CLIENT NOTIFICATIONS ---------------------- */

    #[test]
    fn test_client_notifications() {
        //ClientInitializedNotification
        let message = get_message("ntf_initialized", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::InitializedNotification(_)))
        ));

        //ClientRootsListChangedNotification
        let message = get_message("ntf_root_list_changed", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::RootsListChangedNotification(_)))
        ));

        //ClientCancelledNotification
        let message = get_message("ntf_cancelled", LATEST_PROTOCOL_VERSION);

        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::CancelledNotification(notification) if notification.params.reason == Some("Request timed out".to_string())))
        ));
    }

    /* ---------------------- SERVER REQUESTS ---------------------- */
    #[test]
    fn test_server_requests() {
        //ServerCreateMessageRequest
        let message = get_message("req_sampling_create_message_1", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Request(server_message)
                if matches!(&server_message.request,RequestFromServer::ServerRequest(server_request)
                if matches!( server_request, ServerRequest::CreateMessageRequest(_)))
        ));

        let message = get_message("req_sampling_create_message_2", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Request(server_message)
                if matches!(&server_message.request,RequestFromServer::ServerRequest(server_request)
                if matches!( server_request, ServerRequest::CreateMessageRequest(_)))
        ));
    }

    /* ---------------------- CLIENT & SERVER ERRORS ---------------------- */

    #[test]
    fn test_errors() {
        let message: ClientMessage = get_message("err_sampling_rejected", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ClientMessage::Error(_)));

        let message: ServerMessage = get_message("err_sampling_rejected", LATEST_PROTOCOL_VERSION);
        assert!(matches!(message, ServerMessage::Error(_)));
    }

    #[test]
    fn test_deserialize_with_wrong_method() {
        let payload = r#"{"method":"sampling/INVALID","params":{"maxTokens":0,"messages":[]}}"#;
        let result = serde_json::from_str::<CreateMessageRequest>(payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_with_wrong_jsonrpc_version() {
        let payload = r#"{"error":{"code":-32603,"message":"Internal error"},"id":0,"jsonrpc":"1.0"}"#;

        let result = serde_json::from_str::<JsonrpcError>(payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_server_notification_deserialization() {
        let json = json!({
            "method": "notifications/progress",
            "params": {
                "progress": 50,
                "status": "test",
                "task_id": "test",
                "progressToken":"xyz"
            }
        });

        let notif: ServerNotification = serde_json::from_value(json).unwrap();
        if let ServerNotification::ProgressNotification(req) = notif {
            assert_eq!(req.method(), "notifications/progress");
        } else {
            panic!("Unexpected variant");
        }
    }
}
