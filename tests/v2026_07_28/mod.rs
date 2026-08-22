//! Runtime spot-checks for the 2026-07-28 schema_utils:
//! - `RequestMetaObject` field renames keep wire names on (de)serialization,
//! - `ServerResult` custom `Deserialize` routes `input_required` vs concrete results,
//! - InputRequired retry stamping (`with_input_responses` / `with_request_state`),
//! - `_meta` stamping via `RequestFromClient::with_meta`.
use rust_mcp_schema::schema_utils::*;
use rust_mcp_schema::*;

fn sample_meta() -> RequestMetaObject {
    RequestMetaObject::new("2026-07-28", ClientCapabilities::default()).with_client_info(Implementation {
        name: "spot-check".to_string(),
        version: "0.1.0".to_string(),
        ..Default::default()
    })
}

#[test]
fn request_meta_object_wire_names_roundtrip() {
    let meta = sample_meta();
    let json = serde_json::to_value(&meta).unwrap();

    // Wire names preserved (extension keys), not the Rust field names.
    assert_eq!(json["io.modelcontextprotocol/protocolVersion"], "2026-07-28");
    assert!(json.get("io.modelcontextprotocol/clientCapabilities").is_some());
    assert!(json.get("io.modelcontextprotocol/clientInfo").is_some());
    assert!(json.get("protocol_version").is_none());

    let back: RequestMetaObject = serde_json::from_value(json).unwrap();
    assert_eq!(back.protocol_version, "2026-07-28");
    assert_eq!(back.client_info.unwrap().name, "spot-check");
}

#[test]
fn request_meta_object_preserves_trace_context_and_vendor_keys() {
    // SEP-414: `traceparent`/`tracestate`/`baggage` (and vendor-prefixed keys) are reserved
    // `_meta` keys carried through the `extra` catch-all map. They must survive a
    // serialize/deserialize round-trip losslessly.
    let meta = sample_meta()
        .with_traceparent("00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01")
        .with_tracestate("rojo=00f067aa0ba902b7")
        .with_baggage("userId=alice,serverNode=DF:28,isProduction=false");

    // Accessors read back the typed values.
    assert_eq!(
        meta.traceparent(),
        Some("00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01")
    );
    assert_eq!(meta.tracestate(), Some("rojo=00f067aa0ba902b7"));
    assert_eq!(meta.baggage(), Some("userId=alice,serverNode=DF:28,isProduction=false"));

    // Serialize: the reserved keys must appear verbatim at the `_meta` top level.
    let json = serde_json::to_value(&meta).unwrap();
    assert_eq!(json["traceparent"], "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01");
    assert_eq!(json["tracestate"], "rojo=00f067aa0ba902b7");
    assert_eq!(json["baggage"], "userId=alice,serverNode=DF:28,isProduction=false");

    // Round-trip preserves everything (including a vendor-prefixed key).
    let with_vendor = {
        let mut v = serde_json::to_value(&meta).unwrap();
        v.as_object_mut()
            .unwrap()
            .insert("com.example.traceThing".to_string(), serde_json::json!({ "x": 1 }));
        v
    };
    let back: RequestMetaObject = serde_json::from_value(with_vendor).unwrap();
    assert_eq!(
        back.traceparent(),
        Some("00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01")
    );
    assert_eq!(back.tracestate(), Some("rojo=00f067aa0ba902b7"));
    assert_eq!(back.baggage(), Some("userId=alice,serverNode=DF:28,isProduction=false"));
    let extra = back.extra.expect("vendor key kept in extra");
    assert_eq!(extra.get("com.example.traceThing"), Some(&serde_json::json!({ "x": 1 })));
}

#[test]
fn request_meta_object_extra_is_none_when_no_trace_context() {
    // Note on serde `flatten` semantics: when no unmatched keys are present, deserialization
    // materializes `extra` as `Some({})` rather than `None`. This is benign — an empty map
    // flattens to nothing on re-serialize, so the wire behavior is identical. The meaningful
    // contract is that the *accessors* report absence (`None`) when the keys are missing.
    let meta = sample_meta();
    assert!(meta.extra.is_none());
    assert_eq!(meta.traceparent(), None);
    assert_eq!(meta.tracestate(), None);
    assert_eq!(meta.baggage(), None);

    let back: RequestMetaObject = serde_json::from_value(serde_json::to_value(&meta).unwrap()).unwrap();
    // Accessors must report absence even when `extra` is `Some({})`.
    assert_eq!(back.traceparent(), None);
    assert_eq!(back.tracestate(), None);
    assert_eq!(back.baggage(), None);

    // And re-serialization must not introduce any extra keys.
    let json = serde_json::to_value(&back).unwrap();
    assert!(json.get("traceparent").is_none());
    assert!(json.get("tracestate").is_none());
    assert!(json.get("baggage").is_none());
}

#[test]
fn server_result_routes_input_required() {
    let payload = serde_json::json!({
        "resultType": "input_required",
        "requestState": "opaque-123",
        "inputRequests": {
            "elicit-1": {
                "jsonrpc": "2.0",
                "id": 7,
                "method": "elicitation/create",
                "params": {
                    "mode": "form",
                    "message": "Provide your name",
                    "_meta": sample_meta(),
                    "requestedSchema": { "type": "object", "properties": {} }
                }
            }
        }
    });

    let result: ServerResult = serde_json::from_value(payload).unwrap();
    assert!(result.is_input_required());
    assert!(!result.is_complete());

    let input_required = result.as_input_required().expect("must be input_required");
    assert_eq!(input_required.request_state.as_deref(), Some("opaque-123"));

    let requests = input_required.input_requests.as_ref().expect("inputRequests present");
    assert_eq!(requests.len(), 1);
    let (key, input_request) = requests.iter().next().unwrap();
    assert_eq!(key, "elicit-1");
    assert_eq!(input_request.method(), "elicitation/create");
    assert!(input_request.as_elicit().is_some());
    assert!(input_request.as_create_message().is_none());
}

#[test]
fn server_result_complete_keeps_concrete_data() {
    // Regression guard for the untagged-greedy-match bug: a complete CallToolResult must
    // deserialize as `CallToolResult` with its `content` intact.
    let payload = serde_json::json!({
        "resultType": "complete",
        "content": [{ "type": "text", "text": "hello", "annotations": null, "_meta": null }]
    });

    let result: ServerResult = serde_json::from_value(payload).unwrap();
    assert!(result.is_complete());
    match result {
        ServerResult::CallToolResult(r) => {
            assert_eq!(r.content.len(), 1);
            assert_eq!(r.result_type, "complete");
        }
        other => panic!("expected CallToolResult, got {:?}", other),
    }
}

#[test]
fn server_result_absent_result_type_defaults_to_complete() {
    // Spec backward-compatibility rule: "when a client receives a result from a server
    // implementing an earlier protocol version (which does not include resultType), the
    // client MUST treat the absent field as 'complete'." Concrete result structs declare
    // `result_type` as a required field, so the `ServerResult` deserializer must materialize
    // the default before variant matching — otherwise the payload matches nothing and the
    // message is silently dropped downstream.
    let payload = serde_json::json!({
        "content": [{ "type": "text", "text": "legacy", "annotations": null, "_meta": null }]
    });

    let result: ServerResult = serde_json::from_value(payload).unwrap();
    assert!(result.is_complete());
    match result {
        ServerResult::CallToolResult(r) => {
            assert_eq!(r.content.len(), 1);
            assert_eq!(r.result_type, "complete");
        }
        other => panic!("expected CallToolResult, got {:?}", other),
    }

    // The generic `Result` catch-all must also tolerate an absent `resultType`.
    let generic: ServerResult = serde_json::from_value(serde_json::json!({ "custom": 1 })).unwrap();
    match generic {
        ServerResult::Result(r) => assert_eq!(r.result_type, "complete"),
        other => panic!("expected generic Result, got {:?}", other),
    }
}

#[test]
fn retry_stamping_on_request_from_client() {
    let meta = sample_meta();
    let params = CallToolRequestParams::new("my_tool", meta);
    let request = RequestFromClient::CallToolRequest(params);

    let responses = InputResponses::new().insert(
        "elicit-1",
        ElicitResult {
            action: ElicitResultAction::Accept,
            content: None,
        },
    );

    let retried = request
        .with_input_responses(responses, Some("opaque-123".to_string()))
        .expect("CallToolRequest accepts input responses");

    match retried {
        RequestFromClient::CallToolRequest(params) => {
            assert!(params.input_responses.is_some());
            assert_eq!(params.request_state.as_deref(), Some("opaque-123"));
        }
        other => panic!("expected CallToolRequest, got {:?}", other),
    }

    // Unsupported variant must error, not silently pass through.
    let list_request = RequestFromClient::ListToolsRequest(PaginatedRequestParams::default());
    assert!(list_request.with_input_responses(InputResponses::new(), None).is_err());
}

#[test]
fn with_meta_stamps_all_standard_requests() {
    let meta = sample_meta();
    let request = RequestFromClient::ListToolsRequest(PaginatedRequestParams::default()).with_meta(meta);

    let stamped = request.meta().expect("standard request carries meta");
    assert_eq!(stamped.protocol_version, "2026-07-28");

    // Wire check: `_meta` serializes with extension keys inside request params.
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["_meta"]["io.modelcontextprotocol/protocolVersion"], "2026-07-28");
}

#[test]
fn cancelled_notification_params_default_compiles() {
    // Guard for the `RequestId: Default` fix (required `requestId` + derived `Default`).
    let params = CancelledNotificationParams::default();
    assert!(matches!(params.request_id, RequestId::Integer(0)));
}

#[test]
fn client_notification_deserializes_with_validators() {
    // 2026-07-28: `ClientNotification` is a plain struct (not a union) and must derive
    // `Deserialize`; its const `jsonrpc`/`method` fields are enforced by the generated
    // validators (which would otherwise be dead code).
    let good = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/cancelled",
        "params": { "requestId": 42 }
    });
    let notification: ClientNotification = serde_json::from_value(good).unwrap();
    assert_eq!(notification.method(), "notifications/cancelled");
    assert!(matches!(notification.params.request_id, RequestId::Integer(42)));

    let bad_method = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/progress",
        "params": { "requestId": 42 }
    });
    assert!(serde_json::from_value::<ClientNotification>(bad_method).is_err());
}

#[test]
fn sdk_error_codes_match_schema_constants() {
    // Regression guard: SdkErrorCodes previously carried the pre-RC codes -32003/-32004
    // while the 2026-07-28 RC schema assigns -32021/-32022 (see spec changelog:
    // `-32003` → `-32021`, `-32004` → `-32022`).
    assert_eq!(
        i64::from(SdkErrorCodes::MISSING_REQUIRED_CLIENT_CAPABILITY),
        MISSING_REQUIRED_CLIENT_CAPABILITY
    );
    assert_eq!(
        i64::from(SdkErrorCodes::UNSUPPORTED_PROTOCOL_VERSION),
        UNSUPPORTED_PROTOCOL_VERSION
    );

    // JSON-RPC standard codes must stay aligned too.
    assert_eq!(i64::from(SdkErrorCodes::PARSE_ERROR), PARSE_ERROR);
    assert_eq!(i64::from(SdkErrorCodes::INVALID_REQUEST), INVALID_REQUEST);
    assert_eq!(i64::from(SdkErrorCodes::METHOD_NOT_FOUND), METHOD_NOT_FOUND);
    assert_eq!(i64::from(SdkErrorCodes::INVALID_PARAMS), INVALID_PARAMS);
    assert_eq!(i64::from(SdkErrorCodes::INTERNAL_ERROR), INTERNAL_ERROR);
}

#[test]
fn rpc_error_codes_match_schema_constants() {
    assert_eq!(
        i64::from(RpcErrorCodes::MISSING_REQUIRED_CLIENT_CAPABILITY),
        MISSING_REQUIRED_CLIENT_CAPABILITY
    );
    assert_eq!(
        i64::from(RpcErrorCodes::UNSUPPORTED_PROTOCOL_VERSION),
        UNSUPPORTED_PROTOCOL_VERSION
    );
    assert_eq!(i64::from(RpcErrorCodes::HEADER_MISMATCH), HEADER_MISMATCH);
    assert_eq!(i64::from(RpcErrorCodes::PARSE_ERROR), PARSE_ERROR);
    assert_eq!(i64::from(RpcErrorCodes::INVALID_REQUEST), INVALID_REQUEST);
    assert_eq!(i64::from(RpcErrorCodes::METHOD_NOT_FOUND), METHOD_NOT_FOUND);
    assert_eq!(i64::from(RpcErrorCodes::INVALID_PARAMS), INVALID_PARAMS);
    assert_eq!(i64::from(RpcErrorCodes::INTERNAL_ERROR), INTERNAL_ERROR);
}

#[test]
fn sdk_error_uses_schema_aligned_wire_code() {
    // An SdkError built for a missing-capability failure must carry the RC wire code,
    // not the stale pre-RC -32003.
    let error = SdkError::new(
        SdkErrorCodes::MISSING_REQUIRED_CLIENT_CAPABILITY,
        "missing capability".to_string(),
        None,
    );
    assert_eq!(error.code, -32021);
    assert_eq!(error.code, MISSING_REQUIRED_CLIENT_CAPABILITY);

    let error = SdkError::new(SdkErrorCodes::UNSUPPORTED_PROTOCOL_VERSION, "unsupported".to_string(), None);
    assert_eq!(error.code, -32022);
    assert_eq!(error.code, UNSUPPORTED_PROTOCOL_VERSION);
}

#[test]
fn result_from_client_from_impls_cover_all_variants() {
    // Regression guard: the 2026 schema's `ClientResult` is only `$ref: Result`, which once
    // caused the generator to skip `From` impls for the `InputResponse` variants of
    // `ResultFromClient` (CreateMessageResult / ListRootsResult / ElicitResult).
    let elicit = ElicitResult {
        action: ElicitResultAction::Accept,
        content: None,
    };
    let from_elicit = ResultFromClient::from(elicit);
    assert!(matches!(from_elicit, ResultFromClient::ElicitResult(_)));

    let roots = ListRootsResult { roots: vec![] };
    let from_roots = ResultFromClient::from(roots);
    assert!(matches!(from_roots, ResultFromClient::ListRootsResult(_)));

    // Message-level conversion must wrap into `MessageFromClient::ResultFromClient`.
    let message = MessageFromClient::from(ListRootsResult { roots: vec![] });
    assert!(matches!(message, MessageFromClient::ResultFromClient(_)));
}

#[test]
fn try_from_result_from_client_extracts_expected_variant() {
    let ok_value = ResultFromClient::ElicitResult(ElicitResult {
        action: ElicitResultAction::Decline,
        content: None,
    });
    let elicit = ElicitResult::try_from(ok_value).expect("matching variant must convert");
    assert!(matches!(elicit.action, ElicitResultAction::Decline));

    let wrong_variant = ResultFromClient::ListRootsResult(ListRootsResult { roots: vec![] });
    assert!(ElicitResult::try_from(wrong_variant).is_err());

    let generic = ResultFromClient::Result(Result {
        meta: None,
        result_type: "complete".to_string(),
        extra: None,
    });
    let generic = GenericResult::try_from(generic).expect("Result variant must convert");
    assert_eq!(generic.result_type, "complete");
}
