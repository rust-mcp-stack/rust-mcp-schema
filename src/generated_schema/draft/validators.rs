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

/// Macro to generate a field-specific validator function for use with Serde.
///
/// This avoids repetitive boilerplate when you have multiple fields/structs
/// requiring constant string validation.
///
/// # Syntax
/// ```ignore
/// validate!(fn_name, "StructName", "field_name", "expected_value");
/// ```
///
/// - `fn_name`: The function name to generate.
/// - `StructName`: The name of the struct (for error messages).
/// - `field_name`: The name of the field (for error messages).
/// - `expected_value`: The required constant string value.
///
macro_rules! validate {
    ($func_name:ident,  $struct:expr, $field:expr, $expected:expr $(,)?) => {
        pub(crate) fn $func_name<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            const_str_validator($struct, $field, $expected, deserializer)
        }
    };
}

//* Validator Functions *//
validate!(audio_content_type_, "AudioContent", "type_", "audio");
validate!(boolean_schema_type_, "BooleanSchema", "type_", "boolean");
validate!(call_tool_request_jsonrpc, "CallToolRequest", "jsonrpc", "2.0");
validate!(call_tool_request_method, "CallToolRequest", "method", "tools/call");
validate!(cancelled_notification_jsonrpc, "CancelledNotification", "jsonrpc", "2.0");
validate!(
    cancelled_notification_method,
    "CancelledNotification",
    "method",
    "notifications/cancelled"
);
validate!(complete_request_jsonrpc, "CompleteRequest", "jsonrpc", "2.0");
validate!(complete_request_method, "CompleteRequest", "method", "completion/complete");
validate!(create_message_request_jsonrpc, "CreateMessageRequest", "jsonrpc", "2.0");
validate!(
    create_message_request_method,
    "CreateMessageRequest",
    "method",
    "sampling/createMessage"
);
validate!(elicit_request_jsonrpc, "ElicitRequest", "jsonrpc", "2.0");
validate!(elicit_request_method, "ElicitRequest", "method", "elicitation/create");
validate!(elicit_requested_schema_type_, "ElicitRequestedSchema", "type_", "object");
validate!(embedded_resource_type_, "EmbeddedResource", "type_", "resource");
validate!(enum_schema_type_, "EnumSchema", "type_", "string");
validate!(get_prompt_request_jsonrpc, "GetPromptRequest", "jsonrpc", "2.0");
validate!(get_prompt_request_method, "GetPromptRequest", "method", "prompts/get");
validate!(image_content_type_, "ImageContent", "type_", "image");
validate!(initialize_request_jsonrpc, "InitializeRequest", "jsonrpc", "2.0");
validate!(initialize_request_method, "InitializeRequest", "method", "initialize");
validate!(initialized_notification_jsonrpc, "InitializedNotification", "jsonrpc", "2.0");
validate!(
    initialized_notification_method,
    "InitializedNotification",
    "method",
    "notifications/initialized"
);
validate!(jsonrpc_error_jsonrpc, "JsonrpcError", "jsonrpc", "2.0");
validate!(jsonrpc_notification_jsonrpc, "JsonrpcNotification", "jsonrpc", "2.0");
validate!(jsonrpc_request_jsonrpc, "JsonrpcRequest", "jsonrpc", "2.0");
validate!(jsonrpc_response_jsonrpc, "JsonrpcResponse", "jsonrpc", "2.0");
validate!(list_prompts_request_jsonrpc, "ListPromptsRequest", "jsonrpc", "2.0");
validate!(list_prompts_request_method, "ListPromptsRequest", "method", "prompts/list");
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
validate!(list_resources_request_jsonrpc, "ListResourcesRequest", "jsonrpc", "2.0");
validate!(
    list_resources_request_method,
    "ListResourcesRequest",
    "method",
    "resources/list"
);
validate!(list_roots_request_jsonrpc, "ListRootsRequest", "jsonrpc", "2.0");
validate!(list_roots_request_method, "ListRootsRequest", "method", "roots/list");
validate!(list_tools_request_jsonrpc, "ListToolsRequest", "jsonrpc", "2.0");
validate!(list_tools_request_method, "ListToolsRequest", "method", "tools/list");
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
validate!(paginated_request_jsonrpc, "PaginatedRequest", "jsonrpc", "2.0");
validate!(ping_request_jsonrpc, "PingRequest", "jsonrpc", "2.0");
validate!(ping_request_method, "PingRequest", "method", "ping");
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
validate!(
    roots_list_changed_notification_jsonrpc,
    "RootsListChangedNotification",
    "jsonrpc",
    "2.0"
);
validate!(
    roots_list_changed_notification_method,
    "RootsListChangedNotification",
    "method",
    "notifications/roots/list_changed"
);
validate!(set_level_request_jsonrpc, "SetLevelRequest", "jsonrpc", "2.0");
validate!(set_level_request_method, "SetLevelRequest", "method", "logging/setLevel");
validate!(string_schema_type_, "StringSchema", "type_", "string");
validate!(subscribe_request_jsonrpc, "SubscribeRequest", "jsonrpc", "2.0");
validate!(subscribe_request_method, "SubscribeRequest", "method", "resources/subscribe");
validate!(text_content_type_, "TextContent", "type_", "text");
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
validate!(tool_output_schema_type_, "ToolOutputSchema", "type_", "object");
validate!(unsubscribe_request_jsonrpc, "UnsubscribeRequest", "jsonrpc", "2.0");
validate!(
    unsubscribe_request_method,
    "UnsubscribeRequest",
    "method",
    "resources/unsubscribe"
);
