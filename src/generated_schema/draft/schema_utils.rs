use crate::generated_schema::mcp_draft::*;

use serde::ser::SerializeStruct;
use serde_json::{json, Value};
use std::hash::{Hash, Hasher};
use std::result;
use std::{fmt::Display, str::FromStr};

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

pub type ResultFromClient = ClientResult;

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
    // pub fn new(id: RequestId, request: RequestFromServer) -> Self {
    //     let method = request.method().to_string();
    //     Self {
    //         id,
    //         jsonrpc: JSONRPC_VERSION.to_string(),
    //         method,
    //         request,
    //     }
    // }

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
pub type ResultFromServer = ServerResult;

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
    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
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
impl TryFrom<ResultFromClient> for GenericResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        match value {
            ClientResult::GetTaskPayloadResult(result) => Ok(result.into()),
            ClientResult::Result(result) => Ok(result),
            _ => Err(RpcError::internal_error().with_message("Not a Result".to_string())),
        }
    }
}
impl TryFrom<ResultFromClient> for GetTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ClientResult::GetTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for GetTaskPayloadResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ClientResult::GetTaskPayloadResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskPayloadResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for CancelTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ClientResult::CancelTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CancelTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for ListTasksResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ClientResult::ListTasksResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListTasksResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for CreateMessageResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ClientResult::CreateMessageResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CreateMessageResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for ListRootsResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ClientResult::ListRootsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListRootsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for ElicitResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        if let ClientResult::ElicitResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ElicitResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GenericResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        match value {
            ServerResult::GetTaskPayloadResult(result) => Ok(result.into()),
            ServerResult::Result(result) => Ok(result),
            _ => Err(RpcError::internal_error().with_message("Not a Result".to_string())),
        }
    }
}
impl TryFrom<ResultFromServer> for InitializeResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::InitializeResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a InitializeResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListResourcesResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListResourcesResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListResourcesResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListResourceTemplatesResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListResourceTemplatesResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListResourceTemplatesResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ReadResourceResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ReadResourceResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ReadResourceResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListPromptsResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListPromptsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListPromptsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GetPromptResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::GetPromptResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetPromptResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListToolsResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListToolsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListToolsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CallToolResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::CallToolResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CallToolResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GetTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::GetTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GetTaskPayloadResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::GetTaskPayloadResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetTaskPayloadResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CancelTaskResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::CancelTaskResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CancelTaskResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListTasksResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListTasksResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListTasksResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CompleteResult {
    type Error = RpcError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::CompleteResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CompleteResult".to_string()))
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
