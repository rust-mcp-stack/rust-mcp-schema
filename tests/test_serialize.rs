#[path = "common/common.rs"]
pub mod common;

mod test_serialize {
    use std::str::FromStr;
    use std::vec;

    #[cfg(feature = "2024_11_05")]
    use rust_mcp_schema::mcp_2024_11_05::schema_utils::*;
    #[cfg(feature = "2024_11_05")]
    use rust_mcp_schema::mcp_2024_11_05::*;

    #[cfg(feature = "2025_03_26")]
    use rust_mcp_schema::mcp_2025_03_26::schema_utils::*;
    #[cfg(feature = "2025_03_26")]
    use rust_mcp_schema::mcp_2025_03_26::*;

    #[cfg(feature = "draft")]
    use rust_mcp_schema::mcp_draft::schema_utils::*;
    #[cfg(feature = "draft")]
    use rust_mcp_schema::mcp_draft::*;

    #[cfg(feature = "latest")]
    use rust_mcp_schema::schema_utils::*;
    #[cfg(feature = "latest")]
    use rust_mcp_schema::*;

    use serde_json::json;

    use super::common::re_serialize;

    /* ---------------------- CLIENT REQUESTS ---------------------- */
    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_initialize_request() {
        // create a ClientMessage
        let request = InitializeRequest::new(InitializeRequestParams {
            capabilities: ClientCapabilities {
                experimental: None,
                roots: None,
                sampling: None,
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                elicitation: None,
            },
            client_info: Implementation {
                name: "client-name".to_string(),
                version: "0.0.1".to_string(),
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                title: None,
            },
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        });

        let client_request = ClientRequest::InitializeRequest(request);

        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(client_request.clone()),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(message.is_request());
        assert!(!message.is_response());
        assert!(!message.is_notification());
        assert!(!message.is_error());
        assert!(message.message_type() == MessageTypes::Request);
        assert!(
            matches!(message.request_id(), Some(request_id) if matches!(request_id , RequestId::Integer(r) if *r == 15))
        );

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::InitializeRequest(_)))
        ));

        // test From<ClientRequest> for RequestFromClient
        let message: ClientMessage =
            ClientMessage::Request(ClientJsonrpcRequest::new(RequestId::Integer(15), client_request.into()));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::InitializeRequest(_)))
        ));
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_list_resources_request() {
        // create a ClientMessage
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::ListResourcesRequest(ListResourcesRequest::new(None))),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListResourcesRequest(_)))
        ));
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_read_resource_request() {
        // create a ClientMessage
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::ReadResourceRequest(ReadResourceRequest::new(
                ReadResourceRequestParams {
                    uri: "test://static/resource/1".to_string(),
                },
            ))),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ReadResourceRequest(_)))
        ));
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_list_prompts_request() {
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::ListPromptsRequest(ListPromptsRequest::new(None))),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListPromptsRequest(_)))
        ));
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_get_prompt_request() {
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::GetPromptRequest(GetPromptRequest::new(
                GetPromptRequestParams {
                    name: "simple_prompt".to_string(),
                    arguments: None,
                },
            ))),
        ));

        let message: ClientMessage = re_serialize(message);
        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::GetPromptRequest(_)))
        ));
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_list_tools_request() {
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::ListToolsRequest(ListToolsRequest::new(None))),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::ListToolsRequest(_)))
        ));
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_call_tool_request() {
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::CallToolRequest(CallToolRequest::new(CallToolRequestParams {
                name: "add".to_string(),
                arguments: None,
            }))),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::CallToolRequest(_)))
        ));
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_ping_request() {
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::PingRequest(PingRequest::new(None))),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::PingRequest(_)))
        ));
    }

    #[test]
    fn test_client_custom_request() {
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::CustomRequest(json!({"method":"my_custom_method"})),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
        if matches!(&client_message.request, RequestFromClient::CustomRequest(_)) && client_message.method == "my_custom_method"
        ));

        // test From<serde_json::Value> for RequestFromClient
        let message: ClientMessage = ClientMessage::Request(ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            json!({"method":"my_custom_method"}).into(),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Request(client_message)
                if matches!(&client_message.request, RequestFromClient::CustomRequest(_)) && client_message.method == "my_custom_method"
        ));
    }

    /* ---------------------- CLIENT RESPONSES ---------------------- */
    #[test]
    fn test_list_tools_result() {
        let client_result = ClientResult::CreateMessageResult(CreateMessageResult {
            content: CreateMessageResultContent::TextContent(TextContent::new(
                "This is a stub response.".to_string(),
                None,
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                None,
            )),
            meta: None,
            model: "stub-model".to_string(),
            role: Role::Assistant,
            stop_reason: Some("endTurn".to_string()),
        });

        let message: ClientMessage = ClientMessage::Response(ClientJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromClient::ClientResult(client_result.clone()),
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(!message.is_request());
        assert!(message.is_response());
        assert!(!message.is_notification());
        assert!(!message.is_error());
        assert!(message.message_type() == MessageTypes::Response);
        assert!(
            matches!(message.request_id(), Some(request_id) if matches!(request_id , RequestId::Integer(r) if *r == 15))
        );

        assert!(matches!(message, ClientMessage::Response(client_message)
                if matches!(&client_message.result, ResultFromClient::ClientResult(client_result)
                        if matches!( client_result, ClientResult::CreateMessageResult(_))
                )
        ));

        // test From<ClientResult> for ResultFromClient
        let message: ClientMessage =
            ClientMessage::Response(ClientJsonrpcResponse::new(RequestId::Integer(15), client_result.into()));

        assert!(matches!(message, ClientMessage::Response(client_message)
                if matches!(&client_message.result, ResultFromClient::ClientResult(client_result)
                        if matches!( client_result, ClientResult::CreateMessageResult(_))
                )
        ));
    }

    /* ---------------------- SERVER RESPONSES ---------------------- */

    #[test]
    fn test_server_initialize_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::InitializeResult(InitializeResult {
                capabilities: ServerCapabilities {
                    experimental: None,
                    logging: None,
                    prompts: None,
                    resources: None,
                    tools: None,
                    #[cfg(any(feature = "2025_03_26", feature = "draft", feature = "2025_06_18"))]
                    completions: None,
                },
                instructions: None,
                meta: None,
                protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
                server_info: Implementation {
                    #[cfg(feature = "draft")]
                    icons: vec![],
                    #[cfg(feature = "draft")]
                    website_url: None,
                    name: "example-servers/everything".to_string(),
                    version: "1.0.0".to_string(),
                    #[cfg(any(feature = "2025_06_18", feature = "draft"))]
                    title: None,
                },
            })),
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(!message.is_request());
        assert!(message.is_response());
        assert!(!message.is_notification());
        assert!(!message.is_error());
        assert!(message.message_type() == MessageTypes::Response);

        assert!(
            matches!(message.request_id(), Some(request_id) if matches!(request_id , RequestId::Integer(r) if *r == 15))
        );

        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::InitializeResult(_)))
        ));
    }

    #[test]
    fn test_server_read_resource_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::ReadResourceResult(ReadResourceResult {
                contents: vec![],
                meta: None,
            })),
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ReadResourceResult(_)))
        ));
    }

    #[test]
    fn test_server_list_prompts_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::ListPromptsResult(ListPromptsResult {
                meta: None,
                next_cursor: None,
                prompts: vec![],
            })),
        ));

        let message: ServerMessage = re_serialize(message);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListPromptsResult(_)))
        ));
    }

    #[test]
    fn test_server_get_prompt_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::GetPromptResult(GetPromptResult {
                meta: None,
                description: None,
                messages: vec![],
            })),
        ));

        let message: ServerMessage = re_serialize(message);
        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::GetPromptResult(_)))
        ));
    }

    #[test]
    fn test_server_list_tools_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::ListToolsResult(ListToolsResult {
                meta: None,
                next_cursor: None,
                tools: vec![],
            })),
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::ListToolsResult(_)))
        ));
    }

    #[cfg(any(feature = "2025_03_26", feature = "2024_11_05"))]
    #[test]
    fn test_server_call_tool_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::CallToolResult(CallToolResult {
                meta: None,
                content: vec![],
                is_error: None,
            })),
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::CallToolResult(_)))
        ));
    }

    #[test]
    fn test_server_custom_result() {
        let custom_result: serde_json::Map<String, serde_json::Value> = json!({
            "custom_key":"custom_value",
            "custom_number": 15.21
        })
        .as_object()
        .unwrap()
        .clone();

        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::Result(Result {
                meta: None,
                extra: Some(custom_result),
            })),
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(matches!(message, ServerMessage::Response(server_message)
                if matches!(&server_message.result, ResultFromServer::ServerResult(server_result)
                if matches!(server_result, ServerResult::Result(result)
                if result.extra.is_some()
            ))
        ));
    }

    /* ---------------------- CLIENT NOTIFICATIONS ---------------------- */

    #[test]
    fn test_client_initialized_notification() {
        let init_notification =
            InitializedNotification::new(Some(InitializedNotificationParams { meta: None, extra: None }));

        let message: ClientMessage =
            ClientMessage::Notification(ClientJsonrpcNotification::new(NotificationFromClient::ClientNotification(
                ClientNotification::InitializedNotification(init_notification.clone()),
            )));

        let message: ClientMessage = re_serialize(message);

        assert!(!message.is_request());
        assert!(!message.is_response());
        assert!(message.is_notification());
        assert!(!message.is_error());
        assert!(message.message_type() == MessageTypes::Notification);
        assert!(message.request_id().is_none());

        assert!(matches!(message, ClientMessage::Notification(client_message)
        if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::InitializedNotification(_)))
        ));

        // test  From<InitializedNotification> for NotificationFromClient
        let message: ClientMessage =
            ClientMessage::Notification(ClientJsonrpcNotification::new(init_notification.clone().into()));

        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::InitializedNotification(_)))
        ));

        // test  From<InitializedNotification> for ClientJsonrpcNotification
        let message: ClientMessage = ClientMessage::Notification(init_notification.into());

        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::InitializedNotification(_)))
        ));
    }

    #[test]
    fn test_client_root_list_changed_notification() {
        let message: ClientMessage =
            ClientMessage::Notification(ClientJsonrpcNotification::new(NotificationFromClient::ClientNotification(
                ClientNotification::RootsListChangedNotification(RootsListChangedNotification::new(None)),
            )));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::RootsListChangedNotification(_)))
        ));
    }

    #[test]
    fn test_client_cancelled_notification() {
        let message: ClientMessage =
            ClientMessage::Notification(ClientJsonrpcNotification::new(NotificationFromClient::ClientNotification(
                ClientNotification::CancelledNotification(CancelledNotification::new(CancelledNotificationParams {
                    reason: Some("Request timed out".to_string()),
                    request_id: RequestId::Integer(15),
                })),
            )));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromClient::ClientNotification(client_notification)
                if matches!( client_notification, ClientNotification::CancelledNotification(notification) if notification.params.reason == Some("Request timed out".to_string())))
        ));
    }

    #[test]
    fn test_client_custom_notification() {
        let message: ClientMessage = ClientMessage::Notification(ClientJsonrpcNotification::new(
            NotificationFromClient::CustomNotification(json!({"method":"my_notification"})),
        ));

        let message: ClientMessage = re_serialize(message);

        // test Display trait
        let str = message.to_string();
        assert_eq!(str, "{\"jsonrpc\":\"2.0\",\"method\":\"my_notification\",\"params\":{\"method\":\"my_notification\",\"params\":{\"method\":\"my_notification\"}}}");

        assert!(matches!(message, ClientMessage::Notification(client_message)
                if matches!(&client_message.notification, NotificationFromClient::CustomNotification(_)) && client_message.method == "my_notification"
        ));
    }

    /* ---------------------- SERVER NOTIFICATIONS ---------------------- */
    #[test]
    fn test_server_cancel_notification() {
        let cancel_notification = CancelledNotification::new(CancelledNotificationParams {
            reason: Some("Request timed out".to_string()),
            request_id: RequestId::Integer(15),
        });
        let message: ServerMessage =
            ServerMessage::Notification(ServerJsonrpcNotification::new(NotificationFromServer::ServerNotification(
                ServerNotification::CancelledNotification(cancel_notification.clone()),
            )));

        let message: ServerMessage = re_serialize(message);

        assert!(!message.is_request());
        assert!(!message.is_response());
        assert!(message.is_notification());
        assert!(!message.is_error());
        assert!(message.message_type() == MessageTypes::Notification);
        assert!(message.request_id().is_none());

        assert!(matches!(message, ServerMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromServer::ServerNotification(client_notification)
                if matches!( client_notification, ServerNotification::CancelledNotification(_)))
        ));

        // test From<CancelledNotification> for NotificationFromServer
        let message: ServerMessage =
            ServerMessage::Notification(ServerJsonrpcNotification::new(cancel_notification.clone().into()));

        assert!(matches!(message, ServerMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromServer::ServerNotification(client_notification)
                if matches!( client_notification, ServerNotification::CancelledNotification(_)))
        ));

        // test From<CancelledNotification> for ServerNotification
        let message: ServerMessage = ServerMessage::Notification(cancel_notification.into());

        assert!(matches!(message, ServerMessage::Notification(client_message)
                if matches!(&client_message.notification,NotificationFromServer::ServerNotification(client_notification)
                if matches!( client_notification, ServerNotification::CancelledNotification(_)))
        ));
    }

    /* ---------------------- SERVER REQUESTS ---------------------- */
    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_server_requests() {
        let message: ServerMessage = ServerMessage::Request(ServerJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromServer::ServerRequest(ServerRequest::CreateMessageRequest(CreateMessageRequest::new(
                CreateMessageRequestParams {
                    include_context: None,
                    max_tokens: 21,
                    messages: vec![],
                    metadata: None,
                    model_preferences: None,
                    stop_sequences: vec![],
                    system_prompt: None,
                    temperature: None,
                },
            ))),
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(message.is_request());
        assert!(!message.is_response());
        assert!(!message.is_notification());
        assert!(!message.is_error());
        assert!(message.message_type() == MessageTypes::Request);

        assert!(
            matches!(message.request_id(), Some(request_id) if matches!(request_id , RequestId::Integer(r) if *r == 15))
        );

        assert!(matches!(message, ServerMessage::Request(server_message)
        if matches!(&server_message.request,RequestFromServer::ServerRequest(server_request)
                if matches!( server_request, ServerRequest::CreateMessageRequest(_)))
        ));
    }

    #[test]
    fn test_client_custom_server_request() {
        let message: ServerMessage = ServerMessage::Request(ServerJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromServer::CustomRequest(json!({"method":"my_custom_method"})),
        ));

        // test Display trait
        let str = message.to_string();
        assert_eq!(
            str,
            "{\"id\":15,\"jsonrpc\":\"2.0\",\"method\":\"my_custom_method\",\"params\":{\"method\":\"my_custom_method\"}}"
        );

        let message: ServerMessage = re_serialize(message);

        assert!(matches!(message, ServerMessage::Request(server_message)
                if matches!(&server_message.request, RequestFromServer::CustomRequest(_)) && server_message.method == "my_custom_method"
        ));
    }

    /* ---------------------- CLIENT & SERVER ERRORS ---------------------- */

    #[test]
    fn test_errors() {
        let message: ClientMessage = ClientMessage::Error(JsonrpcError::create(
            RequestId::Integer(15),
            RpcErrorCodes::INTERNAL_ERROR,
            "err_sampling_rejected".to_string(),
            None,
        ));

        let message: ClientMessage = re_serialize(message);

        assert!(matches!(message, ClientMessage::Error(_)));
        assert!(!message.is_request());
        assert!(!message.is_response());
        assert!(!message.is_notification());
        assert!(message.is_error());
        assert!(message.message_type() == MessageTypes::Error);

        assert!(
            matches!(message.request_id(), Some(request_id) if matches!(request_id , RequestId::Integer(r) if *r == 15))
        );

        let message: ServerMessage = ServerMessage::Error(JsonrpcError::create(
            RequestId::Integer(15),
            RpcErrorCodes::INTERNAL_ERROR,
            "err_sampling_rejected".to_string(),
            None,
        ));

        let message: ServerMessage = re_serialize(message);

        assert!(matches!(message, ServerMessage::Error(_)));

        assert!(!message.is_request());
        assert!(!message.is_response());
        assert!(!message.is_notification());
        assert!(message.is_error());
        assert!(message.message_type() == MessageTypes::Error);

        assert!(
            matches!(message.request_id(), Some(request_id) if matches!(request_id , RequestId::Integer(r) if *r == 15))
        );
    }

    /* ---------------------- RpcError ---------------------- */
    #[test]
    fn test_new() {
        let error_object = RpcError::new(
            RpcErrorCodes::METHOD_NOT_FOUND,
            "Error Message!".to_string(),
            Some(json!({"details":"error detail"})),
        );

        let error_object: RpcError = re_serialize(error_object);

        assert_eq!(error_object.code, -32601);
        assert_eq!(error_object.message, "Error Message!".to_string());
        matches!(error_object.data, Some(data) if data["details"].as_str().unwrap() == "error detail");
    }

    #[test]
    fn test_method_not_found() {
        let error_object = RpcError::method_not_found();
        assert_eq!(error_object.code, -32601);
        assert_eq!(error_object.message, "Method not found".to_string());
        assert!(error_object.data.is_none());

        // builder pattern
        let error_object = RpcError::method_not_found()
            .with_message("Error Message!".to_string())
            .with_data(Some(json!({"details":"error detail"})));

        let error_object: RpcError = re_serialize(error_object);

        assert_eq!(error_object.code, -32601);
        assert_eq!(error_object.message, "Error Message!".to_string());
        matches!(error_object.data, Some(data) if data["details"].as_str().unwrap() == "error detail");
    }

    #[test]
    fn test_invalid_params() {
        let error_object = RpcError::invalid_params();
        assert_eq!(error_object.code, -32602);
        assert_eq!(error_object.message, "Invalid params".to_string());
        assert!(error_object.data.is_none());

        // builder pattern
        let error_object = RpcError::invalid_params()
            .with_message("Error Message!".to_string())
            .with_data(Some(json!({"details":"error detail"})));

        let error_object: RpcError = re_serialize(error_object);

        assert_eq!(error_object.code, -32602);
        assert_eq!(error_object.message, "Error Message!".to_string());
        matches!(error_object.data, Some(data) if data["details"].as_str().unwrap() == "error detail");
    }

    #[test]
    fn test_invalid_request() {
        let error_object = RpcError::invalid_request();
        assert_eq!(error_object.code, -32600);
        assert_eq!(error_object.message, "Invalid request".to_string());
        assert!(error_object.data.is_none());

        // builder pattern
        let error_object = RpcError::invalid_request()
            .with_message("Error Message!".to_string())
            .with_data(Some(json!({"details":"error detail"})));

        let error_object: RpcError = re_serialize(error_object);

        assert_eq!(error_object.code, -32600);
        assert_eq!(error_object.message, "Error Message!".to_string());
        matches!(error_object.data, Some(data) if data["details"].as_str().unwrap() == "error detail");
    }

    #[test]
    fn test_internal_error() {
        let error_object = RpcError::internal_error();
        assert_eq!(error_object.code, -32603);
        assert_eq!(error_object.message, "Internal error".to_string());
        assert!(error_object.data.is_none());

        // builder pattern
        let error_object = RpcError::internal_error()
            .with_message("Error Message!".to_string())
            .with_data(Some(json!({"details":"error detail"})));

        let error_object: RpcError = re_serialize(error_object);

        assert_eq!(error_object.code, -32603);
        assert_eq!(error_object.message, "Error Message!".to_string());
        matches!(error_object.data, Some(data) if data["details"].as_str().unwrap() == "error detail");
    }

    #[test]
    fn test_parse_error() {
        let error_object = RpcError::parse_error();
        assert_eq!(error_object.code, -32700);
        assert_eq!(error_object.message, "Parse error".to_string());
        assert!(error_object.data.is_none());

        // builder pattern
        let error_object = RpcError::parse_error()
            .with_message("Error Message!".to_string())
            .with_data(Some(json!({"details":"error detail"})));

        let error_object: RpcError = re_serialize(error_object);

        assert_eq!(error_object.code, -32700);
        assert_eq!(error_object.message, "Error Message!".to_string());
        matches!(error_object.data, Some(data) if data["details"].as_str().unwrap() == "error detail");
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_jsonrpc_request() {
        let message = ClientJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromClient::ClientRequest(ClientRequest::PingRequest(PingRequest::new(None))),
        );

        let message_str = message.to_string();

        let message: ClientJsonrpcRequest = ClientJsonrpcRequest::from_str(&message_str).unwrap();

        assert!(matches!(&message.request, RequestFromClient::ClientRequest(client_request)
                if matches!(client_request, ClientRequest::PingRequest(_))));
    }

    #[test]
    fn test_client_jsonrpc_notification() {
        let message = ClientJsonrpcNotification::new(NotificationFromClient::CustomNotification(json!({"method":"notify"})));

        let message_str = message.to_string();

        let message: ClientJsonrpcNotification = ClientJsonrpcNotification::from_str(&message_str).unwrap();

        assert!(
            matches!(&message.notification, NotificationFromClient::CustomNotification(client_request)
                if client_request["method"] == "notify")
        );
    }

    #[test]
    fn test_server_jsonrpc_request() {
        let message = ServerJsonrpcRequest::new(
            RequestId::Integer(15),
            RequestFromServer::CustomRequest(json!({"method":"req"})),
        );

        let message_str = message.to_string();

        let message: ServerJsonrpcRequest = ServerJsonrpcRequest::from_str(&message_str).unwrap();

        assert!(matches!(&message.request, RequestFromServer::CustomRequest(request)
                if request["method"] == "req"));
    }

    #[test]
    fn test_server_jsonrpc_notification() {
        let message = ServerJsonrpcNotification::new(NotificationFromServer::CustomNotification(json!({"method":"notify"})));

        let message_str = message.to_string();

        let message: ServerJsonrpcNotification = ServerJsonrpcNotification::from_str(&message_str).unwrap();

        assert!(
            matches!(&message.notification, NotificationFromServer::CustomNotification(server_request)
                if server_request["method"] == "notify")
        );
    }

    #[test]
    fn test_server_list_resources_result() {
        let message: ServerMessage = ServerMessage::Response(ServerJsonrpcResponse::new(
            RequestId::Integer(15),
            ResultFromServer::ServerResult(ServerResult::ListResourcesResult(ListResourcesResult {
                meta: None,
                next_cursor: None,
                resources: vec![Resource {
                    #[cfg(feature = "draft")]
                    icons: vec![],
                    annotations: None,
                    description: None,
                    mime_type: None,
                    name: "Resource 1".to_string(),
                    uri: "test://static/resource/1".to_string(),
                    size: None,
                    #[cfg(any(feature = "2025_06_18", feature = "draft"))]
                    meta: None,
                    #[cfg(any(feature = "2025_06_18", feature = "draft"))]
                    title: None,
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
