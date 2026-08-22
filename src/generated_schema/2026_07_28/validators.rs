/// Validates that a deserialized string field matches a given constant value.
///
/// This function is intended for use with `#[serde(deserialize_with)]` to enforce
/// that a field in a struct always has a fixed, expected string value during deserialization.
///
/// # Parameters
/// - `struct_name`: The name of the struct where this validation is applied.
/// - `field_name`: The name of the field being validated.
/// - `expected`: The expected constant string value for the field.
/// - `deserializer`: The Serde deserializer for the field.
///
/// # Returns
/// - `Ok(String)` if the deserialized value matches the expected value.
/// - `Err(D::Error)` if the value differs, with an error message indicating
///   which struct and field failed validation.
///
pub fn const_str_validator<'de, D>(
    struct_name: &'static str,
    field_name: &'static str,
    expected: &'static str,
    deserializer: D,
) -> Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let value: String = serde::Deserialize::deserialize(deserializer)?;
    if value == expected {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(format!(
            "Expected field `{field_name}` in struct `{struct_name}` as const value '{expected}', but got '{value}'",
        )))
    }
}

/// Validator for `Option<String>` fields:
/// - None      → accepted
/// - Some(s)   → s must exactly match `expected`
pub fn const_str_option_validator<'de, D>(
    struct_name: &'static str,
    field_name: &'static str,
    expected: &'static str,
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let opt: Option<String> = serde::Deserialize::deserialize(deserializer)?;
    match opt {
        Some(ref value) if value != expected => {
            Err(serde::de::Error::custom(format!(
                "Expected field `{field_name}` in struct `{struct_name}` to be None or exactly \"{expected}\", but got Some(\"{value}\")",
            )))
        }
        Some(value) => Ok(Some(value)), // value == expected
        None => Ok(None),
    }
}

// Used only by the `i64` arm of `validate!`; some schema versions (e.g. 2026-07-28) have no
// integer const properties, so it would otherwise be flagged as dead code there.
#[allow(dead_code)]
fn i64_validator<'de, D>(
    struct_name: &'static str,
    field_name: &'static str,
    expected: i64,
    deserializer: D,
) -> Result<i64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let value = serde::Deserialize::deserialize(deserializer)?;
    if value == expected {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(format!(
            "Invalid {struct_name}::{field_name}: expected {expected}, got {value}"
        )))
    }
}

macro_rules! validate {
    // === String validation (required) ===
    ($func_name:ident,  $struct:expr, $field:expr, $expected:expr $(,)?) => {
        pub(crate) fn $func_name<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            const_str_validator($struct, $field, $expected, deserializer)
        }
    };

    // Optional String case (with trailing `, option`)
    ($func_name:ident, $struct:expr, $field:expr, $expected:expr, option $(,)?) => {
        pub(crate) fn $func_name<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            const_str_option_validator($struct, $field, $expected, deserializer)
        }
    };

    // === i64 validation (required) ===
    ($func_name:ident, $struct:expr, $field:expr, $expected:expr, i64 $(,)?) => {
        pub(crate) fn $func_name<'de, D>(deserializer: D) -> Result<i64, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            i64_validator($struct, $field, $expected, deserializer)
        }
    };
}

//* Validator Functions *//
validate!(audio_content_type_, "AudioContent", "type_", "audio");
validate!(boolean_schema_type_, "BooleanSchema", "type_", "boolean");
validate!(call_tool_request_jsonrpc, "CallToolRequest", "jsonrpc", "2.0");
validate!(call_tool_request_method, "CallToolRequest", "method", "tools/call");
validate!(call_tool_result_response_jsonrpc, "CallToolResultResponse", "jsonrpc", "2.0");
validate!(cancelled_notification_jsonrpc, "CancelledNotification", "jsonrpc", "2.0");
validate!(
    cancelled_notification_method,
    "CancelledNotification",
    "method",
    "notifications/cancelled"
);
validate!(client_notification_jsonrpc, "ClientNotification", "jsonrpc", "2.0");
validate!(
    client_notification_method,
    "ClientNotification",
    "method",
    "notifications/cancelled"
);
validate!(complete_request_jsonrpc, "CompleteRequest", "jsonrpc", "2.0");
validate!(complete_request_method, "CompleteRequest", "method", "completion/complete");
validate!(complete_result_response_jsonrpc, "CompleteResultResponse", "jsonrpc", "2.0");
validate!(
    create_message_request_method,
    "CreateMessageRequest",
    "method",
    "sampling/createMessage"
);
validate!(discover_request_jsonrpc, "DiscoverRequest", "jsonrpc", "2.0");
validate!(discover_request_method, "DiscoverRequest", "method", "server/discover");
validate!(discover_result_response_jsonrpc, "DiscoverResultResponse", "jsonrpc", "2.0");
validate!(elicit_request_method, "ElicitRequest", "method", "elicitation/create");
validate!(
    elicit_request_form_params_mode,
    "ElicitRequestFormParams",
    "mode",
    "form",
    option
);
validate!(
    elicit_request_form_params_requested_schema_type_,
    "ElicitRequestFormParamsRequestedSchema",
    "type_",
    "object"
);
validate!(elicit_request_url_params_mode, "ElicitRequestUrlParams", "mode", "url");
validate!(embedded_resource_type_, "EmbeddedResource", "type_", "resource");
validate!(get_prompt_request_jsonrpc, "GetPromptRequest", "jsonrpc", "2.0");
validate!(get_prompt_request_method, "GetPromptRequest", "method", "prompts/get");
validate!(
    get_prompt_result_response_jsonrpc,
    "GetPromptResultResponse",
    "jsonrpc",
    "2.0"
);
validate!(header_mismatch_error_jsonrpc, "HeaderMismatchError", "jsonrpc", "2.0");
validate!(image_content_type_, "ImageContent", "type_", "image");
validate!(jsonrpc_error_response_jsonrpc, "JsonrpcErrorResponse", "jsonrpc", "2.0");
validate!(jsonrpc_notification_jsonrpc, "JsonrpcNotification", "jsonrpc", "2.0");
validate!(jsonrpc_request_jsonrpc, "JsonrpcRequest", "jsonrpc", "2.0");
validate!(jsonrpc_result_response_jsonrpc, "JsonrpcResultResponse", "jsonrpc", "2.0");
validate!(legacy_titled_enum_schema_type_, "LegacyTitledEnumSchema", "type_", "string");
validate!(list_prompts_request_jsonrpc, "ListPromptsRequest", "jsonrpc", "2.0");
validate!(list_prompts_request_method, "ListPromptsRequest", "method", "prompts/list");
validate!(
    list_prompts_result_response_jsonrpc,
    "ListPromptsResultResponse",
    "jsonrpc",
    "2.0"
);
validate!(
    list_resource_templates_request_jsonrpc,
    "ListResourceTemplatesRequest",
    "jsonrpc",
    "2.0"
);
validate!(
    list_resource_templates_request_method,
    "ListResourceTemplatesRequest",
    "method",
    "resources/templates/list"
);
validate!(
    list_resource_templates_result_response_jsonrpc,
    "ListResourceTemplatesResultResponse",
    "jsonrpc",
    "2.0"
);
validate!(list_resources_request_jsonrpc, "ListResourcesRequest", "jsonrpc", "2.0");
validate!(
    list_resources_request_method,
    "ListResourcesRequest",
    "method",
    "resources/list"
);
validate!(
    list_resources_result_response_jsonrpc,
    "ListResourcesResultResponse",
    "jsonrpc",
    "2.0"
);
validate!(list_roots_request_method, "ListRootsRequest", "method", "roots/list");
validate!(list_tools_request_jsonrpc, "ListToolsRequest", "jsonrpc", "2.0");
validate!(list_tools_request_method, "ListToolsRequest", "method", "tools/list");
validate!(
    list_tools_result_response_jsonrpc,
    "ListToolsResultResponse",
    "jsonrpc",
    "2.0"
);
validate!(
    logging_message_notification_jsonrpc,
    "LoggingMessageNotification",
    "jsonrpc",
    "2.0"
);
validate!(
    logging_message_notification_method,
    "LoggingMessageNotification",
    "method",
    "notifications/message"
);
validate!(
    missing_required_client_capability_error_jsonrpc,
    "MissingRequiredClientCapabilityError",
    "jsonrpc",
    "2.0"
);
validate!(paginated_request_jsonrpc, "PaginatedRequest", "jsonrpc", "2.0");
validate!(progress_notification_jsonrpc, "ProgressNotification", "jsonrpc", "2.0");
validate!(
    progress_notification_method,
    "ProgressNotification",
    "method",
    "notifications/progress"
);
validate!(
    prompt_list_changed_notification_jsonrpc,
    "PromptListChangedNotification",
    "jsonrpc",
    "2.0"
);
validate!(
    prompt_list_changed_notification_method,
    "PromptListChangedNotification",
    "method",
    "notifications/prompts/list_changed"
);
validate!(prompt_reference_type_, "PromptReference", "type_", "ref/prompt");
validate!(read_resource_request_jsonrpc, "ReadResourceRequest", "jsonrpc", "2.0");
validate!(
    read_resource_request_method,
    "ReadResourceRequest",
    "method",
    "resources/read"
);
validate!(
    read_resource_result_response_jsonrpc,
    "ReadResourceResultResponse",
    "jsonrpc",
    "2.0"
);
validate!(resource_link_type_, "ResourceLink", "type_", "resource_link");
validate!(
    resource_list_changed_notification_jsonrpc,
    "ResourceListChangedNotification",
    "jsonrpc",
    "2.0"
);
validate!(
    resource_list_changed_notification_method,
    "ResourceListChangedNotification",
    "method",
    "notifications/resources/list_changed"
);
validate!(
    resource_template_reference_type_,
    "ResourceTemplateReference",
    "type_",
    "ref/resource"
);
validate!(
    resource_updated_notification_jsonrpc,
    "ResourceUpdatedNotification",
    "jsonrpc",
    "2.0"
);
validate!(
    resource_updated_notification_method,
    "ResourceUpdatedNotification",
    "method",
    "notifications/resources/updated"
);
validate!(string_schema_type_, "StringSchema", "type_", "string");
validate!(
    subscriptions_acknowledged_notification_jsonrpc,
    "SubscriptionsAcknowledgedNotification",
    "jsonrpc",
    "2.0"
);
validate!(
    subscriptions_acknowledged_notification_method,
    "SubscriptionsAcknowledgedNotification",
    "method",
    "notifications/subscriptions/acknowledged"
);
validate!(
    subscriptions_listen_request_jsonrpc,
    "SubscriptionsListenRequest",
    "jsonrpc",
    "2.0"
);
validate!(
    subscriptions_listen_request_method,
    "SubscriptionsListenRequest",
    "method",
    "subscriptions/listen"
);
validate!(
    subscriptions_listen_result_response_jsonrpc,
    "SubscriptionsListenResultResponse",
    "jsonrpc",
    "2.0"
);
validate!(text_content_type_, "TextContent", "type_", "text");
validate!(
    titled_multi_select_enum_schema_type_,
    "TitledMultiSelectEnumSchema",
    "type_",
    "array"
);
validate!(
    titled_single_select_enum_schema_type_,
    "TitledSingleSelectEnumSchema",
    "type_",
    "string"
);
validate!(tool_input_schema_type_, "ToolInputSchema", "type_", "object");
validate!(
    tool_list_changed_notification_jsonrpc,
    "ToolListChangedNotification",
    "jsonrpc",
    "2.0"
);
validate!(
    tool_list_changed_notification_method,
    "ToolListChangedNotification",
    "method",
    "notifications/tools/list_changed"
);
validate!(tool_result_content_type_, "ToolResultContent", "type_", "tool_result");
validate!(tool_use_content_type_, "ToolUseContent", "type_", "tool_use");
validate!(
    unsupported_protocol_version_error_jsonrpc,
    "UnsupportedProtocolVersionError",
    "jsonrpc",
    "2.0"
);
validate!(
    untitled_multi_select_enum_schema_type_,
    "UntitledMultiSelectEnumSchema",
    "type_",
    "array"
);
validate!(
    untitled_multi_select_enum_schema_items_type_,
    "UntitledMultiSelectEnumSchemaItems",
    "type_",
    "string"
);
validate!(
    untitled_single_select_enum_schema_type_,
    "UntitledSingleSelectEnumSchema",
    "type_",
    "string"
);
