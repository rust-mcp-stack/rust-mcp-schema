mod test_deserialize {

    #[cfg(feature = "2024_11_05")]
    use rust_mcp_schema::mcp_2024_11_05::*;

    #[cfg(feature = "2025_03_26")]
    use rust_mcp_schema::mcp_2025_03_26::*;

    #[cfg(feature = "draft")]
    use rust_mcp_schema::mcp_draft::*;

    #[cfg(any(feature = "latest", feature = "2025_06_18"))]
    use rust_mcp_schema::*;

    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use serde_json::{Map, Value};
    use std::collections::HashMap;

    /* ---------------------- TESTS ---------------------- */

    // Helper to test serialization and deserialization
    fn test_serde<T>(original: &T)
    where
        T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        // Serialize the original object to JSON
        let json = serde_json::to_string(original).expect("Failed to serialize original object");

        // Deserialize back to the same type
        let deserialized: T = serde_json::from_str(&json).expect("Failed to deserialize JSON");

        // Serialize the deserialized object to JSON
        let json_deserialized = serde_json::to_string(&deserialized).expect("Failed to serialize deserialized object");

        // Compare the JSON strings to ensure consistency
        assert_eq!(json, json_deserialized, "JSON serialization mismatch for {original:?}");
    }

    #[cfg(not(feature = "2024_11_05"))]
    #[test]
    fn test_annotations() {
        let ann = Annotations {
            audience: vec![Role::User],
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            last_modified: Some("2025-01-01T00:00:00Z".to_string()),
            priority: Some(0.5),
        };
        test_serde(&ann);

        // Edge case: empty
        let ann_empty = Annotations::default();
        test_serde(&ann_empty);
    }

    #[cfg(not(feature = "2024_11_05"))]
    #[test]
    fn test_audio_content() {
        let audio = AudioContent::new(
            "YmFzZTY0ZGF0YQ==".to_string(),
            "audio/mpeg".to_string(),
            Some(Annotations::default()),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            Some(Map::new()),
        );

        test_serde(&audio);

        // Edge case: minimal
        let audio_min = AudioContent::new(
            "YmFzZTY0ZGF0YQ==".to_string(),
            "audio/mpeg".to_string(),
            None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            None,
        );
        test_serde(&audio_min);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_base_metadata() {
        let meta = BaseMetadata {
            name: "test_name".to_string(),
            title: Some("Test Title".to_string()),
        };
        test_serde(&meta);

        // Edge case: minimal
        let meta_min = BaseMetadata {
            name: "test".to_string(),
            title: None,
        };
        test_serde(&meta_min);
    }

    #[test]
    fn test_blob_resource_contents() {
        let blob = BlobResourceContents {
            blob: "YmFzZTY0YmxvYg==".to_string(),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            meta: Some(Map::new()),
            mime_type: Some("application/octet-stream".to_string()),
            uri: "https://example.com/blob".to_string(),
        };
        test_serde(&blob);

        // Edge case: minimal
        let blob_min = BlobResourceContents {
            blob: "YmFzZTY0YmxvYg==".to_string(),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            meta: None,
            mime_type: None,
            uri: "https://example.com/blob".to_string(),
        };
        test_serde(&blob_min);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_boolean_schema() {
        let default = Some(true);
        let description = Some("Test boolean".to_string());
        let title = Some("Bool Title".to_string());

        let schema = BooleanSchema::new(default, description, title);
        test_serde(&schema);

        // Edge case: minimal
        let schema_min = BooleanSchema::new(None, None, None);
        test_serde(&schema_min);
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_call_tool_request() {
        let params = CallToolRequestParams {
            arguments: Some(Map::new()),
            name: "test_tool".to_string(),
        };
        let req = CallToolRequest::new(params);
        test_serde(&req);

        // Validation error test
        let invalid_json = json!({
            "method": "wrong/method",
            "params": {
                "arguments": {},
                "name": "test_tool"
            }
        });
        let result: std::result::Result<CallToolRequest, _> = serde_json::from_value(invalid_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected field `method` in struct `CallToolRequest` as const value 'tools/call'"));
    }

    #[test]
    fn test_call_tool_result() {
        let result = CallToolResult {
            content: vec![],
            is_error: Some(false),
            meta: Some(Map::new()),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            structured_content: Some(Map::new()),
        };
        test_serde(&result);

        // Edge case: minimal
        let result_min = CallToolResult {
            content: vec![],
            is_error: None,
            meta: None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            structured_content: None,
        };
        test_serde(&result_min);
    }

    #[test]
    fn test_cancelled_notification() {
        let params = CancelledNotificationParams {
            reason: Some("test reason".to_string()),
            request_id: RequestId::Integer(1),
        };
        let notif = CancelledNotification::new(params);
        test_serde(&notif);

        // Validation error test
        let invalid_json = json!({
            "method": "wrong/method",
            "params": {
                "requestId": 1,
                "reason": "test"
            }
        });
        let result: std::result::Result<CancelledNotification, _> = serde_json::from_value(invalid_json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected field `method` in struct `CancelledNotification` as const value 'notifications/cancelled'"));
    }

    #[test]
    fn test_client_capabilities() {
        let capabilities = ClientCapabilities {
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            elicitation: Some(Map::new()),
            experimental: Some(HashMap::new()),
            roots: Some(ClientCapabilitiesRoots::default()),
            sampling: Some(Map::new()),
        };
        test_serde(&capabilities);

        // Edge case: empty
        let cap_empty = ClientCapabilities::default();
        test_serde(&cap_empty);
    }

    #[test]
    fn test_client_capabilities_roots() {
        let roots = ClientCapabilitiesRoots {
            list_changed: Some(true),
        };
        test_serde(&roots);

        // Edge case: empty
        let roots_empty = ClientCapabilitiesRoots::default();
        test_serde(&roots_empty);
    }

    #[test]
    fn test_client_notification() {
        let notif = ClientNotification::CancelledNotification(CancelledNotification::new(CancelledNotificationParams {
            reason: None,
            request_id: RequestId::Integer(0),
        }));
        test_serde(&notif);
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_client_request() {
        let req = ClientRequest::CallToolRequest(CallToolRequest::new(CallToolRequestParams {
            arguments: None,
            name: "name".to_string(),
        }));
        test_serde(&req);
    }

    #[test]
    fn test_client_result() {
        let result = ClientResult::CreateMessageResult(CreateMessageResult {
            content: CreateMessageResultContent::TextContent(TextContent::new(
                "test".to_string(),
                None,
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                None,
            )),
            meta: None,
            model: "model".to_string(),
            role: Role::Assistant,
            stop_reason: None,
        });
        test_serde(&result);
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_complete_request() {
        let argument = CompleteRequestParamsArgument {
            name: "test".to_string(),
            value: "test".to_string(),
        };

        let params = CompleteRequestParams {
            argument,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            context: None,
            ref_: CompleteRequestParamsRef::PromptReference(PromptReference::new(
                "test".to_string(),
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                None,
            )),
        };

        let req = CompleteRequest::new(params);
        test_serde(&req);
    }

    #[test]
    fn test_complete_result() {
        let completion = CompleteResultCompletion {
            has_more: Some(false),
            total: Some(0),
            values: vec![],
        };

        let result = CompleteResult { completion, meta: None };
        test_serde(&result);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_content_block() {
        let block = ContentBlock::TextContent(TextContent::new(
            "test".to_string(),
            None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            None,
        ));
        test_serde(&block);
    }

    #[test]
    fn test_create_message_request() {
        let params = CreateMessageRequestParams {
            include_context: None,
            max_tokens: 100,
            messages: vec![],
            metadata: None,
            model_preferences: None,
            stop_sequences: vec![],
            system_prompt: None,
            temperature: None,
        };

        let req = CreateMessageRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            params,
        );
        test_serde(&req);
    }

    #[test]
    fn test_create_message_result() {
        let result = CreateMessageResult {
            content: CreateMessageResultContent::TextContent(TextContent::new(
                "test".to_string(),
                None,
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                None,
            )),
            meta: None,
            model: "model".to_string(),
            role: Role::Assistant,
            stop_reason: None,
        };
        test_serde(&result);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_elicit_request() {
        let params = ElicitRequestParams {
            message: "test".to_string(),
            requested_schema: ElicitRequestedSchema::new(HashMap::new(), vec![]),
        };

        let req = ElicitRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            params,
        );
        test_serde(&req);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_elicit_result() {
        let result = ElicitResult {
            meta: None,
            action: ElicitResultAction::Accept,
            content: None,
        };
        test_serde(&result);
    }

    #[test]
    fn test_embedded_resource() {
        let resource = EmbeddedResource::new(
            EmbeddedResourceResource::TextResourceContents(TextResourceContents {
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                meta: None,
                mime_type: None,
                text: "test".to_string(),
                uri: "ice://test".to_string(),
            }),
            None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            None,
        );

        test_serde(&resource);
    }

    #[test]
    fn test_get_prompt_request() {
        let params = GetPromptRequestParams {
            name: "test".to_string(),
            arguments: None,
        };

        let req = GetPromptRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            params,
        );
        test_serde(&req);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_get_prompt_result() {
        let block = ContentBlock::TextContent(TextContent::new("test".to_string(), None, None));

        let result = GetPromptResult {
            meta: None,
            description: None,
            messages: vec![PromptMessage {
                content: block,
                role: Role::Assistant,
            }],
        };
        test_serde(&result);
    }

    #[test]
    fn test_image_content() {
        let annotations = None;
        let data = "base64".to_string();
        #[cfg(any(feature = "draft", feature = "2025_06_18"))]
        let meta = None;
        let mime_type = "image/png".to_string();

        let content = ImageContent::new(
            data,
            mime_type,
            annotations,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            meta,
        );

        test_serde(&content);
    }

    #[test]
    fn test_initialize_request() {
        let params = InitializeRequestParams {
            capabilities: ClientCapabilities {
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                elicitation: None,
                experimental: None,
                roots: None,
                sampling: None,
            },
            protocol_version: "2025-06-18".to_string(),
            client_info: Implementation {
                name: "name".to_string(),
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                title: Some("title".to_string()),
                version: "version".to_string(),
                #[cfg(feature = "draft")]
                icons: vec![],
                #[cfg(feature = "draft")]
                website_url: Some("https://github.com/rust-mcp-stack/rust-mcp-sdk".to_string()),
            },
        };

        let req = InitializeRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            params,
        );
        test_serde(&req);
    }

    #[test]
    fn test_initialize_result() {
        let result = InitializeResult {
            capabilities: ServerCapabilities::default(),
            meta: None,
            protocol_version: "2025-06-18".to_string(),
            instructions: None,
            server_info: Implementation {
                name: "name".to_string(),
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                title: Some("title".to_string()),
                version: "version".to_string(),
                #[cfg(feature = "draft")]
                icons: vec![],
                #[cfg(feature = "draft")]
                website_url: Some("https://github.com/rust-mcp-stack/rust-mcp-sdk".to_string()),
            },
        };
        test_serde(&result);
    }

    #[test]
    fn test_initialized_notification() {
        let params = InitializedNotificationParams { meta: None, extra: None };

        let notif = InitializedNotification::new(Some(params));
        test_serde(&notif);
    }

    #[test]
    fn test_logging_message_notification() {
        let params = LoggingMessageNotificationParams {
            level: LoggingLevel::Info,
            data: Value::String("data".to_string()),
            logger: Some("fogger".to_string()),
        };

        let notif = LoggingMessageNotification::new(params);
        test_serde(&notif);
    }

    #[test]
    fn test_list_prompts_request() {
        let params = ListPromptsRequestParams { cursor: None };

        let req = ListPromptsRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            Some(params),
        );
        test_serde(&req);
    }

    #[test]
    fn test_list_prompts_result() {
        let result = ListPromptsResult {
            prompts: vec![],
            meta: None,
            next_cursor: None,
        };
        test_serde(&result);
    }

    #[test]
    fn test_list_resources_request() {
        let params = ListResourcesRequestParams { cursor: None };

        let req = ListResourcesRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            Some(params),
        );
        test_serde(&req);
    }

    #[test]
    fn test_list_resources_result() {
        let result = ListResourcesResult {
            resources: vec![],
            meta: None,
            next_cursor: None,
        };
        test_serde(&result);
    }

    #[test]
    fn test_list_resource_templates_request() {
        let params = ListResourceTemplatesRequestParams { cursor: None };

        let req = ListResourceTemplatesRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            Some(params),
        );
        test_serde(&req);
    }

    #[test]
    fn test_list_resource_templates_result() {
        let result = ListResourceTemplatesResult {
            meta: None,
            next_cursor: None,
            resource_templates: vec![ResourceTemplate {
                annotations: None,
                description: None,
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                meta: None,
                mime_type: None,
                name: "name".to_string(),
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                title: None,
                uri_template: "ice://something".to_string(),
                #[cfg(feature = "draft")]
                icons: vec![],
            }],
        };
        test_serde(&result);
    }

    #[test]
    fn test_list_roots_request() {
        let req = ListRootsRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            Some(ListRootsRequestParams { meta: None, extra: None }),
        );
        test_serde(&req);
    }

    #[test]
    fn test_list_roots_result() {
        let result = ListRootsResult {
            roots: vec![],
            meta: None,
        };
        test_serde(&result);
    }

    #[test]
    fn test_list_tools_request() {
        let params = ListToolsRequestParams { cursor: None };
        let req = ListToolsRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            Some(params),
        );
        test_serde(&req);
    }

    #[test]
    fn test_list_tools_result() {
        let result = ListToolsResult {
            tools: vec![],
            meta: None,
            next_cursor: None,
        };
        test_serde(&result);
    }

    #[test]
    fn test_model_preferences() {
        let pref = ModelPreferences {
            cost_priority: None,
            hints: vec![],
            intelligence_priority: None,
            speed_priority: None,
        };
        test_serde(&pref);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_number_schema() {
        let schema = NumberSchema {
            description: None,
            maximum: None,
            minimum: None,
            title: None,
            type_: NumberSchemaType::Integer,
            #[cfg(feature = "draft")]
            default: None,
        };
        test_serde(&schema);
    }

    #[test]
    fn test_ping_request() {
        let params = PingRequestParams { meta: None, extra: None };
        let req = PingRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            Some(params),
        );
        test_serde(&req);
    }

    #[test]
    fn test_progress_notification() {
        let params = ProgressNotificationParams {
            progress: 52.,
            #[cfg(not(feature = "2024_11_05"))]
            message: None,
            progress_token: ProgressToken::Integer(15),
            total: None,
        };

        let notif = ProgressNotification::new(params);
        test_serde(&notif);
    }

    #[test]
    fn test_prompt() {
        let prompt = Prompt {
            description: None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            meta: None,
            name: "test".to_string(),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            title: None,
            arguments: vec![PromptArgument {
                description: None,
                name: "name".to_string(),
                required: None,
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                title: None,
            }],
            #[cfg(feature = "draft")]
            icons: vec![],
        };
        test_serde(&prompt);
    }

    #[test]
    fn test_prompt_list_changed_notification() {
        let params = PromptListChangedNotificationParams { meta: None, extra: None };
        let notif = PromptListChangedNotification::new(Some(params));
        test_serde(&notif);
    }

    #[test]
    fn test_prompt_reference() {
        let ref_ = PromptReference::new(
            "name".to_string(),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            Some("title".to_string()),
        );

        test_serde(&ref_);
    }

    #[test]
    fn test_read_resource_request() {
        let params = ReadResourceRequestParams { uri: "test".to_string() };
        let req = ReadResourceRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            params,
        );
        test_serde(&req);
    }

    #[test]
    fn test_read_resource_result() {
        let result = ReadResourceResult {
            meta: None,
            contents: vec![ReadResourceResultContentsItem::TextResourceContents(TextResourceContents {
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                meta: None,
                mime_type: None,
                text: "test".to_string(),
                uri: "ice://test".to_string(),
            })],
        };
        test_serde(&result);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_resource_link() {
        let annotations = None;
        let meta = None;
        let uri = "test".to_string();
        let description = None;
        let mime_type = None;
        let name = "name".to_string();
        let size = None;
        let title = None;
        let link = ResourceLink::new(
            #[cfg(feature = "draft")]
            vec![],
            name,
            uri,
            annotations,
            description,
            meta,
            mime_type,
            size,
            title,
        );
        test_serde(&link);
    }

    #[test]
    fn test_resource_list_changed_notification() {
        let params = ResourceListChangedNotificationParams { meta: None, extra: None };
        let notif = ResourceListChangedNotification::new(Some(params));
        test_serde(&notif);
    }

    #[test]
    fn test_resource_template() {
        let template = ResourceTemplate {
            annotations: None,
            description: None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            meta: None,
            name: "test".to_string(),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            title: None,
            mime_type: Some("mime/pine".to_string()),
            uri_template: "ice://something".to_string(),
            #[cfg(feature = "draft")]
            icons: vec![],
        };
        test_serde(&template);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_resource_template_reference() {
        let ref_ = ResourceTemplateReference::new("ice://something".to_string());
        test_serde(&ref_);
    }

    #[test]
    fn test_resource_updated_notification() {
        let params = ResourceUpdatedNotificationParams {
            uri: "ice://something".to_string(),
        };
        let notif = ResourceUpdatedNotification::new(params);
        test_serde(&notif);
    }

    #[test]
    fn test_result() {
        let result = Result { meta: None, extra: None };
        test_serde(&result);
    }

    #[test]
    fn test_role() {
        let role = Role::User;
        test_serde(&role);
    }

    #[test]
    fn test_sampling_message() {
        let content = TextContent::new(
            "SamplingMessageContent".to_string(),
            None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            None,
        );

        let message = SamplingMessage {
            content: SamplingMessageContent::TextContent(content),
            role: Role::User,
        };
        test_serde(&message);
    }

    #[test]
    fn test_server_capabilities() {
        let cap = ServerCapabilities::default();
        test_serde(&cap);
    }

    #[test]
    fn test_server_capabilities_tools() {
        let tools = ServerCapabilitiesTools {
            list_changed: Some(true),
        };
        test_serde(&tools);
    }

    #[test]
    fn test_server_notification() {
        let notif = ServerNotification::CancelledNotification(CancelledNotification::new(CancelledNotificationParams {
            reason: Some("because".to_string()),
            request_id: RequestId::Integer(15),
        }));
        test_serde(&notif);
    }

    #[test]
    fn test_server_request() {
        let req = ServerRequest::PingRequest(PingRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(97),
            None,
        ));
        test_serde(&req);
    }

    #[test]
    fn test_server_result() {
        let result = ServerResult::InitializeResult(InitializeResult {
            capabilities: ServerCapabilities::default(),
            meta: None,
            protocol_version: "2025-06-18".to_string(),
            instructions: None,
            server_info: Implementation {
                name: "name".to_string(),
                #[cfg(any(feature = "draft", feature = "2025_06_18"))]
                title: Some("title".to_string()),
                version: "version".to_string(),
                #[cfg(feature = "draft")]
                icons: vec![],
                #[cfg(feature = "draft")]
                website_url: Some("https://github.com/rust-mcp-stack/rust-mcp-sdk".to_string()),
            },
        });
        test_serde(&result);
    }

    #[test]
    fn test_set_level_request() {
        let params = SetLevelRequestParams {
            level: LoggingLevel::Info,
        };

        let req = SetLevelRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(23),
            params,
        );
        test_serde(&req);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_string_schema() {
        let schema = StringSchema::new(
            #[cfg(feature = "draft")]
            Some("default".to_string()),
            Some("description".to_string()),
            None,
            Some(21),
            Some(15),
            Some("title".to_string()),
        );

        test_serde(&schema);
    }

    #[test]
    fn test_subscribe_request() {
        let params = SubscribeRequestParams { uri: "test".to_string() };

        let req = SubscribeRequest::new(
            #[cfg(feature = "draft")]
            RequestId::Integer(22),
            params,
        );
        test_serde(&req);
    }

    #[test]
    fn test_text_content() {
        let content = TextContent::new(
            "test".to_string(),
            None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            None,
        );
        test_serde(&content);
    }

    #[test]
    fn test_text_resource_contents() {
        let content = TextResourceContents {
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            meta: None,
            mime_type: None,
            text: "test".to_string(),
            uri: "test".to_string(),
        };
        test_serde(&content);
    }

    #[test]
    fn test_tool() {
        let input_schema = ToolInputSchema::new(vec!["hey".to_string()], None);
        let tool = Tool {
            #[cfg(not(feature = "2024_11_05"))]
            annotations: None,
            description: None,
            input_schema,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            meta: None,
            name: "test".to_string(),
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            output_schema: None,
            #[cfg(any(feature = "draft", feature = "2025_06_18"))]
            title: None,
            #[cfg(feature = "draft")]
            icons: vec![],
        };
        test_serde(&tool);
    }

    #[cfg(not(feature = "2024_11_05"))]
    #[test]
    fn test_tool_annotations() {
        let ann = ToolAnnotations::default();
        test_serde(&ann);
    }

    #[test]
    fn test_tool_input_schema() {
        let schema = ToolInputSchema::new(vec!["hey".to_string()], None);
        test_serde(&schema);
    }

    #[test]
    fn test_tool_list_changed_notification() {
        let notif = ToolListChangedNotification::new(None);
        test_serde(&notif);
    }

    #[cfg(any(feature = "draft", feature = "2025_06_18"))]
    #[test]
    fn test_tool_output_schema() {
        let schema = ToolOutputSchema::new(vec!["hey".to_string()], None);
        test_serde(&schema);
    }

    #[cfg(not(feature = "draft"))]
    #[test]
    fn test_unsubscribe_request() {
        let params = UnsubscribeRequestParams { uri: "test".to_string() };

        let req = UnsubscribeRequest::new(params);
        test_serde(&req);
    }

    #[test]
    fn test_rpc_error() {
        let error = RpcError {
            code: -32600,
            data: None,
            message: "test".to_string(),
        };
        test_serde(&error);
    }

    #[test]
    fn test_client_request_deserialization() {
        let json = json!({
            "method": "tools/call",
            "params": {
                "name": "add",
                "arguments": {}
            }
        });

        let req: ClientRequest = serde_json::from_value(json).unwrap();
        if let ClientRequest::CallToolRequest(req) = req {
            assert_eq!(req.method(), "tools/call");
            assert_eq!(req.params.name, "add");
        } else {
            panic!("Unexpected variant");
        }
    }

    #[test]
    fn test_server_request_deserialization() {
        let json = json!({
            "method": "sampling/createMessage",
            "params": {
                "maxTokens": 100,
                "messages": []
            }
        });

        let req: ServerRequest = serde_json::from_value(json).unwrap();
        if let ServerRequest::CreateMessageRequest(req) = req {
            assert_eq!(req.method(), "sampling/createMessage");
        } else {
            panic!("Unexpected variant");
        }
    }

    #[test]
    fn test_client_notification_deserialization() {
        let json = json!({
            "method": "notifications/cancelled",
            "params": {
                "requestId": 1,
                "reason": "test"
            }
        });

        let notif: ClientNotification = serde_json::from_value(json).unwrap();
        if let ClientNotification::CancelledNotification(req) = notif {
            assert_eq!(req.method(), "notifications/cancelled");
        } else {
            panic!("Unexpected variant");
        }
    }
}
