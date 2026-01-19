use crate::generated_schema::*;
use serde::ser::SerializeStruct;
use serde_json::{json, Value};
use std::hash::{Hash, Hasher};
use std::result;
use std::{fmt::Display, str::FromStr};

pub const RELATED_TASK_META_KEY: &str = "io.modelcontextprotocol/related-task";

#[derive(Debug, PartialEq)]
pub enum MessageTypes {
    Request,
    Response,
    Notification,
    Error,
}
/// Implements the `Display` trait for the `MessageTypes` enum,
/// allowing it to be converted into a human-readable string.
impl Display for MessageTypes {
    /// Formats the `MessageTypes` enum variant as a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            // Match the current enum variant and return a corresponding string
            match self {
                MessageTypes::Request => "Request",
                MessageTypes::Response => "Response",
                MessageTypes::Notification => "Notification",
                MessageTypes::Error => "Error",
            }
        )
    }
}

/// A utility function used internally to detect the message type from the payload.
/// This function is used when deserializing a `ClientMessage` into strongly-typed structs that represent the specific message received.
#[allow(dead_code)]
fn detect_message_type(value: &serde_json::Value) -> MessageTypes {
    let id_field = value.get("id");

    if id_field.is_some() && value.get("error").is_some() {
        return MessageTypes::Error;
    }

    let method_field = value.get("method");
    let result_field = value.get("result");

    if id_field.is_some() {
        if result_field.is_some() && method_field.is_none() {
            return MessageTypes::Response;
        } else if method_field.is_some() {
            return MessageTypes::Request;
        }
    } else if method_field.is_some() {
        return MessageTypes::Notification;
    }

    MessageTypes::Request
}

/// Represents a generic MCP (Model Context Protocol) message.
/// This trait defines methods to classify and extract information from messages.
pub trait RpcMessage: McpMessage {
    fn request_id(&self) -> Option<&RequestId>;
    fn jsonrpc(&self) -> &str;
}

pub trait McpMessage {
    fn is_response(&self) -> bool;
    fn is_request(&self) -> bool;
    fn is_notification(&self) -> bool;
    fn is_error(&self) -> bool;
    fn message_type(&self) -> MessageTypes;
}

/// A trait for converting a message of type `T` into `Self`.
/// This is useful for transforming mcp messages into a Type that could be serialized into a JsonrpcMessage.
///
/// For example, a ServerMessage can be constructed from a rust_mcp_schema::PingRequest by attaching a RequestId.
/// Eventually, the ServerMessage can be serialized into a valid JsonrpcMessage for transmission over the transport.
pub trait FromMessage<T>
where
    Self: Sized,
{
    fn from_message(message: T, request_id: Option<RequestId>) -> std::result::Result<Self, RpcError>;
}

pub trait ToMessage<T>
where
    T: FromMessage<Self>,
    Self: Sized,
{
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<T, RpcError>;
}

//*******************************//
//** RequestId Implementations **//
//*******************************//

// Implement PartialEq and Eq for RequestId
impl PartialEq for RequestId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RequestId::String(a), RequestId::String(b)) => a == b,
            (RequestId::Integer(a), RequestId::Integer(b)) => a == b,
            _ => false, // Different variants are never equal
        }
    }
}

impl PartialEq<RequestId> for &RequestId {
    fn eq(&self, other: &RequestId) -> bool {
        (*self).eq(other)
    }
}

impl Eq for RequestId {}

// Implement Hash for RequestId, so we can store it in HashMaps, HashSets, etc.
impl Hash for RequestId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RequestId::String(s) => {
                0u8.hash(state); // Prefix with 0 for String variant
                s.hash(state);
            }
            RequestId::Integer(i) => {
                1u8.hash(state); // Prefix with 1 for Integer variant
                i.hash(state);
            }
        }
    }
}

//*******************//
//** ClientMessage **//
//*******************//

/// "Similar to JsonrpcMessage, but with the variants restricted to client-side messages."
/// ClientMessage represents a message sent by an MCP Client and received by an MCP Server.
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ClientMessage {
    Request(ClientJsonrpcRequest),
    Notification(ClientJsonrpcNotification),
    Response(ClientJsonrpcResponse),
    Error(JsonrpcErrorResponse),
}

impl ClientMessage {
    /// Converts the current message into a `ClientJsonrpcResponse` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Response`. If so, it returns the
    /// `ClientJsonrpcResponse` wrapped in a `Result::Ok`. If the message is not a `Response`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(ClientJsonrpcResponse)` if the message is a valid `Response`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_response(self) -> std::result::Result<ClientJsonrpcResponse, RpcError> {
        if let Self::Response(response) = self {
            Ok(response)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Response,
                self.message_type()
            )))
        }
    }

    /// Converts the current message into a `ClientJsonrpcRequest` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Request`. If so, it returns the
    /// `ClientJsonrpcRequest` wrapped in a `Result::Ok`. If the message is not a `Request`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(ClientJsonrpcRequest)` if the message is a valid `Request`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_request(self) -> std::result::Result<ClientJsonrpcRequest, RpcError> {
        if let Self::Request(request) = self {
            Ok(request)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Request,
                self.message_type()
            )))
        }
    }

    /// Converts the current message into a `ClientJsonrpcNotification` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Notification`. If so, it returns the
    /// `ClientJsonrpcNotification` wrapped in a `Result::Ok`. If the message is not a `Notification`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(ClientJsonrpcNotification)` if the message is a valid `Notification`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_notification(self) -> std::result::Result<ClientJsonrpcNotification, RpcError> {
        if let Self::Notification(notification) = self {
            Ok(notification)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Notification,
                self.message_type()
            )))
        }
    }

    /// Converts the current message into a `JsonrpcErrorResponse` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Error`. If so, it returns the
    /// `JsonrpcErrorResponse` wrapped in a `Result::Ok`. If the message is not a `Error`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(JsonrpcErrorResponse)` if the message is a valid `Error`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_error(self) -> std::result::Result<JsonrpcErrorResponse, RpcError> {
        if let Self::Error(error) = self {
            Ok(error)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Error,
                self.message_type()
            )))
        }
    }

    /// Returns `true` if message is an `InitializeRequest`.
    pub fn is_initialize_request(&self) -> bool {
        matches!(self, Self::Request(ClientJsonrpcRequest::InitializeRequest(_)))
    }

    /// Returns `true` if the message is an `InitializedNotification`
    pub fn is_initialized_notification(&self) -> bool {
        matches!(
            self,
            Self::Notification(ClientJsonrpcNotification::InitializedNotification(_))
        )
    }
}

impl From<ClientJsonrpcNotification> for ClientMessage {
    fn from(value: ClientJsonrpcNotification) -> Self {
        Self::Notification(value)
    }
}

impl From<ClientJsonrpcRequest> for ClientMessage {
    fn from(value: ClientJsonrpcRequest) -> Self {
        Self::Request(value)
    }
}

impl From<ClientJsonrpcResponse> for ClientMessage {
    fn from(value: ClientJsonrpcResponse) -> Self {
        Self::Response(value)
    }
}

impl RpcMessage for ClientMessage {
    // Retrieves the request ID associated with the message, if applicable
    fn request_id(&self) -> Option<&RequestId> {
        match self {
            // If the message is a request, return the associated request ID
            ClientMessage::Request(client_jsonrpc_request) => match client_jsonrpc_request {
                ClientJsonrpcRequest::InitializeRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::PingRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::ListResourcesRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::ListResourceTemplatesRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::ReadResourceRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::SubscribeRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::UnsubscribeRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::ListPromptsRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::GetPromptRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::ListToolsRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::CallToolRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::GetTaskRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::GetTaskPayloadRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::CancelTaskRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::ListTasksRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::SetLevelRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::CompleteRequest(request) => Some(&request.id),
                ClientJsonrpcRequest::CustomRequest(request) => Some(&request.id),
            },
            // Notifications do not have request IDs
            ClientMessage::Notification(_) => None,
            // If the message is a response, return the associated request ID
            ClientMessage::Response(client_jsonrpc_response) => Some(&client_jsonrpc_response.id),
            // If the message is an error, return the associated request ID
            ClientMessage::Error(jsonrpc_error) => jsonrpc_error.id.as_ref(),
        }
    }

    fn jsonrpc(&self) -> &str {
        match self {
            ClientMessage::Request(client_jsonrpc_request) => client_jsonrpc_request.jsonrpc(),
            ClientMessage::Notification(notification) => notification.jsonrpc(),
            ClientMessage::Response(client_jsonrpc_response) => client_jsonrpc_response.jsonrpc(),
            ClientMessage::Error(jsonrpc_error) => jsonrpc_error.jsonrpc(),
        }
    }
}

// Implementing the `McpMessage` trait for `ClientMessage`
impl McpMessage for ClientMessage {
    // Returns true if the message is a response type
    fn is_response(&self) -> bool {
        matches!(self, ClientMessage::Response(_))
    }

    // Returns true if the message is a request type
    fn is_request(&self) -> bool {
        matches!(self, ClientMessage::Request(_))
    }

    // Returns true if the message is a notification type (i.e., does not expect a response)
    fn is_notification(&self) -> bool {
        matches!(self, ClientMessage::Notification(_))
    }

    // Returns true if the message represents an error
    fn is_error(&self) -> bool {
        matches!(self, ClientMessage::Error(_))
    }

    /// Determines the type of the message and returns the corresponding `MessageTypes` variant.
    fn message_type(&self) -> MessageTypes {
        match self {
            ClientMessage::Request(_) => MessageTypes::Request,
            ClientMessage::Notification(_) => MessageTypes::Notification,
            ClientMessage::Response(_) => MessageTypes::Response,
            ClientMessage::Error(_) => MessageTypes::Error,
        }
    }
}

//**************************//
//** ClientJsonrpcRequest **//
//**************************//

/// "Similar to JsonrpcRequest , but with the variants restricted to client-side requests."
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[serde(untagged)]
pub enum ClientJsonrpcRequest {
    InitializeRequest(InitializeRequest),
    PingRequest(PingRequest),
    ListResourcesRequest(ListResourcesRequest),
    ListResourceTemplatesRequest(ListResourceTemplatesRequest),
    ReadResourceRequest(ReadResourceRequest),
    SubscribeRequest(SubscribeRequest),
    UnsubscribeRequest(UnsubscribeRequest),
    ListPromptsRequest(ListPromptsRequest),
    GetPromptRequest(GetPromptRequest),
    ListToolsRequest(ListToolsRequest),
    CallToolRequest(CallToolRequest),
    GetTaskRequest(GetTaskRequest),
    GetTaskPayloadRequest(GetTaskPayloadRequest),
    CancelTaskRequest(CancelTaskRequest),
    ListTasksRequest(ListTasksRequest),
    SetLevelRequest(SetLevelRequest),
    CompleteRequest(CompleteRequest),
    CustomRequest(JsonrpcRequest),
}

impl ClientJsonrpcRequest {
    pub fn new(id: RequestId, request: RequestFromClient) -> Self {
        match request {
            RequestFromClient::InitializeRequest(params) => Self::InitializeRequest(InitializeRequest::new(id, params)),
            RequestFromClient::PingRequest(params) => Self::PingRequest(PingRequest::new(id, params)),
            RequestFromClient::ListResourcesRequest(params) => {
                Self::ListResourcesRequest(ListResourcesRequest::new(id, params))
            }
            RequestFromClient::ListResourceTemplatesRequest(params) => {
                Self::ListResourceTemplatesRequest(ListResourceTemplatesRequest::new(id, params))
            }
            RequestFromClient::ReadResourceRequest(params) => {
                Self::ReadResourceRequest(ReadResourceRequest::new(id, params))
            }
            RequestFromClient::SubscribeRequest(params) => Self::SubscribeRequest(SubscribeRequest::new(id, params)),
            RequestFromClient::UnsubscribeRequest(params) => Self::UnsubscribeRequest(UnsubscribeRequest::new(id, params)),
            RequestFromClient::ListPromptsRequest(params) => Self::ListPromptsRequest(ListPromptsRequest::new(id, params)),
            RequestFromClient::GetPromptRequest(params) => Self::GetPromptRequest(GetPromptRequest::new(id, params)),
            RequestFromClient::ListToolsRequest(params) => Self::ListToolsRequest(ListToolsRequest::new(id, params)),
            RequestFromClient::CallToolRequest(params) => Self::CallToolRequest(CallToolRequest::new(id, params)),
            RequestFromClient::GetTaskRequest(params) => Self::GetTaskRequest(GetTaskRequest::new(id, params)),
            RequestFromClient::GetTaskPayloadRequest(params) => {
                Self::GetTaskPayloadRequest(GetTaskPayloadRequest::new(id, params))
            }
            RequestFromClient::CancelTaskRequest(params) => Self::CancelTaskRequest(CancelTaskRequest::new(id, params)),
            RequestFromClient::ListTasksRequest(params) => Self::ListTasksRequest(ListTasksRequest::new(id, params)),
            RequestFromClient::SetLevelRequest(params) => Self::SetLevelRequest(SetLevelRequest::new(id, params)),
            RequestFromClient::CompleteRequest(params) => Self::CompleteRequest(CompleteRequest::new(id, params)),
            RequestFromClient::CustomRequest(params) => {
                Self::CustomRequest(JsonrpcRequest::new(id, params.method, params.params))
            }
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        match self {
            ClientJsonrpcRequest::InitializeRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::PingRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::ListResourcesRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::ListResourceTemplatesRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::ReadResourceRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::SubscribeRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::UnsubscribeRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::ListPromptsRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::GetPromptRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::ListToolsRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::CallToolRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::GetTaskRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::GetTaskPayloadRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::CancelTaskRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::ListTasksRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::SetLevelRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::CompleteRequest(request) => request.jsonrpc(),
            ClientJsonrpcRequest::CustomRequest(request) => request.jsonrpc(),
        }
    }

    pub fn request_id(&self) -> &RequestId {
        match self {
            ClientJsonrpcRequest::InitializeRequest(request) => &request.id,
            ClientJsonrpcRequest::PingRequest(request) => &request.id,
            ClientJsonrpcRequest::ListResourcesRequest(request) => &request.id,
            ClientJsonrpcRequest::ListResourceTemplatesRequest(request) => &request.id,
            ClientJsonrpcRequest::ReadResourceRequest(request) => &request.id,
            ClientJsonrpcRequest::SubscribeRequest(request) => &request.id,
            ClientJsonrpcRequest::UnsubscribeRequest(request) => &request.id,
            ClientJsonrpcRequest::ListPromptsRequest(request) => &request.id,
            ClientJsonrpcRequest::GetPromptRequest(request) => &request.id,
            ClientJsonrpcRequest::ListToolsRequest(request) => &request.id,
            ClientJsonrpcRequest::CallToolRequest(request) => &request.id,
            ClientJsonrpcRequest::GetTaskRequest(request) => &request.id,
            ClientJsonrpcRequest::GetTaskPayloadRequest(request) => &request.id,
            ClientJsonrpcRequest::CancelTaskRequest(request) => &request.id,
            ClientJsonrpcRequest::ListTasksRequest(request) => &request.id,
            ClientJsonrpcRequest::SetLevelRequest(request) => &request.id,
            ClientJsonrpcRequest::CompleteRequest(request) => &request.id,
            ClientJsonrpcRequest::CustomRequest(request) => &request.id,
        }
    }

    pub fn is_task_augmented(&self) -> bool {
        if let ClientJsonrpcRequest::CallToolRequest(call_tool_request) = self {
            call_tool_request.is_task_augmented()
        } else {
            false
        }
    }

    pub fn method(&self) -> &str {
        match self {
            ClientJsonrpcRequest::InitializeRequest(request) => request.method(),
            ClientJsonrpcRequest::PingRequest(request) => request.method(),
            ClientJsonrpcRequest::ListResourcesRequest(request) => request.method(),
            ClientJsonrpcRequest::ListResourceTemplatesRequest(request) => request.method(),
            ClientJsonrpcRequest::ReadResourceRequest(request) => request.method(),
            ClientJsonrpcRequest::SubscribeRequest(request) => request.method(),
            ClientJsonrpcRequest::UnsubscribeRequest(request) => request.method(),
            ClientJsonrpcRequest::ListPromptsRequest(request) => request.method(),
            ClientJsonrpcRequest::GetPromptRequest(request) => request.method(),
            ClientJsonrpcRequest::ListToolsRequest(request) => request.method(),
            ClientJsonrpcRequest::CallToolRequest(request) => request.method(),
            ClientJsonrpcRequest::GetTaskRequest(request) => request.method(),
            ClientJsonrpcRequest::GetTaskPayloadRequest(request) => request.method(),
            ClientJsonrpcRequest::CancelTaskRequest(request) => request.method(),
            ClientJsonrpcRequest::ListTasksRequest(request) => request.method(),
            ClientJsonrpcRequest::SetLevelRequest(request) => request.method(),
            ClientJsonrpcRequest::CompleteRequest(request) => request.method(),
            ClientJsonrpcRequest::CustomRequest(request) => request.method.as_str(),
        }
    }
}

impl From<ClientJsonrpcRequest> for RequestFromClient {
    fn from(request: ClientJsonrpcRequest) -> Self {
        match request {
            ClientJsonrpcRequest::InitializeRequest(request) => Self::InitializeRequest(request.params),
            ClientJsonrpcRequest::PingRequest(request) => Self::PingRequest(request.params),
            ClientJsonrpcRequest::ListResourcesRequest(request) => Self::ListResourcesRequest(request.params),
            ClientJsonrpcRequest::ListResourceTemplatesRequest(request) => {
                Self::ListResourceTemplatesRequest(request.params)
            }
            ClientJsonrpcRequest::ReadResourceRequest(request) => Self::ReadResourceRequest(request.params),
            ClientJsonrpcRequest::SubscribeRequest(request) => Self::SubscribeRequest(request.params),
            ClientJsonrpcRequest::UnsubscribeRequest(request) => Self::UnsubscribeRequest(request.params),
            ClientJsonrpcRequest::ListPromptsRequest(request) => Self::ListPromptsRequest(request.params),
            ClientJsonrpcRequest::GetPromptRequest(request) => Self::GetPromptRequest(request.params),
            ClientJsonrpcRequest::ListToolsRequest(request) => Self::ListToolsRequest(request.params),
            ClientJsonrpcRequest::CallToolRequest(request) => Self::CallToolRequest(request.params),
            ClientJsonrpcRequest::GetTaskRequest(request) => Self::GetTaskRequest(request.params),
            ClientJsonrpcRequest::GetTaskPayloadRequest(request) => Self::GetTaskPayloadRequest(request.params),
            ClientJsonrpcRequest::CancelTaskRequest(request) => Self::CancelTaskRequest(request.params),
            ClientJsonrpcRequest::ListTasksRequest(request) => Self::ListTasksRequest(request.params),
            ClientJsonrpcRequest::SetLevelRequest(request) => Self::SetLevelRequest(request.params),
            ClientJsonrpcRequest::CompleteRequest(request) => Self::CompleteRequest(request.params),
            ClientJsonrpcRequest::CustomRequest(request) => Self::CustomRequest(CustomRequest {
                method: request.method,
                params: request.params,
            }),
        }
    }
}

/// Formats the ClientJsonrpcRequest as a JSON string.
impl Display for ClientJsonrpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

impl FromStr for ClientJsonrpcRequest {
    type Err = RpcError;

    /// Parses a JSON-RPC request from a string.
    ///
    /// This implementation allows `ClientJsonrpcRequest` to be created
    /// from a JSON string using the `from_str` method.
    ///
    /// # Arguments
    /// * `s` - A JSON string representing a JSON-RPC request.
    ///
    /// # Returns
    /// * `Ok(ClientJsonrpcRequest)` if parsing is successful.
    /// * `Err(RpcError)` if the string is not valid JSON.
    ///
    /// # Example
    /// ```
    /// use std::str::FromStr;
    /// use rust_mcp_schema::schema_utils::ClientJsonrpcRequest;
    ///
    /// let json = r#"{"jsonrpc": "2.0", "method": "initialize", "id": 1}"#;
    /// let request = ClientJsonrpcRequest::from_str(json);
    /// assert!(request.is_ok());
    /// ```
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

//*************************//
//** Request From Client **//
//*************************//

/// To determine standard and custom request from the client side
/// Custom requests are of type serde_json::Value and can be deserialized into any custom type.
#[allow(clippy::large_enum_variant)]
#[derive(::serde::Serialize, ::serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RequestFromClient {
    InitializeRequest(InitializeRequestParams),
    PingRequest(Option<RequestParams>),
    ListResourcesRequest(Option<PaginatedRequestParams>),
    ListResourceTemplatesRequest(Option<PaginatedRequestParams>),
    ReadResourceRequest(ReadResourceRequestParams),
    SubscribeRequest(SubscribeRequestParams),
    UnsubscribeRequest(UnsubscribeRequestParams),
    ListPromptsRequest(Option<PaginatedRequestParams>),
    GetPromptRequest(GetPromptRequestParams),
    ListToolsRequest(Option<PaginatedRequestParams>),
    CallToolRequest(CallToolRequestParams),
    GetTaskRequest(GetTaskParams),
    GetTaskPayloadRequest(GetTaskPayloadParams),
    CancelTaskRequest(CancelTaskParams),
    ListTasksRequest(Option<PaginatedRequestParams>),
    SetLevelRequest(SetLevelRequestParams),
    CompleteRequest(CompleteRequestParams),
    CustomRequest(CustomRequest),
}

impl RequestFromClient {
    pub fn method(&self) -> &str {
        match self {
            RequestFromClient::InitializeRequest(_request) => InitializeRequest::method_value(),
            RequestFromClient::PingRequest(_request) => PingRequest::method_value(),
            RequestFromClient::ListResourcesRequest(_request) => ListResourcesRequest::method_value(),
            RequestFromClient::ListResourceTemplatesRequest(_request) => ListResourceTemplatesRequest::method_value(),
            RequestFromClient::ReadResourceRequest(_request) => ReadResourceRequest::method_value(),
            RequestFromClient::SubscribeRequest(_request) => SubscribeRequest::method_value(),
            RequestFromClient::UnsubscribeRequest(_request) => UnsubscribeRequest::method_value(),
            RequestFromClient::ListPromptsRequest(_request) => ListPromptsRequest::method_value(),
            RequestFromClient::GetPromptRequest(_request) => GetPromptRequest::method_value(),
            RequestFromClient::ListToolsRequest(_request) => ListToolsRequest::method_value(),
            RequestFromClient::CallToolRequest(_request) => CallToolRequest::method_value(),
            RequestFromClient::GetTaskRequest(_request) => GetTaskRequest::method_value(),
            RequestFromClient::GetTaskPayloadRequest(_request) => GetTaskPayloadRequest::method_value(),
            RequestFromClient::CancelTaskRequest(_request) => CancelTaskRequest::method_value(),
            RequestFromClient::ListTasksRequest(_request) => ListTasksRequest::method_value(),
            RequestFromClient::SetLevelRequest(_request) => SetLevelRequest::method_value(),
            RequestFromClient::CompleteRequest(_request) => CompleteRequest::method_value(),
            RequestFromClient::CustomRequest(request) => request.method.as_str(),
        }
    }
    /// Returns `true` if the request is an `InitializeRequest`.
    pub fn is_initialize_request(&self) -> bool {
        matches!(self, RequestFromClient::InitializeRequest(_))
    }
}

// impl From<ClientRequest> for RequestFromClient {
//     fn from(value: ClientRequest) -> Self {
//         Self::ClientRequest(value)
//     }
// }

// impl From<serde_json::Value> for RequestFromClient {
//     fn from(value: serde_json::Value) -> Self {
//         Self::CustomRequest(value)
//     }
// }

// impl<'de> serde::Deserialize<'de> for RequestFromClient {
//     fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let raw_value = Value::deserialize(deserializer)?;

//         let client_result = ClientRequest::deserialize(&raw_value);

//         match client_result {
//             Ok(client_request) => Ok(Self::ClientRequest(client_request)),
//             Err(_) => Ok(Self::CustomRequest(raw_value)),
//         }
//     }
// }

//*******************************//
//** ClientJsonrpcNotification **//
//*******************************//

/// "Similar to JsonrpcNotification , but with the variants restricted to client-side notifications."
#[derive(Clone, Debug, ::serde::Deserialize, ::serde::Serialize)]
#[serde(untagged)]
pub enum ClientJsonrpcNotification {
    CancelledNotification(CancelledNotification),
    InitializedNotification(InitializedNotification),
    ProgressNotification(ProgressNotification),
    TaskStatusNotification(TaskStatusNotification),
    RootsListChangedNotification(RootsListChangedNotification),
    CustomNotification(JsonrpcNotification),
}

impl ClientJsonrpcNotification {
    pub fn new(notification: NotificationFromClient) -> Self {
        match notification {
            NotificationFromClient::CancelledNotification(params) => {
                Self::CancelledNotification(CancelledNotification::new(params))
            }
            NotificationFromClient::InitializedNotification(params) => {
                Self::InitializedNotification(InitializedNotification::new(params))
            }
            NotificationFromClient::ProgressNotification(params) => {
                Self::ProgressNotification(ProgressNotification::new(params))
            }
            NotificationFromClient::TaskStatusNotification(params) => {
                Self::TaskStatusNotification(TaskStatusNotification::new(params))
            }
            NotificationFromClient::RootsListChangedNotification(params) => {
                Self::RootsListChangedNotification(RootsListChangedNotification::new(params))
            }
            NotificationFromClient::CustomNotification(params) => {
                Self::CustomNotification(JsonrpcNotification::new(params.method, params.params))
            }
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        match self {
            ClientJsonrpcNotification::CancelledNotification(notification) => notification.jsonrpc(),
            ClientJsonrpcNotification::InitializedNotification(notification) => notification.jsonrpc(),
            ClientJsonrpcNotification::ProgressNotification(notification) => notification.jsonrpc(),
            ClientJsonrpcNotification::TaskStatusNotification(notification) => notification.jsonrpc(),
            ClientJsonrpcNotification::RootsListChangedNotification(notification) => notification.jsonrpc(),
            ClientJsonrpcNotification::CustomNotification(notification) => notification.jsonrpc(),
        }
    }

    /// Returns `true` if the message is an `InitializedNotification`
    pub fn is_initialized_notification(&self) -> bool {
        matches!(self, Self::InitializedNotification(_))
    }
}

impl From<ClientJsonrpcNotification> for NotificationFromClient {
    fn from(notification: ClientJsonrpcNotification) -> Self {
        match notification {
            ClientJsonrpcNotification::CancelledNotification(notification) => {
                Self::CancelledNotification(notification.params)
            }
            ClientJsonrpcNotification::InitializedNotification(notification) => {
                Self::InitializedNotification(notification.params)
            }
            ClientJsonrpcNotification::ProgressNotification(notification) => Self::ProgressNotification(notification.params),
            ClientJsonrpcNotification::TaskStatusNotification(notification) => {
                Self::TaskStatusNotification(notification.params)
            }
            ClientJsonrpcNotification::RootsListChangedNotification(notification) => {
                Self::RootsListChangedNotification(notification.params)
            }
            ClientJsonrpcNotification::CustomNotification(notification) => Self::CustomNotification(CustomNotification {
                method: notification.method,
                params: notification.params,
            }),
        }
    }
}

/// Formats the ClientJsonrpcNotification as a JSON string.
impl Display for ClientJsonrpcNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

impl FromStr for ClientJsonrpcNotification {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

//*******************************//
//**  NotificationFromClient   **//
//*******************************//

/// To determine standard and custom notifications received from the MCP Client
/// Custom notifications are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, ::serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum NotificationFromClient {
    CancelledNotification(CancelledNotificationParams),
    InitializedNotification(Option<NotificationParams>),
    ProgressNotification(ProgressNotificationParams),
    TaskStatusNotification(TaskStatusNotificationParams),
    RootsListChangedNotification(Option<NotificationParams>),
    CustomNotification(CustomNotification),
}

// impl TryFrom<NotificationFromClient> for ClientNotification {
//     type Error = RpcError;
//     fn try_from(value: NotificationFromClient) -> result::Result<Self, Self::Error> {
//         if let NotificationFromClient::ClientNotification(client_notification) = value {
//             Ok(client_notification)
//         } else {
//             Err(RpcError::internal_error().with_message("Not a ClientNotification".to_string()))
//         }
//     }
// }

impl NotificationFromClient {
    /// Returns `true` if the message is an `InitializedNotification`
    pub fn is_initialized_notification(&self) -> bool {
        matches!(self, NotificationFromClient::InitializedNotification(_))
    }

    //TODO: 'static
    pub fn method(&self) -> &str {
        match self {
            NotificationFromClient::CancelledNotification(_notification) => CancelledNotification::method_value(),
            NotificationFromClient::InitializedNotification(_notification) => InitializedNotification::method_value(),
            NotificationFromClient::ProgressNotification(_notification) => ProgressNotification::method_value(),
            NotificationFromClient::TaskStatusNotification(_notification) => TaskStatusNotification::method_value(),
            NotificationFromClient::RootsListChangedNotification(_notification) => {
                RootsListChangedNotification::method_value()
            }
            NotificationFromClient::CustomNotification(notification) => notification.method.as_str(),
        }
    }
}

//*******************************//
//**   ClientJsonrpcResponse   **//
//*******************************//

/// "Similar to JsonrpcResponse , but with the variants restricted to client-side responses."
#[derive(Clone, Debug)]
pub struct ClientJsonrpcResponse {
    pub id: RequestId,
    jsonrpc: ::std::string::String,
    pub result: ResultFromClient,
}

impl ClientJsonrpcResponse {
    pub fn new(id: RequestId, result: ResultFromClient) -> Self {
        Self {
            id,
            jsonrpc: JSONRPC_VERSION.to_string(),
            result,
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        &self.jsonrpc
    }
}

/// Formats the ClientJsonrpcResponse as a JSON string.
impl Display for ClientJsonrpcResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

impl FromStr for ClientJsonrpcResponse {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
//*******************************//
//**      ResultFromClient     **//
//*******************************//

#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResultFromClient {
    GetTaskResult(GetTaskResult),
    CancelTaskResult(CancelTaskResult),
    ListTasksResult(ListTasksResult),
    CreateMessageResult(CreateMessageResult),
    ListRootsResult(ListRootsResult),
    ElicitResult(ElicitResult),
    CreateTaskResult(CreateTaskResult),
    Result(Result),
    GetTaskPayloadResult(GetTaskPayloadResult),
}

#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ClientTaskResult {
    ElicitResult(ElicitResult),
    CreateMessageResult(CreateMessageResult),
}

pub type ServerTaskResult = CallToolResult;

impl TryFrom<ResultFromClient> for ClientTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        match value {
            ResultFromClient::CreateMessageResult(create_message_result) => {
                Ok(Self::CreateMessageResult(create_message_result))
            }
            ResultFromClient::ElicitResult(elicit_result) => Ok(Self::ElicitResult(elicit_result)),
            _ => Err(RpcError::internal_error().with_message("Not a ClientTaskResult variant".to_string())),
        }
    }
}

impl From<ClientTaskResult> for ResultFromClient {
    fn from(value: ClientTaskResult) -> Self {
        match value {
            ClientTaskResult::ElicitResult(elicit_result) => Self::ElicitResult(elicit_result),
            ClientTaskResult::CreateMessageResult(create_message_result) => Self::CreateMessageResult(create_message_result),
        }
    }
}
//*******************************//
//**       ClientMessage       **//
//*******************************//

impl FromStr for ClientMessage {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

//*******************//
//** ServerMessage **//
//*******************//

/// "Similar to JsonrpcMessage, but with the variants restricted to client-side messages."
/// ServerMessage represents a message sent by an MCP Server and received by an MCP Client.
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ServerMessage {
    Request(ServerJsonrpcRequest),
    Notification(ServerJsonrpcNotification),
    Response(ServerJsonrpcResponse),
    Error(JsonrpcErrorResponse),
}

impl ServerMessage {
    /// Converts the current message into a `ServerJsonrpcResponse` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Response`. If so, it returns the
    /// `ServerJsonrpcResponse` wrapped in a `Result::Ok`. If the message is not a `Response`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(ServerJsonrpcResponse)` if the message is a valid `Response`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_response(self) -> std::result::Result<ServerJsonrpcResponse, RpcError> {
        if let Self::Response(response) = self {
            Ok(response)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Response,
                self.message_type()
            )))
        }
    }

    /// Converts the current message into a `ServerJsonrpcRequest` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Request`. If so, it returns the
    /// `ServerJsonrpcRequest` wrapped in a `Result::Ok`. If the message is not a `Request`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(ServerJsonrpcRequest)` if the message is a valid `Request`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_request(self) -> std::result::Result<ServerJsonrpcRequest, RpcError> {
        if let Self::Request(request) = self {
            Ok(request)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Request,
                self.message_type()
            )))
        }
    }

    /// Converts the current message into a `ServerJsonrpcNotification` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Notification`. If so, it returns the
    /// `ServerJsonrpcNotification` wrapped in a `Result::Ok`. If the message is not a `Notification`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(ServerJsonrpcNotification)` if the message is a valid `Notification`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_notification(self) -> std::result::Result<ServerJsonrpcNotification, RpcError> {
        if let Self::Notification(notification) = self {
            Ok(notification)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Notification,
                self.message_type()
            )))
        }
    }

    /// Converts the current message into a `JsonrpcErrorResponse` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Error`. If so, it returns the
    /// `JsonrpcErrorResponse` wrapped in a `Result::Ok`. If the message is not a `Error`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(JsonrpcErrorResponse)` if the message is a valid `Error`.
    /// - `Err(RpcError)` if the message type is invalid
    pub fn as_error(self) -> std::result::Result<JsonrpcErrorResponse, RpcError> {
        if let Self::Error(error) = self {
            Ok(error)
        } else {
            Err(RpcError::internal_error().with_message(format!(
                "Invalid message type, expected: \"{}\" received\"{}\"",
                MessageTypes::Error,
                self.message_type()
            )))
        }
    }
}

impl From<ServerJsonrpcNotification> for ServerMessage {
    fn from(value: ServerJsonrpcNotification) -> Self {
        Self::Notification(value)
    }
}

impl From<ServerJsonrpcRequest> for ServerMessage {
    fn from(value: ServerJsonrpcRequest) -> Self {
        Self::Request(value)
    }
}

impl From<ServerJsonrpcResponse> for ServerMessage {
    fn from(value: ServerJsonrpcResponse) -> Self {
        Self::Response(value)
    }
}

impl RpcMessage for ServerMessage {
    // Retrieves the request ID associated with the message, if applicable
    fn request_id(&self) -> Option<&RequestId> {
        match self {
            // If the message is a request, return the associated request ID
            ServerMessage::Request(server_jsonrpc_request) => match server_jsonrpc_request {
                ServerJsonrpcRequest::PingRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::GetTaskRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::GetTaskPayloadRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::CancelTaskRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::ListTasksRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::CreateMessageRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::ListRootsRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::ElicitRequest(request) => Some(&request.id),
                ServerJsonrpcRequest::CustomRequest(request) => Some(&request.id),
            },
            // Notifications do not have request IDs
            ServerMessage::Notification(_) => None,
            // If the message is a response, return the associated request ID
            ServerMessage::Response(server_jsonrpc_response) => Some(&server_jsonrpc_response.id),
            // If the message is an error, return the associated request ID
            ServerMessage::Error(jsonrpc_error) => jsonrpc_error.id.as_ref(),
        }
    }

    fn jsonrpc(&self) -> &str {
        match self {
            // If the message is a request, return the associated request ID
            ServerMessage::Request(server_jsonrpc_request) => match server_jsonrpc_request {
                ServerJsonrpcRequest::PingRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::GetTaskRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::GetTaskPayloadRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::CancelTaskRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::ListTasksRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::CreateMessageRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::ListRootsRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::ElicitRequest(request) => request.jsonrpc(),
                ServerJsonrpcRequest::CustomRequest(request) => request.jsonrpc(),
            },

            // Notifications do not have request IDs
            ServerMessage::Notification(notification) => notification.jsonrpc(),
            // If the message is a response, return the associated request ID
            ServerMessage::Response(server_jsonrpc_response) => server_jsonrpc_response.jsonrpc(),
            // If the message is an error, return the associated request ID
            ServerMessage::Error(jsonrpc_error) => jsonrpc_error.jsonrpc(),
        }
    }
}

// Implementing the `McpMessage` trait for `ServerMessage`
impl McpMessage for ServerMessage {
    // Returns true if the message is a response type
    fn is_response(&self) -> bool {
        matches!(self, ServerMessage::Response(_))
    }

    // Returns true if the message is a request type
    fn is_request(&self) -> bool {
        matches!(self, ServerMessage::Request(_))
    }

    // Returns true if the message is a notification type (i.e., does not expect a response)
    fn is_notification(&self) -> bool {
        matches!(self, ServerMessage::Notification(_))
    }

    // Returns true if the message represents an error
    fn is_error(&self) -> bool {
        matches!(self, ServerMessage::Error(_))
    }

    /// Determines the type of the message and returns the corresponding `MessageTypes` variant.
    fn message_type(&self) -> MessageTypes {
        match self {
            ServerMessage::Request(_) => MessageTypes::Request,
            ServerMessage::Notification(_) => MessageTypes::Notification,
            ServerMessage::Response(_) => MessageTypes::Response,
            ServerMessage::Error(_) => MessageTypes::Error,
        }
    }
}

impl FromStr for ServerMessage {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

//**************************//
//** ServerJsonrpcRequest **//
//**************************//

/// "Similar to JsonrpcRequest , but with the variants restricted to client-side requests."
#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
pub enum ServerJsonrpcRequest {
    PingRequest(PingRequest),
    GetTaskRequest(GetTaskRequest),
    GetTaskPayloadRequest(GetTaskPayloadRequest),
    CancelTaskRequest(CancelTaskRequest),
    ListTasksRequest(ListTasksRequest),
    CreateMessageRequest(CreateMessageRequest),
    ListRootsRequest(ListRootsRequest),
    ElicitRequest(ElicitRequest),
    CustomRequest(JsonrpcRequest),
}

impl ServerJsonrpcRequest {
    pub fn new(request_id: RequestId, request: RequestFromServer) -> Self {
        match request {
            RequestFromServer::PingRequest(params) => Self::PingRequest(PingRequest::new(request_id, params)),
            RequestFromServer::GetTaskRequest(params) => Self::GetTaskRequest(GetTaskRequest::new(request_id, params)),
            RequestFromServer::GetTaskPayloadRequest(params) => {
                Self::GetTaskPayloadRequest(GetTaskPayloadRequest::new(request_id, params))
            }
            RequestFromServer::CancelTaskRequest(params) => {
                Self::CancelTaskRequest(CancelTaskRequest::new(request_id, params))
            }
            RequestFromServer::ListTasksRequest(params) => Self::ListTasksRequest(ListTasksRequest::new(request_id, params)),
            RequestFromServer::CreateMessageRequest(params) => {
                Self::CreateMessageRequest(CreateMessageRequest::new(request_id, params))
            }
            RequestFromServer::ListRootsRequest(params) => Self::ListRootsRequest(ListRootsRequest::new(request_id, params)),
            RequestFromServer::ElicitRequest(params) => Self::ElicitRequest(ElicitRequest::new(request_id, params)),
            RequestFromServer::CustomRequest(request) => {
                Self::CustomRequest(JsonrpcRequest::new(request_id, request.method, request.params))
            }
        }
    }

    pub fn request_id(&self) -> &RequestId {
        match self {
            ServerJsonrpcRequest::PingRequest(request) => &request.id,
            ServerJsonrpcRequest::GetTaskRequest(request) => &request.id,
            ServerJsonrpcRequest::GetTaskPayloadRequest(request) => &request.id,
            ServerJsonrpcRequest::CancelTaskRequest(request) => &request.id,
            ServerJsonrpcRequest::ListTasksRequest(request) => &request.id,
            ServerJsonrpcRequest::CreateMessageRequest(request) => &request.id,
            ServerJsonrpcRequest::ListRootsRequest(request) => &request.id,
            ServerJsonrpcRequest::ElicitRequest(request) => &request.id,
            ServerJsonrpcRequest::CustomRequest(request) => &request.id,
        }
    }

    pub fn jsonrpc(&self) -> &::std::string::String {
        match self {
            ServerJsonrpcRequest::PingRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::GetTaskRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::GetTaskPayloadRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::CancelTaskRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::ListTasksRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::CreateMessageRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::ListRootsRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::ElicitRequest(request) => request.jsonrpc(),
            ServerJsonrpcRequest::CustomRequest(request) => request.jsonrpc(),
        }
    }

    pub fn is_task_augmented(&self) -> bool {
        match self {
            ServerJsonrpcRequest::ElicitRequest(request) => request.params.is_task_augmented(),
            ServerJsonrpcRequest::CreateMessageRequest(request) => request.params.is_task_augmented(),
            _ => false,
        }
    }

    pub fn method(&self) -> &str {
        match self {
            ServerJsonrpcRequest::PingRequest(request) => request.method(),
            ServerJsonrpcRequest::GetTaskRequest(request) => request.method(),
            ServerJsonrpcRequest::GetTaskPayloadRequest(request) => request.method(),
            ServerJsonrpcRequest::CancelTaskRequest(request) => request.method(),
            ServerJsonrpcRequest::ListTasksRequest(request) => request.method(),
            ServerJsonrpcRequest::CreateMessageRequest(request) => request.method(),
            ServerJsonrpcRequest::ListRootsRequest(request) => request.method(),
            ServerJsonrpcRequest::ElicitRequest(request) => request.method(),
            ServerJsonrpcRequest::CustomRequest(request) => request.method.as_str(),
        }
    }
}

/// Formats the ServerJsonrpcRequest as a JSON string.
impl Display for ServerJsonrpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

impl FromStr for ServerJsonrpcRequest {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct CustomRequest {
    pub method: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub params: ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
}

//*************************//
//** Request From Server **//
//*************************//

/// To determine standard and custom request from the server side
/// Custom requests are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, ::serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RequestFromServer {
    PingRequest(Option<RequestParams>),
    GetTaskRequest(GetTaskParams),
    GetTaskPayloadRequest(GetTaskPayloadParams),
    CancelTaskRequest(CancelTaskParams),
    ListTasksRequest(Option<PaginatedRequestParams>),
    CreateMessageRequest(CreateMessageRequestParams),
    ListRootsRequest(Option<RequestParams>),
    ElicitRequest(ElicitRequestParams),
    CustomRequest(CustomRequest),
}

impl From<ServerJsonrpcRequest> for RequestFromServer {
    fn from(request: ServerJsonrpcRequest) -> Self {
        match request {
            ServerJsonrpcRequest::PingRequest(request) => Self::PingRequest(request.params),
            ServerJsonrpcRequest::GetTaskRequest(request) => Self::GetTaskRequest(request.params),
            ServerJsonrpcRequest::GetTaskPayloadRequest(request) => Self::GetTaskPayloadRequest(request.params),
            ServerJsonrpcRequest::CancelTaskRequest(request) => Self::CancelTaskRequest(request.params),
            ServerJsonrpcRequest::ListTasksRequest(request) => Self::ListTasksRequest(request.params),
            ServerJsonrpcRequest::CreateMessageRequest(request) => Self::CreateMessageRequest(request.params),
            ServerJsonrpcRequest::ListRootsRequest(request) => Self::ListRootsRequest(request.params),
            ServerJsonrpcRequest::ElicitRequest(request) => Self::ElicitRequest(request.params),
            ServerJsonrpcRequest::CustomRequest(request) => Self::CustomRequest(CustomRequest {
                method: request.method,
                params: request.params,
            }),
        }
    }
}

impl RequestFromServer {
    pub fn method(&self) -> &str {
        match self {
            RequestFromServer::PingRequest(_request) => PingRequest::method_value(),
            RequestFromServer::GetTaskRequest(_request) => GetTaskRequest::method_value(),
            RequestFromServer::GetTaskPayloadRequest(_request) => GetTaskPayloadRequest::method_value(),
            RequestFromServer::CancelTaskRequest(_request) => CancelTaskRequest::method_value(),
            RequestFromServer::ListTasksRequest(_request) => ListTasksRequest::method_value(),
            RequestFromServer::CreateMessageRequest(_request) => CreateMessageRequest::method_value(),
            RequestFromServer::ListRootsRequest(_request) => ListRootsRequest::method_value(),
            RequestFromServer::ElicitRequest(_request) => ElicitRequest::method_value(),
            RequestFromServer::CustomRequest(request) => request.method.as_str(),
        }
    }
}

//*******************************//
//** ServerJsonrpcNotification **//
//*******************************//

/// "Similar to JsonrpcNotification , but with the variants restricted to server-side notifications."
#[derive(Clone, Debug, ::serde::Deserialize, ::serde::Serialize)]
#[serde(untagged)]
pub enum ServerJsonrpcNotification {
    CancelledNotification(CancelledNotification),
    ProgressNotification(ProgressNotification),
    ResourceListChangedNotification(ResourceListChangedNotification),
    ResourceUpdatedNotification(ResourceUpdatedNotification),
    PromptListChangedNotification(PromptListChangedNotification),
    ToolListChangedNotification(ToolListChangedNotification),
    TaskStatusNotification(TaskStatusNotification),
    LoggingMessageNotification(LoggingMessageNotification),
    ElicitationCompleteNotification(ElicitationCompleteNotification),
    CustomNotification(JsonrpcNotification),
}

impl From<ServerJsonrpcNotification> for NotificationFromServer {
    fn from(notification: ServerJsonrpcNotification) -> Self {
        match notification {
            ServerJsonrpcNotification::CancelledNotification(notification) => {
                Self::CancelledNotification(notification.params)
            }
            ServerJsonrpcNotification::ProgressNotification(notification) => Self::ProgressNotification(notification.params),
            ServerJsonrpcNotification::ResourceListChangedNotification(notification) => {
                Self::ResourceListChangedNotification(notification.params)
            }
            ServerJsonrpcNotification::ResourceUpdatedNotification(notification) => {
                Self::ResourceUpdatedNotification(notification.params)
            }
            ServerJsonrpcNotification::PromptListChangedNotification(notification) => {
                Self::PromptListChangedNotification(notification.params)
            }
            ServerJsonrpcNotification::ToolListChangedNotification(notification) => {
                Self::ToolListChangedNotification(notification.params)
            }
            ServerJsonrpcNotification::TaskStatusNotification(notification) => {
                Self::TaskStatusNotification(notification.params)
            }
            ServerJsonrpcNotification::LoggingMessageNotification(notification) => {
                Self::LoggingMessageNotification(notification.params)
            }
            ServerJsonrpcNotification::ElicitationCompleteNotification(notification) => {
                Self::ElicitationCompleteNotification(notification.params)
            }
            ServerJsonrpcNotification::CustomNotification(notification) => Self::CustomNotification(CustomNotification {
                method: notification.method,
                params: notification.params,
            }),
        }
    }
}

//TODO: check do we need from_message() or this
impl ServerJsonrpcNotification {
    pub fn new(notification: NotificationFromServer) -> Self {
        match notification {
            NotificationFromServer::CancelledNotification(params) => {
                Self::CancelledNotification(CancelledNotification::new(params))
            }
            NotificationFromServer::ProgressNotification(params) => {
                Self::ProgressNotification(ProgressNotification::new(params))
            }
            NotificationFromServer::ResourceListChangedNotification(params) => {
                Self::ResourceListChangedNotification(ResourceListChangedNotification::new(params))
            }
            NotificationFromServer::ResourceUpdatedNotification(params) => {
                Self::ResourceUpdatedNotification(ResourceUpdatedNotification::new(params))
            }
            NotificationFromServer::PromptListChangedNotification(params) => {
                Self::PromptListChangedNotification(PromptListChangedNotification::new(params))
            }
            NotificationFromServer::ToolListChangedNotification(params) => {
                Self::ToolListChangedNotification(ToolListChangedNotification::new(params))
            }
            NotificationFromServer::TaskStatusNotification(params) => {
                Self::TaskStatusNotification(TaskStatusNotification::new(params))
            }
            NotificationFromServer::LoggingMessageNotification(params) => {
                Self::LoggingMessageNotification(LoggingMessageNotification::new(params))
            }
            NotificationFromServer::ElicitationCompleteNotification(params) => {
                Self::ElicitationCompleteNotification(ElicitationCompleteNotification::new(params))
            }
            NotificationFromServer::CustomNotification(params) => {
                Self::CustomNotification(JsonrpcNotification::new(params.method, params.params))
            }
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        match self {
            ServerJsonrpcNotification::CancelledNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::ProgressNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::ResourceListChangedNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::ResourceUpdatedNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::PromptListChangedNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::ToolListChangedNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::TaskStatusNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::LoggingMessageNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::ElicitationCompleteNotification(notification) => notification.jsonrpc(),
            ServerJsonrpcNotification::CustomNotification(notification) => notification.jsonrpc(),
        }
    }
}

/// Formats the ServerJsonrpcNotification as a JSON string.
impl Display for ServerJsonrpcNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

impl FromStr for ServerJsonrpcNotification {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
//*******************************//
//**  NotificationFromServer   **//
//*******************************//

/// To determine standard and custom notifications received from the MCP Server
/// Custom notifications are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, ::serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum NotificationFromServer {
    CancelledNotification(CancelledNotificationParams),
    ProgressNotification(ProgressNotificationParams),
    ResourceListChangedNotification(Option<NotificationParams>),
    ResourceUpdatedNotification(ResourceUpdatedNotificationParams),
    PromptListChangedNotification(Option<NotificationParams>),
    ToolListChangedNotification(Option<NotificationParams>),
    TaskStatusNotification(TaskStatusNotificationParams),
    LoggingMessageNotification(LoggingMessageNotificationParams),
    ElicitationCompleteNotification(ElicitCompleteParams),
    CustomNotification(CustomNotification),
}

impl NotificationFromServer {
    pub fn method(&self) -> &str {
        match self {
            NotificationFromServer::CancelledNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::ProgressNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::ResourceListChangedNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::ResourceUpdatedNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::PromptListChangedNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::ToolListChangedNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::TaskStatusNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::LoggingMessageNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::ElicitationCompleteNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::CustomNotification(params) => params.method.as_str(),
        }
    }
}

//*******************************//
//**   ServerJsonrpcResponse   **//
//*******************************//

/// "Similar to JsonrpcResponse , but with the variants restricted to server-side responses."
#[derive(Clone, Debug)]
pub struct ServerJsonrpcResponse {
    pub id: RequestId,
    jsonrpc: ::std::string::String,
    pub result: ResultFromServer,
}

impl ServerJsonrpcResponse {
    pub fn new(id: RequestId, result: ResultFromServer) -> Self {
        Self {
            id,
            jsonrpc: JSONRPC_VERSION.to_string(),
            result,
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        &self.jsonrpc
    }
}

/// Formats the ServerJsonrpcResponse as a JSON string.
impl Display for ServerJsonrpcResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

impl FromStr for ServerJsonrpcResponse {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
//*******************************//
//**      ResultFromServer     **//
//*******************************//
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResultFromServer {
    InitializeResult(InitializeResult),
    ListResourcesResult(ListResourcesResult),
    ListResourceTemplatesResult(ListResourceTemplatesResult),
    ReadResourceResult(ReadResourceResult),
    ListPromptsResult(ListPromptsResult),
    GetPromptResult(GetPromptResult),
    ListToolsResult(ListToolsResult),
    CallToolResult(CallToolResult),
    GetTaskResult(GetTaskResult),
    CancelTaskResult(CancelTaskResult),
    ListTasksResult(ListTasksResult),
    CompleteResult(CompleteResult),
    CreateTaskResult(CreateTaskResult),
    Result(Result),
    GetTaskPayloadResult(GetTaskPayloadResult),
}

//***************************//
//** impl for JsonrpcErrorResponse **//
//***************************//

/// Formats the ServerJsonrpcResponse as a JSON string.
impl Display for JsonrpcErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

impl FromStr for JsonrpcErrorResponse {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

//**************************//
//**  MessageFromServer   **//
//**************************//

/// An enum representing various types of messages that can be sent from an MCP Server.
/// It provides a typed structure for the message payload while skipping internal details like
/// `requestId` and protocol version, which are used solely by the transport layer and
/// do not need to be exposed to the user.
#[derive(::serde::Serialize, ::serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum MessageFromServer {
    RequestFromServer(RequestFromServer),
    ResultFromServer(ResultFromServer),
    NotificationFromServer(NotificationFromServer),
    Error(RpcError),
}

impl From<RequestFromServer> for MessageFromServer {
    fn from(value: RequestFromServer) -> Self {
        Self::RequestFromServer(value)
    }
}

impl From<ResultFromServer> for MessageFromServer {
    fn from(value: ResultFromServer) -> Self {
        Self::ResultFromServer(value)
    }
}

impl From<NotificationFromServer> for MessageFromServer {
    fn from(value: NotificationFromServer) -> Self {
        Self::NotificationFromServer(value)
    }
}

impl From<RpcError> for MessageFromServer {
    fn from(value: RpcError) -> Self {
        Self::Error(value)
    }
}

impl McpMessage for MessageFromServer {
    fn is_response(&self) -> bool {
        matches!(self, MessageFromServer::ResultFromServer(_))
    }

    fn is_request(&self) -> bool {
        matches!(self, MessageFromServer::RequestFromServer(_))
    }

    fn is_notification(&self) -> bool {
        matches!(self, MessageFromServer::NotificationFromServer(_))
    }

    fn is_error(&self) -> bool {
        matches!(self, MessageFromServer::Error(_))
    }

    fn message_type(&self) -> MessageTypes {
        match self {
            MessageFromServer::RequestFromServer(_) => MessageTypes::Request,
            MessageFromServer::ResultFromServer(_) => MessageTypes::Response,
            MessageFromServer::NotificationFromServer(_) => MessageTypes::Notification,
            MessageFromServer::Error(_) => MessageTypes::Error,
        }
    }
}

impl FromMessage<MessageFromServer> for ServerMessage {
    fn from_message(message: MessageFromServer, request_id: Option<RequestId>) -> std::result::Result<Self, RpcError> {
        match message {
            MessageFromServer::RequestFromServer(request_from_server) => {
                let request_id =
                    request_id.ok_or_else(|| RpcError::internal_error().with_message("request_id is None!".to_string()))?;

                let rpc_message = match request_from_server {
                    RequestFromServer::PingRequest(params) => {
                        ServerJsonrpcRequest::PingRequest(PingRequest::new(request_id, params))
                    }
                    RequestFromServer::GetTaskRequest(params) => {
                        ServerJsonrpcRequest::GetTaskRequest(GetTaskRequest::new(request_id, params))
                    }
                    RequestFromServer::GetTaskPayloadRequest(params) => {
                        ServerJsonrpcRequest::GetTaskPayloadRequest(GetTaskPayloadRequest::new(request_id, params))
                    }
                    RequestFromServer::CancelTaskRequest(params) => {
                        ServerJsonrpcRequest::CancelTaskRequest(CancelTaskRequest::new(request_id, params))
                    }
                    RequestFromServer::ListTasksRequest(params) => {
                        ServerJsonrpcRequest::ListTasksRequest(ListTasksRequest::new(request_id, params))
                    }
                    RequestFromServer::CreateMessageRequest(params) => {
                        ServerJsonrpcRequest::CreateMessageRequest(CreateMessageRequest::new(request_id, params))
                    }
                    RequestFromServer::ListRootsRequest(params) => {
                        ServerJsonrpcRequest::ListRootsRequest(ListRootsRequest::new(request_id, params))
                    }
                    RequestFromServer::ElicitRequest(params) => {
                        ServerJsonrpcRequest::ElicitRequest(ElicitRequest::new(request_id, params))
                    }
                    RequestFromServer::CustomRequest(params) => {
                        ServerJsonrpcRequest::CustomRequest(JsonrpcRequest::new(request_id, params.method, params.params))
                    }
                };

                Ok(ServerMessage::Request(rpc_message))
            }
            MessageFromServer::ResultFromServer(result_from_server) => {
                let request_id =
                    request_id.ok_or_else(|| RpcError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
                    request_id,
                    result_from_server,
                )))
            }
            MessageFromServer::NotificationFromServer(notification_from_server) => {
                if request_id.is_some() {
                    return Err(RpcError::internal_error()
                        .with_message("request_id expected to be None for Notifications!".to_string()));
                }
                Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(
                    notification_from_server,
                )))
            }
            MessageFromServer::Error(jsonrpc_error_error) => Ok(ServerMessage::Error(JsonrpcErrorResponse::new(
                jsonrpc_error_error,
                request_id,
            ))),
        }
    }
}

//**************************//
//**  MessageFromClient   **//
//**************************//

/// An enum representing various types of messages that can be sent from an MCP Client.
/// It provides a typed structure for the message payload while skipping internal details like
/// `requestId` and protocol version, which are used solely by the transport layer and
/// do not need to be exposed to the user.
#[derive(::serde::Serialize, ::serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum MessageFromClient {
    RequestFromClient(RequestFromClient),
    ResultFromClient(ResultFromClient),
    NotificationFromClient(NotificationFromClient),
    Error(RpcError),
}

impl MessageFromClient {
    /// Returns `true` if the message is an `InitializeRequest`.
    pub fn is_initialize_request(&self) -> bool {
        matches!(self, Self::RequestFromClient(RequestFromClient::InitializeRequest(_)))
    }

    /// Returns `true` if the message is an `InitializedNotification`
    pub fn is_initialized_notification(&self) -> bool {
        matches!(
            self,
            Self::NotificationFromClient(NotificationFromClient::InitializedNotification(_))
        )
    }
}

impl From<RequestFromClient> for MessageFromClient {
    fn from(value: RequestFromClient) -> Self {
        Self::RequestFromClient(value)
    }
}

impl From<ResultFromClient> for MessageFromClient {
    fn from(value: ResultFromClient) -> Self {
        Self::ResultFromClient(value)
    }
}

impl From<NotificationFromClient> for MessageFromClient {
    fn from(value: NotificationFromClient) -> Self {
        Self::NotificationFromClient(value)
    }
}

impl From<RpcError> for MessageFromClient {
    fn from(value: RpcError) -> Self {
        Self::Error(value)
    }
}

impl McpMessage for MessageFromClient {
    fn is_response(&self) -> bool {
        matches!(self, MessageFromClient::ResultFromClient(_))
    }

    fn is_request(&self) -> bool {
        matches!(self, MessageFromClient::RequestFromClient(_))
    }

    fn is_notification(&self) -> bool {
        matches!(self, MessageFromClient::NotificationFromClient(_))
    }

    fn is_error(&self) -> bool {
        matches!(self, MessageFromClient::Error(_))
    }

    fn message_type(&self) -> MessageTypes {
        match self {
            MessageFromClient::RequestFromClient(_) => MessageTypes::Request,
            MessageFromClient::ResultFromClient(_) => MessageTypes::Response,
            MessageFromClient::NotificationFromClient(_) => MessageTypes::Notification,
            MessageFromClient::Error(_) => MessageTypes::Error,
        }
    }
}

impl FromMessage<MessageFromClient> for ClientMessage {
    fn from_message(message: MessageFromClient, request_id: Option<RequestId>) -> std::result::Result<Self, RpcError> {
        match message {
            MessageFromClient::RequestFromClient(request_from_client) => {
                let request_id =
                    request_id.ok_or_else(|| RpcError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ClientMessage::Request(ClientJsonrpcRequest::new(
                    request_id,
                    request_from_client,
                )))
            }
            MessageFromClient::ResultFromClient(result_from_client) => {
                let request_id =
                    request_id.ok_or_else(|| RpcError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ClientMessage::Response(ClientJsonrpcResponse::new(
                    request_id,
                    result_from_client,
                )))
            }
            MessageFromClient::NotificationFromClient(notification_from_client) => {
                if request_id.is_some() {
                    return Err(RpcError::internal_error()
                        .with_message("request_id expected to be None for Notifications!".to_string()));
                }

                Ok(ClientMessage::Notification(ClientJsonrpcNotification::new(
                    notification_from_client,
                )))
            }
            MessageFromClient::Error(jsonrpc_error_error) => Ok(ClientMessage::Error(JsonrpcErrorResponse::new(
                jsonrpc_error_error,
                request_id,
            ))),
        }
    }
}

//**************************//
//**  UnknownTool Error   **//
//**************************//

/// A custom error type `UnknownTool` that wraps a `String`.
/// This can be used as the error type in the result of a `CallToolRequest` when a non-existent or unimplemented tool is called.
#[derive(Debug)]
pub struct UnknownTool(pub String);

// Implement `Display` for `UnknownTool` to format the error message.
impl core::fmt::Display for UnknownTool {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The formatted string will display "Unknown tool: <tool_name>"
        write!(f, "Unknown tool: {}", self.0)
    }
}

// Implement the `Error` trait for `UnknownTool`, making it a valid error type.
impl std::error::Error for UnknownTool {}

//***************************//
//**  CallToolError Error  **//
//***************************//
/// A specific error type that can hold any kind of error and is used to
/// encapsulate various error scenarios when a `CallToolRequest` fails.
#[derive(Debug)]
pub struct CallToolError(pub Box<dyn std::error::Error>);

// Implement methods for `CallToolError` to handle different error types.
impl CallToolError {
    /// Constructor to create a new `CallToolError` from a generic error.
    pub fn new<E: std::error::Error + 'static>(err: E) -> Self {
        // Box the error to fit inside the `CallToolError` struct
        CallToolError(Box::new(err))
    }

    /// Specific constructor to create a `CallToolError` for an `UnknownTool` error.
    pub fn unknown_tool(tool_name: impl Into<String>) -> Self {
        // Create a `CallToolError` from an `UnknownTool` error (wrapped in a `Box`).
        CallToolError(Box::new(UnknownTool(tool_name.into())))
    }

    /// Creates a `CallToolError` indicating that task-augmented tool calls are not supported.
    /// This constructor is used when a task-augmented tool call is requested
    /// but the capability is not advertised by the peer.
    pub fn unsupported_task_augmented_tool_call() -> Self {
        Self::from_message("Task-augmented tool calls are not supported.".to_string())
    }

    /// Creates a `CallToolError` for invalid arguments with optional details.
    ///
    pub fn invalid_arguments(tool_name: impl AsRef<str>, message: Option<String>) -> Self {
        // Trim tool_name to remove whitespace and check for emptiness
        let tool_name = tool_name.as_ref().trim();
        if tool_name.is_empty() {
            return Self::from_message("Invalid arguments: tool name cannot be empty".to_string());
        }

        // Use a descriptive default message if none provided
        let default_message = "no additional details provided".to_string();
        let message = message.unwrap_or(default_message);

        // Format the full error message
        let full_message = format!("Invalid arguments for tool '{tool_name}': {message}");

        Self::from_message(full_message)
    }

    /// Creates a new `CallToolError` from a string message.
    ///
    /// This is useful for generating ad-hoc or one-off errors without defining a custom error type.
    /// Internally, it wraps the string in a lightweight error type that implements the `Error` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = rust_mcp_schema::schema_utils::CallToolError::from_message("Something went wrong");
    /// println!("{:?}", err);
    /// ```
    ///
    /// # Parameters
    ///
    /// - `message`: Any type that can be converted into a `String` (e.g., `&str` or `String`)
    ///
    /// # Returns
    ///
    /// A `CallToolError` wrapping a dynamic error created from the provided message.
    pub fn from_message(message: impl Into<String>) -> Self {
        struct MsgError(String);
        impl std::fmt::Debug for MsgError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl std::fmt::Display for MsgError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl std::error::Error for MsgError {}

        CallToolError::new(MsgError(message.into()))
    }
}

/// Converts a `CallToolError` into a `RpcError`.
///
/// The conversion creates an internal error variant of `RpcError`
/// and attaches the string representation of the original `CallToolError` as a message.
///
impl From<CallToolError> for RpcError {
    fn from(value: CallToolError) -> Self {
        Self::internal_error().with_message(value.to_string())
    }
}

/// Conversion of `CallToolError` into a `CallToolResult` with an error.
impl From<CallToolError> for CallToolResult {
    fn from(value: CallToolError) -> Self {
        // Convert `CallToolError` to a `CallToolResult`
        CallToolResult {
            content: vec![TextContent::new(value.to_string(), None, None).into()],
            is_error: Some(true),
            meta: None,
            structured_content: None,
        }
    }
}

// Implement `Display` for `CallToolError` to provide a user-friendly error message.
impl core::fmt::Display for CallToolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement `Error` for `CallToolError` to propagate the source of the error.
impl std::error::Error for CallToolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl CallToolRequest {
    /// Retrieves the name of the tool from the request parameters.
    ///
    /// This method provides access to the tool name stored within the `params` field
    /// of the `CallToolRequest` struct, returning it as a string reference.
    ///
    /// # Returns
    /// A reference to the string containing the tool's name.
    pub fn tool_name(&self) -> &str {
        &self.params.name
    }
}

impl<T: Into<String>> From<T> for TextContent {
    fn from(value: T) -> Self {
        TextContent::new(value.into(), None, None)
    }
}

impl TextResourceContents {
    pub fn new<T: Into<String>>(text: T, uri: T) -> Self {
        TextResourceContents {
            meta: None,
            mime_type: None,
            text: text.into(),
            uri: uri.into(),
        }
    }
    /// Assigns metadata to the TextResourceContents, enabling the inclusion of extra context or details.
    pub fn with_meta(mut self, meta: serde_json::Map<String, Value>) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn with_mime_type<T: Into<String>>(mut self, mime_type: T) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn with_uri<T: Into<String>>(mut self, uri: T) -> Self {
        self.uri = uri.into();
        self
    }
}

impl BlobResourceContents {
    pub fn new<T: Into<String>>(base64_text: T, uri: T) -> Self {
        BlobResourceContents {
            meta: None,
            mime_type: None,
            blob: base64_text.into(),
            uri: uri.into(),
        }
    }
    /// Assigns metadata to the BlobResourceContents, enabling the inclusion of extra context or details.
    pub fn with_meta(mut self, meta: serde_json::Map<String, Value>) -> Self {
        self.meta = Some(meta);
        self
    }
    pub fn with_mime_type<T: Into<String>>(mut self, mime_type: T) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }
    pub fn with_uri<T: Into<String>>(mut self, uri: T) -> Self {
        self.uri = uri.into();
        self
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum ClientMessages {
    Single(ClientMessage),
    Batch(Vec<ClientMessage>),
}

impl ClientMessages {
    pub fn is_batch(&self) -> bool {
        matches!(self, ClientMessages::Batch(_))
    }

    pub fn includes_request(&self) -> bool {
        match self {
            ClientMessages::Single(client_message) => client_message.is_request(),
            ClientMessages::Batch(client_messages) => client_messages.iter().any(ClientMessage::is_request),
        }
    }

    pub fn as_single(self) -> result::Result<ClientMessage, SdkError> {
        match self {
            ClientMessages::Single(client_message) => Ok(client_message),
            ClientMessages::Batch(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert ClientMessages::Batch to ClientMessage::Single")),
        }
    }
    pub fn as_batch(self) -> result::Result<Vec<ClientMessage>, SdkError> {
        match self {
            ClientMessages::Single(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert ClientMessage::Single to ClientMessages::Batch")),
            ClientMessages::Batch(client_messages) => Ok(client_messages),
        }
    }
}

impl From<ClientMessage> for ClientMessages {
    fn from(value: ClientMessage) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<ClientMessage>> for ClientMessages {
    fn from(value: Vec<ClientMessage>) -> Self {
        Self::Batch(value)
    }
}

impl Display for ClientMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum ServerMessages {
    Single(ServerMessage),
    Batch(Vec<ServerMessage>),
}

impl ServerMessages {
    pub fn is_batch(&self) -> bool {
        matches!(self, ServerMessages::Batch(_))
    }

    pub fn includes_request(&self) -> bool {
        match self {
            ServerMessages::Single(server_message) => server_message.is_request(),
            ServerMessages::Batch(server_messages) => server_messages.iter().any(ServerMessage::is_request),
        }
    }

    pub fn as_single(self) -> result::Result<ServerMessage, SdkError> {
        match self {
            ServerMessages::Single(server_message) => Ok(server_message),
            ServerMessages::Batch(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert ServerMessages::Batch to ServerMessage::Single")),
        }
    }
    pub fn as_batch(self) -> result::Result<Vec<ServerMessage>, SdkError> {
        match self {
            ServerMessages::Single(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert ServerMessage::Single to ServerMessages::Batch")),
            ServerMessages::Batch(server_messages) => Ok(server_messages),
        }
    }
}

impl From<ServerMessage> for ServerMessages {
    fn from(value: ServerMessage) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<ServerMessage>> for ServerMessages {
    fn from(value: Vec<ServerMessage>) -> Self {
        Self::Batch(value)
    }
}

impl Display for ServerMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum MessagesFromServer {
    Single(MessageFromServer),
    Batch(Vec<MessageFromServer>),
}

impl MessagesFromServer {
    pub fn is_batch(&self) -> bool {
        matches!(self, MessagesFromServer::Batch(_))
    }

    pub fn includes_request(&self) -> bool {
        match self {
            MessagesFromServer::Single(server_message) => server_message.is_request(),
            MessagesFromServer::Batch(server_messages) => server_messages.iter().any(MessageFromServer::is_request),
        }
    }

    pub fn as_single(self) -> result::Result<MessageFromServer, SdkError> {
        match self {
            MessagesFromServer::Single(server_message) => Ok(server_message),
            MessagesFromServer::Batch(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert MessagesFromServer::Batch to MessageFromServer::Single")),
        }
    }
    pub fn as_batch(self) -> result::Result<Vec<MessageFromServer>, SdkError> {
        match self {
            MessagesFromServer::Single(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert MessageFromServer::Single to MessagesFromServer::Batch")),
            MessagesFromServer::Batch(server_messages) => Ok(server_messages),
        }
    }
}

impl From<MessageFromServer> for MessagesFromServer {
    fn from(value: MessageFromServer) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<MessageFromServer>> for MessagesFromServer {
    fn from(value: Vec<MessageFromServer>) -> Self {
        Self::Batch(value)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum MessagesFromClient {
    Single(MessageFromClient),
    Batch(Vec<MessageFromClient>),
}

impl MessagesFromClient {
    pub fn is_batch(&self) -> bool {
        matches!(self, MessagesFromClient::Batch(_))
    }

    pub fn includes_request(&self) -> bool {
        match self {
            MessagesFromClient::Single(server_message) => server_message.is_request(),
            MessagesFromClient::Batch(server_messages) => server_messages.iter().any(MessageFromClient::is_request),
        }
    }

    pub fn as_single(self) -> result::Result<MessageFromClient, SdkError> {
        match self {
            MessagesFromClient::Single(server_message) => Ok(server_message),
            MessagesFromClient::Batch(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert MessagesFromClient::Batch to MessageFromClient::Single")),
        }
    }
    pub fn as_batch(self) -> result::Result<Vec<MessageFromClient>, SdkError> {
        match self {
            MessagesFromClient::Single(_) => Err(SdkError::internal_error()
                .with_message("Error: cannot convert MessageFromClient::Single to MessagesFromClient::Batch")),
            MessagesFromClient::Batch(server_messages) => Ok(server_messages),
        }
    }
}

impl From<MessageFromClient> for MessagesFromClient {
    fn from(value: MessageFromClient) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<MessageFromClient>> for MessagesFromClient {
    fn from(value: Vec<MessageFromClient>) -> Self {
        Self::Batch(value)
    }
}

#[derive(Debug)]
pub struct StringSchemaFormatError {
    invalid_value: String,
}

impl core::fmt::Display for StringSchemaFormatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid string schema format: '{}'", self.invalid_value)
    }
}

impl std::error::Error for StringSchemaFormatError {}

impl FromStr for StringSchemaFormat {
    type Err = StringSchemaFormatError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "date" => Ok(Self::Date),
            "date-time" => Ok(Self::DateTime),
            "email" => Ok(Self::Email),
            "uri" => Ok(Self::Uri),
            _ => Err(StringSchemaFormatError {
                invalid_value: s.to_string(),
            }),
        }
    }
}

// Helper: handle all single-select enum variants
fn try_from_enum_schema(map: &serde_json::Map<String, Value>) -> result::Result<PrimitiveSchemaDefinition, RpcError> {
    // All enum schemas should have type: "string" (or missing, but usually present)
    let has_one_of = map.contains_key("oneOf");
    let has_enum = map.contains_key("enum");
    let has_enum_names = map.contains_key("enumNames");

    if has_one_of {
        let schema: TitledSingleSelectEnumSchema = serde_json::from_value(Value::Object(map.clone())).map_err(|e| {
            RpcError::parse_error().with_message(format!("Failed to parse TitledSingleSelectEnumSchema: {e}"))
        })?;

        Ok(PrimitiveSchemaDefinition::TitledSingleSelectEnumSchema(schema))
    } else if has_enum && has_enum_names {
        let schema: LegacyTitledEnumSchema = serde_json::from_value(Value::Object(map.clone()))
            .map_err(|e| RpcError::parse_error().with_message(format!("Failed to parse LegacyTitledEnumSchema: {e}")))?;
        Ok(PrimitiveSchemaDefinition::LegacyTitledEnumSchema(schema))
    } else if has_enum {
        let schema: UntitledSingleSelectEnumSchema = serde_json::from_value(Value::Object(map.clone())).map_err(|e| {
            RpcError::parse_error().with_message(format!("Failed to parse UntitledSingleSelectEnumSchema: {e}"))
        })?;
        Ok(PrimitiveSchemaDefinition::UntitledSingleSelectEnumSchema(schema))
    } else {
        Err(RpcError::parse_error().with_message("Invalid enum schema: missing 'enum' or 'oneOf'".to_string()))
    }
}

// Helper: handle multi-select (array) enum schemas
fn try_from_multi_select_schema(
    map: &serde_json::Map<String, Value>,
) -> result::Result<PrimitiveSchemaDefinition, RpcError> {
    let items = map
        .get("items")
        .ok_or(RpcError::parse_error().with_message("Array schema missing 'items' field".to_string()))?;

    let items_obj = items
        .as_object()
        .ok_or(RpcError::parse_error().with_message("Field 'items' must be an object".to_string()))?;

    if items_obj.contains_key("anyOf") {
        let schema: TitledMultiSelectEnumSchema = serde_json::from_value(Value::Object(map.clone())).map_err(|e| {
            RpcError::parse_error().with_message(format!("Failed to parse TitledMultiSelectEnumSchema: {e}"))
        })?;
        Ok(PrimitiveSchemaDefinition::TitledMultiSelectEnumSchema(schema))
    } else if items_obj.contains_key("enum") {
        let schema: UntitledMultiSelectEnumSchema = serde_json::from_value(Value::Object(map.clone())).map_err(|e| {
            RpcError::parse_error().with_message(format!("Failed to parse UntitledMultiSelectEnumSchema: {e}"))
        })?;
        Ok(PrimitiveSchemaDefinition::UntitledMultiSelectEnumSchema(schema))
    } else {
        Err(RpcError::parse_error()
            .with_message("Array schema 'items' must contain 'enum' or 'oneOf' to be a multi-select enum".to_string()))
    }
}

impl TryFrom<&serde_json::Map<String, Value>> for PrimitiveSchemaDefinition {
    type Error = RpcError;

    fn try_from(value: &serde_json::Map<String, serde_json::Value>) -> result::Result<Self, Self::Error> {
        // 1. First: detect enum schemas (they look like strings but have enum/oneOf)
        if value.contains_key("enum") || value.contains_key("oneOf") {
            return try_from_enum_schema(value);
        }

        // 2. Then: detect multi-select array schemas (type: "array" + items with enum/oneOf)
        if value.get("type").and_then(|v| v.as_str()) == Some("array") {
            return try_from_multi_select_schema(value);
        }

        let input_type = value
            .get("type")
            .and_then(|v| v.as_str())
            .or_else(|| value.get("oneOf").map(|_| "enum")) // if "oneOf" exists, return "enum"
            .ok_or_else(|| {
                RpcError::parse_error().with_message("'type' is missing and data type is not supported!".to_string())
            })?;

        let description = value.get("description").and_then(|v| v.as_str().map(|s| s.to_string()));
        let title = value.get("title").and_then(|v| v.as_str().map(|s| s.to_string()));

        let schema_definition: PrimitiveSchemaDefinition = match input_type {
            "string" => {
                let max_length = value.get("maxLength").and_then(|v| v.as_number().and_then(|n| n.as_i64()));
                let min_length = value.get("minLength").and_then(|v| v.as_number().and_then(|n| n.as_i64()));
                let default = value.get("default").and_then(|v| v.as_str().map(|s| s.to_string()));

                let format_str = value.get("format").and_then(|v| v.as_str());
                let format = format_str.and_then(|s| StringSchemaFormat::from_str(s).ok());

                PrimitiveSchemaDefinition::StringSchema(StringSchema::new(
                    default,
                    description,
                    format,
                    max_length,
                    min_length,
                    title,
                ))
            }
            "number" | "integer" => {
                let maximum = value.get("maximum").and_then(|v| v.as_number().and_then(|n| n.as_i64()));
                let minimum = value.get("minimum").and_then(|v| v.as_number().and_then(|n| n.as_i64()));
                let default = value.get("default").and_then(|v| v.as_number().and_then(|n| n.as_i64()));

                PrimitiveSchemaDefinition::NumberSchema(NumberSchema {
                    default,
                    description,
                    maximum,
                    minimum,
                    title,
                    type_: if input_type == "integer" {
                        NumberSchemaType::Integer
                    } else {
                        NumberSchemaType::Number
                    },
                })
            }
            "boolean" => {
                let default = value.get("default").and_then(|v| v.as_bool().map(|s| s.to_owned()));
                PrimitiveSchemaDefinition::BooleanSchema(BooleanSchema::new(default, description, title))
            }
            other => {
                return Err(RpcError::parse_error().with_message(format!("'{other}' type is not currently supported")));
            }
        };

        Ok(schema_definition)
    }
}

impl RequestFromServer {
    pub fn is_task_augmented(&self) -> bool {
        match self {
            RequestFromServer::CreateMessageRequest(create_message_request_params) => {
                create_message_request_params.is_task_augmented()
            }
            RequestFromServer::ElicitRequest(elicit_request_params) => elicit_request_params.is_task_augmented(),
            _ => false,
        }
    }
}

impl MessageFromServer {
    pub fn is_task_augmented(&self) -> bool {
        match self {
            MessageFromServer::RequestFromServer(request_from_server) => request_from_server.is_task_augmented(),
            _ => false,
        }
    }
}

impl CallToolRequest {
    pub fn is_task_augmented(&self) -> bool {
        self.params.is_task_augmented()
    }
}

impl CallToolRequestParams {
    pub fn is_task_augmented(&self) -> bool {
        self.task.is_some()
    }
}

impl CreateMessageRequestParams {
    pub fn is_task_augmented(&self) -> bool {
        self.task.is_some()
    }
}

impl ElicitRequestParams {
    pub fn is_task_augmented(&self) -> bool {
        match self {
            ElicitRequestParams::UrlParams(elicit_request_url_params) => elicit_request_url_params.task.is_some(),
            ElicitRequestParams::FormParams(elicit_request_form_params) => elicit_request_form_params.task.is_some(),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ElicitRequestParams::UrlParams(elicit_request_url_params) => elicit_request_url_params.message.as_str(),
            ElicitRequestParams::FormParams(elicit_request_form_params) => elicit_request_form_params.message.as_str(),
        }
    }

    /// Set task metadata , requesting task-augmented execution for this request
    pub fn with_task(mut self, task: TaskMetadata) -> Self {
        match &mut self {
            ElicitRequestParams::UrlParams(params) => {
                params.task = Some(task);
            }
            ElicitRequestParams::FormParams(params) => {
                params.task = Some(task);
            }
        }
        self
    }
}

impl ElicitRequestUrlParams {
    /// Set task metadata , requesting task-augmented execution for this request
    pub fn with_task(mut self, task: TaskMetadata) -> Self {
        self.task = Some(task);
        self
    }
}

impl ElicitRequestFormParams {
    /// Set task metadata , requesting task-augmented execution for this request
    pub fn with_task(mut self, task: TaskMetadata) -> Self {
        self.task = Some(task);
        self
    }
}

impl ServerCapabilities {
    /// Returns `true` if the server supports listing tasks.
    ///
    /// This is determined by whether the `list` capability is present.
    pub fn can_list_tasks(&self) -> bool {
        self.tasks.as_ref().is_some_and(|tasks| tasks.can_list_tasks())
    }

    /// Returns `true` if the server supports canceling tasks.
    ///
    /// This is determined by whether the `cancel` capability is present.
    pub fn can_cancel_tasks(&self) -> bool {
        self.tasks.as_ref().is_some_and(|tasks| tasks.can_cancel_tasks())
    }

    /// Returns `true` if the server supports task-augmented tools/call requests
    pub fn can_run_task_augmented_tools(&self) -> bool {
        self.tasks.as_ref().is_some_and(|tasks| tasks.can_run_task_augmented_tools())
    }

    pub fn can_handle_request(&self, client_request: &ClientJsonrpcRequest) -> std::result::Result<(), RpcError> {
        let request_method = client_request.method();

        // Helper function for creating error messages
        fn create_error(capability: &str, method: &str) -> RpcError {
            RpcError::internal_error().with_message(create_unsupported_capability_message("Server", capability, method))
        }

        match client_request {
            ClientJsonrpcRequest::SetLevelRequest(_) if self.logging.is_none() => {
                return Err(create_error("logging", request_method));
            }
            ClientJsonrpcRequest::GetPromptRequest(_) | ClientJsonrpcRequest::ListPromptsRequest(_)
                if self.prompts.is_none() =>
            {
                return Err(create_error("prompts", request_method));
            }

            ClientJsonrpcRequest::ListResourcesRequest(_)
            | ClientJsonrpcRequest::ListResourceTemplatesRequest(_)
            | ClientJsonrpcRequest::ReadResourceRequest(_)
            | ClientJsonrpcRequest::SubscribeRequest(_)
            | ClientJsonrpcRequest::UnsubscribeRequest(_)
                if self.resources.is_none() =>
            {
                return Err(create_error("resources", request_method));
            }

            ClientJsonrpcRequest::CallToolRequest(call_tool_request)
                if call_tool_request.is_task_augmented() && !self.can_run_task_augmented_tools() =>
            {
                return Err(create_error("Task-augmented tool call", request_method));
            }

            ClientJsonrpcRequest::CallToolRequest(_) | ClientJsonrpcRequest::ListToolsRequest(_) if self.tools.is_none() => {
                return Err(create_error("tools", request_method));
            }
            ClientJsonrpcRequest::CompleteRequest(_) if self.completions.is_none() => {
                return Err(create_error("completions", request_method));
            }

            ClientJsonrpcRequest::GetTaskRequest(_)
            | ClientJsonrpcRequest::GetTaskPayloadRequest(_)
            | ClientJsonrpcRequest::CancelTaskRequest(_)
            | ClientJsonrpcRequest::ListTasksRequest(_)
                if self.tasks.is_none() =>
            {
                return Err(create_error("task", request_method));
            }
            ClientJsonrpcRequest::ListTasksRequest(_) if !self.can_list_tasks() => {
                return Err(create_error("listing tasks", request_method));
            }
            ClientJsonrpcRequest::CancelTaskRequest(_) if !self.can_cancel_tasks() => {
                return Err(create_error("task cancellation", request_method));
            }
            _ => {}
        };
        Ok(())
    }

    /// Asserts that the server supports the requested notification.
    ///
    /// Verifies that the server advertises support for the notification type,
    /// allowing callers to avoid sending notifications that the server does not
    /// support. This can be used to prevent issuing requests to peers that lack
    /// the required capability.
    pub fn can_accept_notification(&self, notification_method: &str) -> std::result::Result<(), RpcError> {
        let entity = "Server";

        if LoggingMessageNotification::method_value().eq(notification_method) && self.logging.is_none() {
            return Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                entity,
                "logging",
                notification_method,
            )));
        }

        if [
            ResourceUpdatedNotification::method_value(),
            ResourceListChangedNotification::method_value(),
        ]
        .contains(&notification_method)
            && self.resources.is_none()
        {
            return Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                entity,
                "notifying about resources",
                notification_method,
            )));
        }

        if ToolListChangedNotification::method_value().eq(notification_method) && self.tools.is_none() {
            return Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                entity,
                "notifying of tool list changes",
                notification_method,
            )));
        }

        if PromptListChangedNotification::method_value().eq(notification_method) && self.prompts.is_none() {
            return Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                entity,
                "notifying of prompt list changes",
                notification_method,
            )));
        }

        Ok(())
    }
}

impl ServerTasks {
    /// Returns `true` if the server supports listing tasks.
    ///
    /// This is determined by whether the `list` capability is present.
    pub fn can_list_tasks(&self) -> bool {
        self.list.is_some()
    }

    /// Returns `true` if the server supports canceling tasks.
    ///
    /// This is determined by whether the `cancel` capability is present.
    pub fn can_cancel_tasks(&self) -> bool {
        self.cancel.is_some()
    }

    /// Returns `true` if the server supports task-augmented tools/call requests
    pub fn can_run_task_augmented_tools(&self) -> bool {
        if let Some(requests) = self.requests.as_ref() {
            if let Some(tools) = requests.tools.as_ref() {
                return tools.call.is_some();
            }
        }
        false
    }
}

/// Formats an assertion error message for unsupported capabilities.
///
/// Constructs a string describing that a specific entity (e.g., server or client) lacks
/// support for a required capability, needed for a particular method.
///
/// # Arguments
/// - `entity`: The name of the entity (e.g., "Server" or "Client") that lacks support.
/// - `capability`: The name of the unsupported capability or tool.
/// - `method_name`: The name of the method requiring the capability.
///
/// # Returns
/// A formatted string detailing the unsupported capability error.
///
/// # Examples
/// ```ignore
/// let msg = create_unsupported_capability_message("Server", "tools", rust_mcp_schema::ListResourcesRequest::method_value());
/// assert_eq!(msg, "Server does not support resources (required for resources/list)");
/// ```
fn create_unsupported_capability_message(entity: &str, capability: &str, method_name: &str) -> String {
    format!("{entity} does not support {capability} (required for {method_name})")
}

impl ClientCapabilities {
    /// Returns `true` if the server supports listing tasks.
    ///
    /// This is determined by whether the `list` capability is present.
    pub fn can_list_tasks(&self) -> bool {
        self.tasks.as_ref().is_some_and(|task| task.can_list_tasks())
    }

    /// Returns `true` if the server supports canceling tasks.
    ///
    /// This is determined by whether the `cancel` capability is present.
    pub fn can_cancel_tasks(&self) -> bool {
        self.tasks.as_ref().is_some_and(|task| task.can_cancel_tasks())
    }

    /// Returns `true` if the client can request elicitation.
    pub fn can_accept_elicitation_task(&self) -> bool {
        self.tasks.as_ref().is_some_and(|task| task.can_accept_elicitation_task())
    }

    /// Returns `true` if the client can request message sampling.
    pub fn can_accept_sampling_task(&self) -> bool {
        self.tasks.as_ref().is_some_and(|task| task.can_accept_sampling_task())
    }

    pub fn can_handle_request(&self, server_jsonrpc_request: &ServerJsonrpcRequest) -> std::result::Result<(), RpcError> {
        let request_method = server_jsonrpc_request.method();

        // Helper function for creating error messages
        fn create_error(capability: &str, method: &str) -> RpcError {
            RpcError::internal_error().with_message(create_unsupported_capability_message("Client", capability, method))
        }

        match server_jsonrpc_request {
            ServerJsonrpcRequest::CreateMessageRequest(create_message_request) => {
                match self.sampling.as_ref() {
                    Some(samplig_capabilities) => {
                        //  include_context requested but not supported
                        if create_message_request.params.include_context.is_some() && samplig_capabilities.context.is_none()
                        {
                            return Err(create_error("context inclusion", request_method));
                        }

                        if create_message_request.params.tool_choice.is_some() && samplig_capabilities.tools.is_none() {
                            return Err(create_error("tool choice", request_method));
                        }
                    }
                    None => {
                        return Err(create_error("sampling capability", request_method));
                    }
                }

                if create_message_request.params.is_task_augmented() && !self.can_accept_sampling_task() {
                    return Err(create_error("sampling task", request_method));
                }
            }
            ServerJsonrpcRequest::ListRootsRequest(_) => {
                if self.roots.is_none() {
                    return Err(create_error("roots capability", request_method));
                }
            }
            ServerJsonrpcRequest::GetTaskRequest(_) | ServerJsonrpcRequest::GetTaskPayloadRequest(_) => {
                if self.tasks.is_none() {
                    return Err(create_error("Task", request_method));
                }
            }
            ServerJsonrpcRequest::CancelTaskRequest(_) => {
                if let Some(tasks) = self.tasks.as_ref() {
                    if !tasks.can_cancel_tasks() {
                        return Err(create_error("task cancellation", request_method));
                    }
                }
            }
            ServerJsonrpcRequest::ListTasksRequest(_) => {
                if let Some(tasks) = self.tasks.as_ref() {
                    if !tasks.can_list_tasks() {
                        return Err(create_error("listing tasks", request_method));
                    }
                }
            }

            ServerJsonrpcRequest::ElicitRequest(elicit_request) => {
                if self.elicitation.is_none() {
                    return Err(create_error("input elicitation", request_method));
                }

                if elicit_request.params.is_task_augmented() && !self.can_accept_elicitation_task() {
                    return Err(create_error("elicitation task", request_method));
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn can_accept_notification(&self, notification_method: &str) -> std::result::Result<(), RpcError> {
        let entity = "Client";

        if RootsListChangedNotification::method_value().eq(notification_method) && self.roots.is_none() {
            return Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                entity,
                "roots list changed notifications",
                notification_method,
            )));
        }

        Ok(())
    }
}

impl ClientTasks {
    /// Returns `true` if the server supports listing tasks.
    ///
    /// This is determined by whether the `list` capability is present.
    pub fn can_list_tasks(&self) -> bool {
        self.list.is_some()
    }

    /// Returns `true` if the server supports canceling tasks.
    ///
    /// This is determined by whether the `cancel` capability is present.
    pub fn can_cancel_tasks(&self) -> bool {
        self.cancel.is_some()
    }

    /// Returns `true` if the client can request elicitation.
    pub fn can_accept_elicitation_task(&self) -> bool {
        if let Some(requests) = self.requests.as_ref() {
            if let Some(elicitation) = requests.elicitation.as_ref() {
                return elicitation.create.is_some();
            }
        }
        false
    }

    /// Returns `true` if the client can request message sampling.
    pub fn can_accept_sampling_task(&self) -> bool {
        if let Some(requests) = self.requests.as_ref() {
            if let Some(sampling) = requests.sampling.as_ref() {
                return sampling.create_message.is_some();
            }
        }
        false
    }
}

impl From<JsonrpcRequest> for CustomRequest {
    fn from(request: JsonrpcRequest) -> Self {
        Self {
            method: request.method,
            params: request.params,
        }
    }
}

impl From<JsonrpcNotification> for CustomNotification {
    fn from(notification: JsonrpcNotification) -> Self {
        Self {
            method: notification.method,
            params: notification.params,
        }
    }
}

/// Returns `true` if the task is in a terminal state.
/// Terminal states are states where the task has finished and will not change anymore.
impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        match self {
            TaskStatus::Cancelled | TaskStatus::Completed | TaskStatus::Failed => true,
            TaskStatus::InputRequired | TaskStatus::Working => false,
        }
    }
}

/// Returns `true` if the task is in a terminal state.
/// Terminal states are states where the task has finished and will not change anymore.
impl Task {
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}

impl GetTaskResult {
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}

impl GetTaskPayloadResult {
    /// Retrieves the related task ID from the metadata, if it exists.
    ///
    /// This function looks for a key corresponding to the `RELATED_TASK_META_KEY` in the
    /// `meta` field of the struct. If the key exists and contains a string value, it returns
    /// it as an `Option<&str>`. If the key is missing or not a string, it returns `None`.
    ///
    /// # Returns
    /// - `Some(&str)` if a related task ID exists.
    /// - `None` if no related task ID is found.
    pub fn related_task_id(&self) -> Option<&str> {
        self.meta
            .as_ref()
            .and_then(|v| v.get(RELATED_TASK_META_KEY))
            .and_then(|v| v.as_str())
    }

    /// Sets the related task ID in the metadata.
    ///
    /// This function inserts a `taskId` key with the provided `task_id` into the `meta` field.
    /// If the `meta` field is `None`, it creates a new `serde_json::Map` and assigns it to `meta`.
    /// The `task_id` is converted into a string before being inserted.
    ///
    /// # Type Parameters
    /// - `T`: The type of the `task_id`. It must implement `Into<String>` to allow flexible
    ///   conversion of various types (e.g., `&str`, `String`).
    ///
    /// # Arguments
    /// - `task_id`: The ID of the related task to set. This can be any type that can be converted
    ///   into a `String`.
    pub fn set_related_task_id<T>(&mut self, task_id: T)
    where
        T: Into<String>,
    {
        let task_json = json!({ "taskId": task_id.into() });

        if let Some(meta) = &mut self.meta {
            meta.insert(RELATED_TASK_META_KEY.into(), task_json);
        } else {
            let mut new_meta = serde_json::Map::new();
            new_meta.insert(RELATED_TASK_META_KEY.into(), task_json);
            self.meta = Some(new_meta);
        }
    }
}

pub trait McpMetaEx {
    /// Retrieves the related task ID from the metadata, if it exists.
    ///
    /// This function looks for a key corresponding to the `RELATED_TASK_META_KEY` in the
    /// `meta` field of the struct. If the key exists and contains a string value, it returns
    /// it as an `Option<&str>`. If the key is missing or not a string, it returns `None`.
    ///
    /// # Returns
    /// - `Some(&str)` if a related task ID exists.
    /// - `None` if no related task ID is found.
    fn related_task_id(&self) -> Option<&str>;
    /// Sets the related task ID in the metadata.
    ///
    /// This function inserts a `taskId` key with the provided `task_id` into the `meta` field.
    /// If the `meta` field is `None`, it creates a new `serde_json::Map` and assigns it to `meta`.
    /// The `task_id` is converted into a string before being inserted.
    ///
    /// # Type Parameters
    /// - `T`: The type of the `task_id`. It must implement `Into<String>` to allow flexible
    ///   conversion of various types (e.g., `&str`, `String`).
    ///
    /// # Arguments
    /// - `task_id`: The ID of the related task to set. This can be any type that can be converted
    ///   into a `String`.
    fn set_related_task_id<T>(&mut self, task_id: T)
    where
        T: Into<String>;

    fn with_related_task_id<T>(self, task_id: T) -> Self
    where
        T: Into<String>;

    /// Add a key-value pair.
    /// Value can be anything that serde can serialize (primitives, vecs, maps, structs, etc.).
    fn add<K, V>(self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>;

    /// Conditionally add a field (useful for optional metadata).
    fn add_if<K, V>(self, condition: bool, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>;

    /// Add a raw pre-built Value (e.g. from json! macro or complex logic).
    fn add_raw<K>(self, key: K, value: Value) -> Self
    where
        K: Into<String>;
}

impl McpMetaEx for serde_json::Map<String, Value> {
    fn related_task_id(&self) -> Option<&str> {
        self.get(RELATED_TASK_META_KEY).and_then(|v| v.as_str())
    }

    fn set_related_task_id<T>(&mut self, task_id: T)
    where
        T: Into<String>,
    {
        let task_json = json!({ "taskId": task_id.into() });
        self.entry(RELATED_TASK_META_KEY)
            .and_modify(|e| *e = task_json.clone())
            .or_insert_with(|| task_json);
    }

    fn with_related_task_id<T>(mut self, task_id: T) -> Self
    where
        T: Into<String>,
    {
        self.set_related_task_id(task_id);
        self
    }

    /// Add a key-value pair.
    /// Value can be anything that serde can serialize (primitives, vecs, maps, structs, etc.).
    fn add<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        self.insert(key.into(), value.into());
        self
    }

    /// Conditionally add a field (useful for optional metadata).
    fn add_if<K, V>(mut self, condition: bool, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
    {
        if condition {
            self.insert(key.into(), value.into());
            self
        } else {
            self
        }
    }

    /// Add a raw pre-built Value (e.g. from json! macro or complex logic).
    fn add_raw<K>(mut self, key: K, value: Value) -> Self
    where
        K: Into<String>,
    {
        self.insert(key.into(), value);
        self
    }
}

impl FromStr for Role {
    type Err = RpcError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "assistant" => Ok(Role::Assistant),
            "user" => Ok(Role::User),
            _ => {
                Err(RpcError::parse_error()
                    .with_message(format!("Invalid role '{s}'. Expected one of: 'assistant', 'user'")))
            }
        }
    }
}

pub type CustomNotification = CustomRequest;

/// BEGIN AUTO GENERATED
impl ::serde::Serialize for ServerJsonrpcResponse {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_struct("JsonrpcResponse", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("jsonrpc", &self.jsonrpc)?;
        state.serialize_field("result", &self.result)?;
        state.end()
    }
}
impl<'de> ::serde::Deserialize<'de> for ServerJsonrpcResponse {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        struct ServerJsonrpcResultVisitor;
        impl<'de> Visitor<'de> for ServerJsonrpcResultVisitor {
            type Value = ServerJsonrpcResponse;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid JSON-RPC response object")
            }
            fn visit_map<M>(self, mut map: M) -> std::result::Result<ServerJsonrpcResponse, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut id: Option<RequestId> = None;
                let mut jsonrpc: Option<String> = None;
                let mut result: Option<Value> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "jsonrpc" => jsonrpc = Some(map.next_value()?),
                        "result" => result = Some(map.next_value()?),
                        _ => {
                            return Err(de::Error::unknown_field(&key, &["id", "jsonrpc", "result"]));
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                let result = result.ok_or_else(|| de::Error::missing_field("result"))?;
                let result = serde_json::from_value::<ResultFromServer>(result).map_err(de::Error::custom)?;
                Ok(ServerJsonrpcResponse { id, jsonrpc, result })
            }
        }
        deserializer.deserialize_struct("JsonrpcResponse", &["id", "jsonrpc", "result"], ServerJsonrpcResultVisitor)
    }
}
impl ::serde::Serialize for ClientJsonrpcResponse {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_struct("JsonrpcResponse", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("jsonrpc", &self.jsonrpc)?;
        state.serialize_field("result", &self.result)?;
        state.end()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClientJsonrpcResponse {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        struct ClientJsonrpcResultVisitor;
        impl<'de> Visitor<'de> for ClientJsonrpcResultVisitor {
            type Value = ClientJsonrpcResponse;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid JSON-RPC response object")
            }
            fn visit_map<M>(self, mut map: M) -> std::result::Result<ClientJsonrpcResponse, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut id: Option<RequestId> = None;
                let mut jsonrpc: Option<String> = None;
                let mut result: Option<Value> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "jsonrpc" => jsonrpc = Some(map.next_value()?),
                        "result" => result = Some(map.next_value()?),
                        _ => {
                            return Err(de::Error::unknown_field(&key, &["id", "jsonrpc", "result"]));
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                let result = result.ok_or_else(|| de::Error::missing_field("result"))?;
                let result = serde_json::from_value::<ResultFromClient>(result).map_err(de::Error::custom)?;
                Ok(ClientJsonrpcResponse { id, jsonrpc, result })
            }
        }
        deserializer.deserialize_struct("JsonrpcResponse", &["id", "jsonrpc", "result"], ClientJsonrpcResultVisitor)
    }
}
impl From<Result> for ResultFromClient {
    fn from(value: Result) -> Self {
        Self::Result(value)
    }
}
impl From<GetTaskResult> for ResultFromClient {
    fn from(value: GetTaskResult) -> Self {
        Self::GetTaskResult(value)
    }
}
impl From<GetTaskPayloadResult> for ResultFromClient {
    fn from(value: GetTaskPayloadResult) -> Self {
        Self::GetTaskPayloadResult(value)
    }
}
impl From<CancelTaskResult> for ResultFromClient {
    fn from(value: CancelTaskResult) -> Self {
        Self::CancelTaskResult(value)
    }
}
impl From<ListTasksResult> for ResultFromClient {
    fn from(value: ListTasksResult) -> Self {
        Self::ListTasksResult(value)
    }
}
impl From<CreateMessageResult> for ResultFromClient {
    fn from(value: CreateMessageResult) -> Self {
        Self::CreateMessageResult(value)
    }
}
impl From<ListRootsResult> for ResultFromClient {
    fn from(value: ListRootsResult) -> Self {
        Self::ListRootsResult(value)
    }
}
impl From<ElicitResult> for ResultFromClient {
    fn from(value: ElicitResult) -> Self {
        Self::ElicitResult(value)
    }
}
impl From<CreateTaskResult> for ResultFromClient {
    fn from(value: CreateTaskResult) -> Self {
        Self::CreateTaskResult(value)
    }
}
impl From<Result> for MessageFromClient {
    fn from(value: Result) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<GetTaskResult> for MessageFromClient {
    fn from(value: GetTaskResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<GetTaskPayloadResult> for MessageFromClient {
    fn from(value: GetTaskPayloadResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<CancelTaskResult> for MessageFromClient {
    fn from(value: CancelTaskResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<ListTasksResult> for MessageFromClient {
    fn from(value: ListTasksResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<CreateMessageResult> for MessageFromClient {
    fn from(value: CreateMessageResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<ListRootsResult> for MessageFromClient {
    fn from(value: ListRootsResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<ElicitResult> for MessageFromClient {
    fn from(value: ElicitResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<CreateTaskResult> for MessageFromClient {
    fn from(value: CreateTaskResult) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<PingRequest> for ServerJsonrpcRequest {
    fn from(value: PingRequest) -> Self {
        Self::PingRequest(value)
    }
}
impl From<GetTaskRequest> for ServerJsonrpcRequest {
    fn from(value: GetTaskRequest) -> Self {
        Self::GetTaskRequest(value)
    }
}
impl From<GetTaskPayloadRequest> for ServerJsonrpcRequest {
    fn from(value: GetTaskPayloadRequest) -> Self {
        Self::GetTaskPayloadRequest(value)
    }
}
impl From<CancelTaskRequest> for ServerJsonrpcRequest {
    fn from(value: CancelTaskRequest) -> Self {
        Self::CancelTaskRequest(value)
    }
}
impl From<ListTasksRequest> for ServerJsonrpcRequest {
    fn from(value: ListTasksRequest) -> Self {
        Self::ListTasksRequest(value)
    }
}
impl From<CreateMessageRequest> for ServerJsonrpcRequest {
    fn from(value: CreateMessageRequest) -> Self {
        Self::CreateMessageRequest(value)
    }
}
impl From<ListRootsRequest> for ServerJsonrpcRequest {
    fn from(value: ListRootsRequest) -> Self {
        Self::ListRootsRequest(value)
    }
}
impl From<ElicitRequest> for ServerJsonrpcRequest {
    fn from(value: ElicitRequest) -> Self {
        Self::ElicitRequest(value)
    }
}
impl From<InitializeRequest> for ClientJsonrpcRequest {
    fn from(value: InitializeRequest) -> Self {
        Self::InitializeRequest(value)
    }
}
impl From<PingRequest> for ClientJsonrpcRequest {
    fn from(value: PingRequest) -> Self {
        Self::PingRequest(value)
    }
}
impl From<ListResourcesRequest> for ClientJsonrpcRequest {
    fn from(value: ListResourcesRequest) -> Self {
        Self::ListResourcesRequest(value)
    }
}
impl From<ListResourceTemplatesRequest> for ClientJsonrpcRequest {
    fn from(value: ListResourceTemplatesRequest) -> Self {
        Self::ListResourceTemplatesRequest(value)
    }
}
impl From<ReadResourceRequest> for ClientJsonrpcRequest {
    fn from(value: ReadResourceRequest) -> Self {
        Self::ReadResourceRequest(value)
    }
}
impl From<SubscribeRequest> for ClientJsonrpcRequest {
    fn from(value: SubscribeRequest) -> Self {
        Self::SubscribeRequest(value)
    }
}
impl From<UnsubscribeRequest> for ClientJsonrpcRequest {
    fn from(value: UnsubscribeRequest) -> Self {
        Self::UnsubscribeRequest(value)
    }
}
impl From<ListPromptsRequest> for ClientJsonrpcRequest {
    fn from(value: ListPromptsRequest) -> Self {
        Self::ListPromptsRequest(value)
    }
}
impl From<GetPromptRequest> for ClientJsonrpcRequest {
    fn from(value: GetPromptRequest) -> Self {
        Self::GetPromptRequest(value)
    }
}
impl From<ListToolsRequest> for ClientJsonrpcRequest {
    fn from(value: ListToolsRequest) -> Self {
        Self::ListToolsRequest(value)
    }
}
impl From<CallToolRequest> for ClientJsonrpcRequest {
    fn from(value: CallToolRequest) -> Self {
        Self::CallToolRequest(value)
    }
}
impl From<GetTaskRequest> for ClientJsonrpcRequest {
    fn from(value: GetTaskRequest) -> Self {
        Self::GetTaskRequest(value)
    }
}
impl From<GetTaskPayloadRequest> for ClientJsonrpcRequest {
    fn from(value: GetTaskPayloadRequest) -> Self {
        Self::GetTaskPayloadRequest(value)
    }
}
impl From<CancelTaskRequest> for ClientJsonrpcRequest {
    fn from(value: CancelTaskRequest) -> Self {
        Self::CancelTaskRequest(value)
    }
}
impl From<ListTasksRequest> for ClientJsonrpcRequest {
    fn from(value: ListTasksRequest) -> Self {
        Self::ListTasksRequest(value)
    }
}
impl From<SetLevelRequest> for ClientJsonrpcRequest {
    fn from(value: SetLevelRequest) -> Self {
        Self::SetLevelRequest(value)
    }
}
impl From<CompleteRequest> for ClientJsonrpcRequest {
    fn from(value: CompleteRequest) -> Self {
        Self::CompleteRequest(value)
    }
}
/// Enum representing SDK error codes.
#[allow(non_camel_case_types)]
pub enum SdkErrorCodes {
    CONNECTION_CLOSED = -32000,
    REQUEST_TIMEOUT = -32001,
    RESOURCE_NOT_FOUND = -32002,
    BAD_REQUEST = -32015,
    SESSION_NOT_FOUND = -32016,
    INVALID_REQUEST = -32600,
    METHOD_NOT_FOUND = -32601,
    INVALID_PARAMS = -32602,
    INTERNAL_ERROR = -32603,
    PARSE_ERROR = -32700,
    URL_ELICITATION_REQUIRED = -32042,
}
impl core::fmt::Display for SdkErrorCodes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SdkErrorCodes::CONNECTION_CLOSED => write!(f, "Connection closed"),
            SdkErrorCodes::REQUEST_TIMEOUT => write!(f, "Request timeout"),
            SdkErrorCodes::INVALID_REQUEST => write!(f, "Invalid request"),
            SdkErrorCodes::METHOD_NOT_FOUND => write!(f, "Method not found"),
            SdkErrorCodes::INVALID_PARAMS => write!(f, "Invalid params"),
            SdkErrorCodes::INTERNAL_ERROR => write!(f, "Internal error"),
            SdkErrorCodes::PARSE_ERROR => write!(f, "Parse Error"),
            SdkErrorCodes::RESOURCE_NOT_FOUND => write!(f, "Resource not found"),
            SdkErrorCodes::BAD_REQUEST => write!(f, "Bad request"),
            SdkErrorCodes::SESSION_NOT_FOUND => write!(f, "Session not found"),
            SdkErrorCodes::URL_ELICITATION_REQUIRED => {
                write!(
                    f,
                    "A required URL was not provided. Please supply the requested URL to continue."
                )
            }
        }
    }
}
impl From<SdkErrorCodes> for i64 {
    fn from(code: SdkErrorCodes) -> Self {
        code as i64
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
pub struct SdkError {
    ///The error type that occurred.
    pub code: i64,
    ///Additional information about the error.
    pub data: ::std::option::Option<::serde_json::Value>,
    ///A short description of the error.
    pub message: ::std::string::String,
}
impl core::fmt::Display for SdkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MCP error {}: {}", self.code, self.message)
    }
}
impl std::error::Error for SdkError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl SdkError {
    pub fn new(
        error_code: SdkErrorCodes,
        message: ::std::string::String,
        data: ::std::option::Option<::serde_json::Value>,
    ) -> Self {
        Self {
            code: error_code.into(),
            data,
            message,
        }
    }
    pub fn connection_closed() -> Self {
        Self {
            code: SdkErrorCodes::CONNECTION_CLOSED.into(),
            data: None,
            message: SdkErrorCodes::CONNECTION_CLOSED.to_string(),
        }
    }
    pub fn request_timeout(timeout: u128) -> Self {
        Self {
            code: SdkErrorCodes::REQUEST_TIMEOUT.into(),
            data: Some(json!({ "timeout" : timeout })),
            message: SdkErrorCodes::REQUEST_TIMEOUT.to_string(),
        }
    }
    pub fn session_not_found() -> Self {
        Self {
            code: SdkErrorCodes::SESSION_NOT_FOUND.into(),
            data: None,
            message: SdkErrorCodes::SESSION_NOT_FOUND.to_string(),
        }
    }
    pub fn invalid_request() -> Self {
        Self {
            code: SdkErrorCodes::INVALID_REQUEST.into(),
            data: None,
            message: SdkErrorCodes::INVALID_REQUEST.to_string(),
        }
    }
    pub fn method_not_found() -> Self {
        Self {
            code: SdkErrorCodes::METHOD_NOT_FOUND.into(),
            data: None,
            message: SdkErrorCodes::METHOD_NOT_FOUND.to_string(),
        }
    }
    pub fn invalid_params() -> Self {
        Self {
            code: SdkErrorCodes::INVALID_PARAMS.into(),
            data: None,
            message: SdkErrorCodes::INVALID_PARAMS.to_string(),
        }
    }
    pub fn internal_error() -> Self {
        Self {
            code: SdkErrorCodes::INTERNAL_ERROR.into(),
            data: None,
            message: SdkErrorCodes::INTERNAL_ERROR.to_string(),
        }
    }
    pub fn parse_error() -> Self {
        Self {
            code: SdkErrorCodes::PARSE_ERROR.into(),
            data: None,
            message: SdkErrorCodes::PARSE_ERROR.to_string(),
        }
    }
    /// Creates a new `RpcError` indicating that a URL elicitation failed
    /// and was required for the operation to continue.
    ///
    /// The resulting error uses the -32042 value as introduced in mcp protocol 2025-11-25
    /// The result json matches a UrlElicitError and the `data` value could be deserialized into UrlElicitErrorData
    ///
    pub fn url_elicit_required(elicit_url_params: Vec<ElicitRequestUrlParams>) -> Self {
        Self {
            code: UrlElicitError::code_value(),
            data: Some(serde_json::to_value(elicit_url_params).unwrap_or_else(|_| {
                json!(
                    { "elicitations" : [], "error" :
                    "failed to UrlElicitError data" }
                )
            })),
            message: "URL required. Please provide a URL.".to_string(),
        }
    }
    pub fn resource_not_found() -> Self {
        Self {
            code: SdkErrorCodes::RESOURCE_NOT_FOUND.into(),
            data: None,
            message: SdkErrorCodes::RESOURCE_NOT_FOUND.to_string(),
        }
    }
    pub fn bad_request() -> Self {
        Self {
            code: SdkErrorCodes::BAD_REQUEST.into(),
            data: None,
            message: SdkErrorCodes::RESOURCE_NOT_FOUND.to_string(),
        }
    }
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }
    pub fn with_data(mut self, data: ::std::option::Option<::serde_json::Value>) -> Self {
        self.data = data;
        self
    }
}
/// Enum representing standard and mcp specific JSON-RPC error codes.
#[allow(non_camel_case_types)]
pub enum RpcErrorCodes {
    PARSE_ERROR = -32700isize,
    INVALID_REQUEST = -32600isize,
    METHOD_NOT_FOUND = -32601isize,
    INVALID_PARAMS = -32602isize,
    INTERNAL_ERROR = -32603isize,
    URL_ELICITATION_REQUIRED = -32042isize,
}
impl From<RpcErrorCodes> for i64 {
    fn from(code: RpcErrorCodes) -> Self {
        code as i64
    }
}
impl RpcError {
    /// Constructs a new `RpcError` with the provided arguments.
    ///
    /// # Arguments
    /// * `error_code` - The JSON-RPC error code.
    /// * `message` - A descriptive error message.
    /// * `data` - Optional additional data.
    ///
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use rust_mcp_schema::{RpcError, schema_utils::RpcErrorCodes};
    ///
    /// let error = RpcError::new(RpcErrorCodes::INVALID_PARAMS, "Invalid params!".to_string(), Some(json!({"details": "Missing method field"})));
    /// assert_eq!(error.code, -32602);
    /// assert_eq!(error.message, "Invalid params!".to_string());
    /// ```
    pub fn new(
        error_code: RpcErrorCodes,
        message: ::std::string::String,
        data: ::std::option::Option<::serde_json::Value>,
    ) -> Self {
        Self {
            code: error_code.into(),
            data,
            message,
        }
    }
    /// Creates a new `RpcError` for "Method not found".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::RpcError;
    ///
    /// let error = RpcError::method_not_found();
    /// assert_eq!(error.code, -32601);
    /// assert_eq!(error.message, "Method not found");
    /// ```
    pub fn method_not_found() -> Self {
        Self {
            code: RpcErrorCodes::METHOD_NOT_FOUND.into(),
            data: None,
            message: "Method not found".to_string(),
        }
    }
    /// Creates a new `RpcError` indicating that a URL elicitation failed
    /// and was required for the operation to continue.
    ///
    /// The resulting error uses the -32042 value as introduced in mcp protocol 2025-11-25
    /// The result json matches a UrlElicitError and the `data` value could be deserialized into UrlElicitErrorData
    ///
    pub fn url_elicit_required(elicit_url_params: Vec<ElicitRequestUrlParams>) -> Self {
        Self {
            code: UrlElicitError::code_value(),
            data: Some(serde_json::to_value(elicit_url_params).unwrap_or_else(|_| {
                json!(
                    { "elicitations" : [], "error" :
                    "failed to UrlElicitError data" }
                )
            })),
            message: "URL required. Please provide a URL.".to_string(),
        }
    }
    /// Creates a new `RpcError` for "Invalid parameters".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::RpcError;
    ///
    /// let error = RpcError::invalid_params();
    /// assert_eq!(error.code, -32602);
    /// ```
    pub fn invalid_params() -> Self {
        Self {
            code: RpcErrorCodes::INVALID_PARAMS.into(),
            data: None,
            message: "Invalid params".to_string(),
        }
    }
    /// Creates a new `RpcError` for "Invalid request".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::RpcError;
    ///
    /// let error = RpcError::invalid_request();
    /// assert_eq!(error.code, -32600);
    /// ```
    pub fn invalid_request() -> Self {
        Self {
            code: RpcErrorCodes::INVALID_REQUEST.into(),
            data: None,
            message: "Invalid request".to_string(),
        }
    }
    /// Creates a new `RpcError` for "Internal error".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::RpcError;
    ///
    /// let error = RpcError::internal_error();
    /// assert_eq!(error.code, -32603);
    /// ```
    pub fn internal_error() -> Self {
        Self {
            code: RpcErrorCodes::INTERNAL_ERROR.into(),
            data: None,
            message: "Internal error".to_string(),
        }
    }
    /// Creates a new `RpcError` for "Parse error".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::RpcError;
    ///
    /// let error = RpcError::parse_error();
    /// assert_eq!(error.code, -32700);
    /// ```
    pub fn parse_error() -> Self {
        Self {
            code: RpcErrorCodes::PARSE_ERROR.into(),
            data: None,
            message: "Parse error".to_string(),
        }
    }
    /// Sets a custom error message.
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::RpcError;
    ///
    /// let error = RpcError::invalid_request().with_message("Request format is invalid".to_string());
    /// assert_eq!(error.message, "Request format is invalid".to_string());
    /// ```
    pub fn with_message<T: Into<String>>(mut self, message: T) -> Self {
        self.message = message.into();
        self
    }
    /// Attaches optional data to the error.
    ///
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use rust_mcp_schema::RpcError;
    ///
    /// let error = RpcError::invalid_request().with_data(Some(json!({"reason": "Missing ID"})));
    /// assert!(error.data.is_some());
    /// ```
    pub fn with_data(mut self, data: ::std::option::Option<::serde_json::Value>) -> Self {
        self.data = data;
        self
    }
}
impl std::error::Error for RpcError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {err}"))
        )
    }
}
impl FromStr for RpcError {
    type Err = RpcError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| RpcError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
///Constructs a new `JsonrpcErrorResponse` using the provided arguments.
impl JsonrpcErrorResponse {
    pub fn create(
        id: Option<RequestId>,
        error_code: RpcErrorCodes,
        error_message: ::std::string::String,
        error_data: ::std::option::Option<::serde_json::Value>,
    ) -> Self {
        Self::new(RpcError::new(error_code, error_message, error_data), id)
    }
}
impl From<Result> for ResultFromServer {
    fn from(value: Result) -> Self {
        Self::Result(value)
    }
}
impl From<InitializeResult> for ResultFromServer {
    fn from(value: InitializeResult) -> Self {
        Self::InitializeResult(value)
    }
}
impl From<ListResourcesResult> for ResultFromServer {
    fn from(value: ListResourcesResult) -> Self {
        Self::ListResourcesResult(value)
    }
}
impl From<ListResourceTemplatesResult> for ResultFromServer {
    fn from(value: ListResourceTemplatesResult) -> Self {
        Self::ListResourceTemplatesResult(value)
    }
}
impl From<ReadResourceResult> for ResultFromServer {
    fn from(value: ReadResourceResult) -> Self {
        Self::ReadResourceResult(value)
    }
}
impl From<ListPromptsResult> for ResultFromServer {
    fn from(value: ListPromptsResult) -> Self {
        Self::ListPromptsResult(value)
    }
}
impl From<GetPromptResult> for ResultFromServer {
    fn from(value: GetPromptResult) -> Self {
        Self::GetPromptResult(value)
    }
}
impl From<ListToolsResult> for ResultFromServer {
    fn from(value: ListToolsResult) -> Self {
        Self::ListToolsResult(value)
    }
}
impl From<CallToolResult> for ResultFromServer {
    fn from(value: CallToolResult) -> Self {
        Self::CallToolResult(value)
    }
}
impl From<GetTaskResult> for ResultFromServer {
    fn from(value: GetTaskResult) -> Self {
        Self::GetTaskResult(value)
    }
}
impl From<GetTaskPayloadResult> for ResultFromServer {
    fn from(value: GetTaskPayloadResult) -> Self {
        Self::GetTaskPayloadResult(value)
    }
}
impl From<CancelTaskResult> for ResultFromServer {
    fn from(value: CancelTaskResult) -> Self {
        Self::CancelTaskResult(value)
    }
}
impl From<ListTasksResult> for ResultFromServer {
    fn from(value: ListTasksResult) -> Self {
        Self::ListTasksResult(value)
    }
}
impl From<CompleteResult> for ResultFromServer {
    fn from(value: CompleteResult) -> Self {
        Self::CompleteResult(value)
    }
}
impl From<CreateTaskResult> for ResultFromServer {
    fn from(value: CreateTaskResult) -> Self {
        Self::CreateTaskResult(value)
    }
}
impl From<Result> for MessageFromServer {
    fn from(value: Result) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<InitializeResult> for MessageFromServer {
    fn from(value: InitializeResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<ListResourcesResult> for MessageFromServer {
    fn from(value: ListResourcesResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<ListResourceTemplatesResult> for MessageFromServer {
    fn from(value: ListResourceTemplatesResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<ReadResourceResult> for MessageFromServer {
    fn from(value: ReadResourceResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<ListPromptsResult> for MessageFromServer {
    fn from(value: ListPromptsResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<GetPromptResult> for MessageFromServer {
    fn from(value: GetPromptResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<ListToolsResult> for MessageFromServer {
    fn from(value: ListToolsResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<CallToolResult> for MessageFromServer {
    fn from(value: CallToolResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<GetTaskResult> for MessageFromServer {
    fn from(value: GetTaskResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<GetTaskPayloadResult> for MessageFromServer {
    fn from(value: GetTaskPayloadResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<CancelTaskResult> for MessageFromServer {
    fn from(value: CancelTaskResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<ListTasksResult> for MessageFromServer {
    fn from(value: ListTasksResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<CompleteResult> for MessageFromServer {
    fn from(value: CompleteResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl From<CreateTaskResult> for MessageFromServer {
    fn from(value: CreateTaskResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl TryFrom<ResultFromClient> for GenericResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        match value {
            ResultFromClient::GetTaskPayloadResult(result) => Ok(result.into()),
            ResultFromClient::Result(result) => Ok(result),
            _ => Err(RpcError::internal_error().with_message("Not a Result".to_string())),
        }
    }
}
impl TryFrom<ResultFromClient> for GetTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::GetTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for GetTaskPayloadResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::GetTaskPayloadResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskPayloadResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for CancelTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::CancelTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CancelTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for ListTasksResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::ListTasksResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListTasksResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for CreateMessageResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::CreateMessageResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CreateMessageResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for ListRootsResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::ListRootsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListRootsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for ElicitResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::ElicitResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ElicitResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for CreateTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ResultFromClient::CreateTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CreateTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GenericResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        match value {
            ResultFromServer::GetTaskPayloadResult(result) => Ok(result.into()),
            ResultFromServer::Result(result) => Ok(result),
            _ => Err(RpcError::internal_error().with_message("Not a Result".to_string())),
        }
    }
}
impl TryFrom<ResultFromServer> for InitializeResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::InitializeResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a InitializeResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListResourcesResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::ListResourcesResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListResourcesResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListResourceTemplatesResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::ListResourceTemplatesResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListResourceTemplatesResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ReadResourceResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::ReadResourceResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ReadResourceResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListPromptsResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::ListPromptsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListPromptsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GetPromptResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::GetPromptResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetPromptResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListToolsResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::ListToolsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListToolsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CallToolResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::CallToolResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CallToolResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GetTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::GetTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GetTaskPayloadResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::GetTaskPayloadResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskPayloadResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CancelTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::CancelTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CancelTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListTasksResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::ListTasksResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListTasksResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CompleteResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::CompleteResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CompleteResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CreateTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ResultFromServer::CreateTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CreateTaskResult".to_string()))
        }
    }
}
impl ContentBlock {
    ///Create a ContentBlock::TextContent
    pub fn text_content(text: ::std::string::String) -> Self {
        TextContent::new(text, None, None).into()
    }
    ///Create a ContentBlock::ImageContent
    pub fn image_content(data: ::std::string::String, mime_type: ::std::string::String) -> Self {
        ImageContent::new(data, mime_type, None, None).into()
    }
    ///Create a ContentBlock::AudioContent
    pub fn audio_content(data: ::std::string::String, mime_type: ::std::string::String) -> Self {
        AudioContent::new(data, mime_type, None, None).into()
    }
    ///Create a ContentBlock::ResourceLink
    pub fn resource_link(value: ResourceLink) -> Self {
        value.into()
    }
    ///Create a ContentBlock::EmbeddedResource
    pub fn embedded_resource(resource: EmbeddedResourceResource) -> Self {
        EmbeddedResource::new(resource, None, None).into()
    }
    ///Returns the content type as a string based on the variant of `ContentBlock`
    pub fn content_type(&self) -> &str {
        match self {
            ContentBlock::TextContent(text_content) => text_content.type_(),
            ContentBlock::ImageContent(image_content) => image_content.type_(),
            ContentBlock::AudioContent(audio_content) => audio_content.type_(),
            ContentBlock::ResourceLink(resource_link) => resource_link.type_(),
            ContentBlock::EmbeddedResource(embedded_resource) => embedded_resource.type_(),
        }
    }
    /// Converts the content to a reference to `TextContent`, returning an error if the conversion is invalid.
    pub fn as_text_content(&self) -> std::result::Result<&TextContent, RpcError> {
        match &self {
            ContentBlock::TextContent(text_content) => Ok(text_content),
            _ => Err(RpcError::internal_error().with_message(format!(
                "Invalid conversion, \"{}\" is not a {}",
                self.content_type(),
                "TextContent"
            ))),
        }
    }
    /// Converts the content to a reference to `TextContent`, returning an error if the conversion is invalid.
    pub fn as_image_content(&self) -> std::result::Result<&ImageContent, RpcError> {
        match &self {
            ContentBlock::ImageContent(image_content) => Ok(image_content),
            _ => Err(RpcError::internal_error().with_message(format!(
                "Invalid conversion, \"{}\" is not a {}",
                self.content_type(),
                "ImageContent"
            ))),
        }
    }
    /// Converts the content to a reference to `TextContent`, returning an error if the conversion is invalid.
    pub fn as_audio_content(&self) -> std::result::Result<&AudioContent, RpcError> {
        match &self {
            ContentBlock::AudioContent(audio_content) => Ok(audio_content),
            _ => Err(RpcError::internal_error().with_message(format!(
                "Invalid conversion, \"{}\" is not a {}",
                self.content_type(),
                "AudioContent"
            ))),
        }
    }
    /// Converts the content to a reference to `TextContent`, returning an error if the conversion is invalid.
    pub fn as_resource_link(&self) -> std::result::Result<&ResourceLink, RpcError> {
        match &self {
            ContentBlock::ResourceLink(resource_link) => Ok(resource_link),
            _ => Err(RpcError::internal_error().with_message(format!(
                "Invalid conversion, \"{}\" is not a {}",
                self.content_type(),
                "ResourceLink"
            ))),
        }
    }
    /// Converts the content to a reference to `TextContent`, returning an error if the conversion is invalid.
    pub fn as_embedded_resource(&self) -> std::result::Result<&EmbeddedResource, RpcError> {
        match &self {
            ContentBlock::EmbeddedResource(embedded_resource) => Ok(embedded_resource),
            _ => Err(RpcError::internal_error().with_message(format!(
                "Invalid conversion, \"{}\" is not a {}",
                self.content_type(),
                "EmbeddedResource"
            ))),
        }
    }
}
impl CallToolResult {
    pub fn text_content(content: Vec<TextContent>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn image_content(content: Vec<ImageContent>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn audio_content(content: Vec<AudioContent>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn resource_link(content: Vec<ResourceLink>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn embedded_resource(content: Vec<EmbeddedResource>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    /// Create a `CallToolResult` with an error, containing an error message in the content
    pub fn with_error(error: CallToolError) -> Self {
        Self {
            content: vec![ContentBlock::TextContent(TextContent::new(error.to_string(), None, None))],
            is_error: Some(true),
            meta: None,
            structured_content: None,
        }
    }
    /// Assigns metadata to the CallToolResult, enabling the inclusion of extra context or details.
    pub fn with_meta(mut self, meta: Option<serde_json::Map<String, Value>>) -> Self {
        self.meta = meta;
        self
    }
    /// Assigns structured_content to the CallToolResult
    pub fn with_structured_content(
        mut self,
        structured_content: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    ) -> Self {
        self.structured_content = Some(structured_content);
        self
    }
    /// Creates a new instance using the provided content blocks.
    ///
    /// This method initializes the structure with the given `content` and
    /// resets all optional fields (`is_error`, `meta`, and `structured_content`)
    /// to `None`.
    ///
    /// # Arguments
    ///
    /// * `content` - A vector of `ContentBlock` values to populate the instance.
    ///
    /// # Returns
    ///
    /// Returns a new instance containing the supplied content.
    pub fn from_content(content: Vec<ContentBlock>) -> Self {
        Self {
            content,
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    /// Adds a single content block to the instance.
    ///
    /// This method appends the provided `content` to the existing `content` vector
    /// and returns the updated instance, enabling a builder-style pattern.
    pub fn add_content(mut self, content: ContentBlock) -> Self {
        self.content.push(content);
        self
    }
}
impl ServerRequest {
    pub fn request_id(&self) -> &RequestId {
        match self {
            ServerRequest::PingRequest(request) => &request.id,
            ServerRequest::GetTaskRequest(request) => &request.id,
            ServerRequest::GetTaskPayloadRequest(request) => &request.id,
            ServerRequest::CancelTaskRequest(request) => &request.id,
            ServerRequest::ListTasksRequest(request) => &request.id,
            ServerRequest::CreateMessageRequest(request) => &request.id,
            ServerRequest::ListRootsRequest(request) => &request.id,
            ServerRequest::ElicitRequest(request) => &request.id,
        }
    }
}
impl ClientRequest {
    pub fn request_id(&self) -> &RequestId {
        match self {
            ClientRequest::InitializeRequest(request) => &request.id,
            ClientRequest::PingRequest(request) => &request.id,
            ClientRequest::ListResourcesRequest(request) => &request.id,
            ClientRequest::ListResourceTemplatesRequest(request) => &request.id,
            ClientRequest::ReadResourceRequest(request) => &request.id,
            ClientRequest::SubscribeRequest(request) => &request.id,
            ClientRequest::UnsubscribeRequest(request) => &request.id,
            ClientRequest::ListPromptsRequest(request) => &request.id,
            ClientRequest::GetPromptRequest(request) => &request.id,
            ClientRequest::ListToolsRequest(request) => &request.id,
            ClientRequest::CallToolRequest(request) => &request.id,
            ClientRequest::GetTaskRequest(request) => &request.id,
            ClientRequest::GetTaskPayloadRequest(request) => &request.id,
            ClientRequest::CancelTaskRequest(request) => &request.id,
            ClientRequest::ListTasksRequest(request) => &request.id,
            ClientRequest::SetLevelRequest(request) => &request.id,
            ClientRequest::CompleteRequest(request) => &request.id,
        }
    }
}
impl From<&str> for IconTheme {
    fn from(s: &str) -> Self {
        match s {
            "dark" => Self::Dark,
            "light" => Self::Light,
            _ => Self::Light,
        }
    }
}
impl From<&str> for ElicitResultContent {
    fn from(value: &str) -> Self {
        Self::Primitive(ElicitResultContentPrimitive::String(value.to_string()))
    }
}
impl From<&str> for ElicitResultContentPrimitive {
    fn from(value: &str) -> Self {
        ElicitResultContentPrimitive::String(value.to_string())
    }
}
impl From<String> for ElicitResultContentPrimitive {
    fn from(value: String) -> Self {
        ElicitResultContentPrimitive::String(value)
    }
}
impl From<String> for ElicitResultContent {
    fn from(value: String) -> Self {
        Self::Primitive(ElicitResultContentPrimitive::String(value))
    }
}
impl From<Vec<&str>> for ElicitResultContent {
    fn from(value: Vec<&str>) -> Self {
        Self::StringArray(value.iter().map(|v| v.to_string()).collect())
    }
}
impl From<i64> for ElicitResultContent {
    fn from(value: i64) -> Self {
        Self::Primitive(value.into())
    }
}
impl CallToolRequestParams {
    pub fn new<T>(tool_name: T) -> Self
    where
        T: ToString,
    {
        Self {
            name: tool_name.to_string(),
            arguments: None,
            meta: None,
            task: None,
        }
    }
    /// Sets the arguments for the tool call.
    pub fn with_arguments(mut self, arguments: serde_json::Map<String, Value>) -> Self {
        self.arguments = Some(arguments);
        self
    }
    /// Assigns metadata to the CallToolRequestParams, enabling the inclusion of extra context or details.
    pub fn with_meta(mut self, meta: CallToolMeta) -> Self {
        self.meta = Some(meta);
        self
    }
    /// Set task metadata , requesting task-augmented execution for this request
    pub fn with_task(mut self, task: TaskMetadata) -> Self {
        self.task = Some(task);
        self
    }
}
/// END AUTO GENERATED
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_detect_message_type() {
        // standard request
        let message = ClientJsonrpcRequest::new(RequestId::Integer(0), RequestFromClient::PingRequest(None));
        let result = detect_message_type(&json!(message));
        assert!(matches!(result, MessageTypes::Request));

        // custom request

        let result = detect_message_type(&json!({
        "id":0,
        "method":"add_numbers",
        "params":{},
        "jsonrpc":"2.0"
        }));
        assert!(matches!(result, MessageTypes::Request));

        // standard notification
        let message = ClientJsonrpcNotification::new(NotificationFromClient::RootsListChangedNotification(None));
        let result = detect_message_type(&json!(message));
        assert!(matches!(result, MessageTypes::Notification));

        // custom notification
        let result = detect_message_type(&json!({
            "method":"notifications/email_sent",
            "jsonrpc":"2.0"
        }));
        assert!(matches!(result, MessageTypes::Notification));

        // standard response
        let message = ClientJsonrpcResponse::new(
            RequestId::Integer(0),
            ListRootsResult {
                meta: None,
                roots: vec![],
            }
            .into(),
        );
        let result = detect_message_type(&json!(message));
        assert!(matches!(result, MessageTypes::Response));

        //custom response

        let result = detect_message_type(&json!({
            "id":1,
            "jsonrpc":"2.0",
            "result":"{}",
        }));
        assert!(matches!(result, MessageTypes::Response));

        // error message
        let message = JsonrpcErrorResponse::create(
            Some(RequestId::Integer(0)),
            RpcErrorCodes::INVALID_PARAMS,
            "Invalid params!".to_string(),
            None,
        );
        let result = detect_message_type(&json!(message));
        assert!(matches!(result, MessageTypes::Error));

        // default
        let result = detect_message_type(&json!({}));
        assert!(matches!(result, MessageTypes::Request));
    }
}
