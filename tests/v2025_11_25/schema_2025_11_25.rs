use super::common::round_trip_test;
use rust_mcp_schema::*;

#[test]
fn test_annotations() {
    let ann = Annotations {
        audience: vec![Role::User, Role::Assistant],
        last_modified: Some("2025-01-12T15:00:58Z".to_string()),
        priority: Some(0.0),
    };
    round_trip_test(&ann);

    round_trip_test(&Annotations::default());
}

#[test]
fn test_tool_execution_task_support_enum() {
    // The enum is usually a simple string, but we test all variants
    let variants = [
        ToolExecutionTaskSupport::Forbidden,
        ToolExecutionTaskSupport::Optional,
        ToolExecutionTaskSupport::Required,
    ];
    for v in variants {
        round_trip_test(&v);
    }
}

#[test]
fn test_base_metadata() {
    let meta = BaseMetadata {
        name: "my-tool".into(),
        title: Some("My Awesome Tool".into()),
    };
    round_trip_test(&meta);

    round_trip_test(&BaseMetadata {
        name: "only-name".into(),
        title: None,
    });
}

#[test]
fn test_audio_content() {
    let audio = AudioContent::new(
        "data:audio/wav;base64,UklGRiQAAABXQVZFZm10IBAAAAABAAEARKwAAIhYAQACABAAZGF0YQAAAA==".into(),
        "audio/wav".into(),
        Some(Annotations::default()),
        None,
    );
    round_trip_test(&audio);
}

#[test]
fn test_call_tool_request() {
    let req = CallToolRequest::new(
        RequestId::String("req-123".into()),
        CallToolRequestParams {
            name: "calculator".into(),
            arguments: Some(serde_json::json!({ "a": 1, "b": 2 }).as_object().unwrap().clone()),
            meta: Some(CallToolMeta::default()),
            task: Some(TaskMetadata::default()),
        },
    );
    round_trip_test(&req);

    // Minimal version (no optionals)
    let minimal = CallToolRequest::new(
        RequestId::Integer(42),
        CallToolRequestParams {
            name: "hello".into(),
            arguments: None,
            meta: None,
            task: None,
        },
    );
    round_trip_test(&minimal);
}

#[test]
fn test_call_tool_result() {
    let result = CallToolResult {
        content: vec![ContentBlock::TextContent(TextContent::new("42".into(), None, None))],
        is_error: Some(true),
        meta: None,
        structured_content: Some(serde_json::json!({ "answer": 42 }).as_object().unwrap().clone()),
    };
    round_trip_test(&result);

    // Success case
    let success = CallToolResult {
        content: vec![],
        is_error: None,
        meta: None,
        structured_content: None,
    };
    round_trip_test(&success);
}

#[test]
fn test_tool_use_and_result_content() {
    let tool_use = ToolUseContent::new(
        "use-1".into(),
        serde_json::json!({ "query": "weather" }).as_object().unwrap().clone(),
        "weather_tool".into(),
        None,
    );
    round_trip_test(&tool_use);

    let tool_result = ToolResultContent::new(
        vec![ContentBlock::TextContent(TextContent::new("Sunny".into(), None, None))],
        "use-1".into(),
        Some(false),
        None,
        None,
    );
    round_trip_test(&tool_result);
}

#[test]
fn test_client_capabilities_roundtrip() {
    let caps = ClientCapabilities {
        elicitation: Some(ClientElicitation::default()),
        experimental: None,
        roots: Some(ClientRoots {
            list_changed: Some(true),
        }),
        sampling: Some(ClientSampling {
            context: Some(serde_json::json!({ "enabled": true }).as_object().unwrap().clone()),
            tools: None,
        }),
        tasks: Some(ClientTasks {
            cancel: Some(serde_json::json!({}).as_object().unwrap().clone()),
            list: None,
            requests: Some(ClientTaskRequest {
                elicitation: Some(ClientTaskElicitation {
                    create: Some(serde_json::json!({}).as_object().unwrap().clone()),
                }),
                sampling: None,
            }),
        }),
    };
    round_trip_test(&caps);

    round_trip_test(&ClientCapabilities::default());
}

#[test]
fn test_untagged_enums() {
    // These are the big ones that often break if serde attributes are wrong
    let notifications: Vec<ClientNotification> = vec![
        ClientNotification::CancelledNotification(CancelledNotification::new(CancelledNotificationParams::default())),
        ClientNotification::InitializedNotification(InitializedNotification::new(None)),
        ClientNotification::ProgressNotification(ProgressNotification::new(ProgressNotificationParams {
            message: Some("message".into()),
            meta: None,
            progress: 53.,
            progress_token: ProgressToken::String("a-b-c".to_string()),
            total: Some(100.),
        })),
    ];

    for n in notifications {
        round_trip_test(&n);
    }

    let requests: Vec<ClientRequest> = vec![
        ClientRequest::PingRequest(PingRequest::new(RequestId::Integer(1), None)),
        ClientRequest::CallToolRequest(CallToolRequest::new(
            RequestId::String("abc".into()),
            CallToolRequestParams {
                name: "test".into(),
                arguments: None,
                meta: None,
                task: None,
            },
        )),
    ];

    for r in requests {
        round_trip_test(&r);
    }
}

#[test]
fn test_error_responses() {
    let err = UrlElicitationRequiredError::new(
        UrlElicitError::new(
            UrlElicitErrorData {
                elicitations: vec![],
                extra: None,
            },
            "Need more info".into(),
        ),
        Some(RequestId::String("req-1".into())),
    );
    round_trip_test(&err);
}

#[test]
fn test_blob_resource_contents() {
    let blob = BlobResourceContents {
        blob: "aGVsbG8gd29ybGQ=".into(),
        meta: None,
        mime_type: Some("text/plain".into()),
        uri: "file:///tmp/hello.txt".into(),
    };
    round_trip_test(&blob);
}

#[test]
fn test_boolean_schema_and_tool_output_schema() {
    let bool_schema = BooleanSchema::new(Some(true), None, Some("Enable magic".into()));
    round_trip_test(&bool_schema);

    let output_schema = ToolOutputSchema::new(
        vec!["result".into()],
        None,
        Some("https://json-schema.org/draft/2020-12/schema".into()),
    );
    round_trip_test(&output_schema);
}
