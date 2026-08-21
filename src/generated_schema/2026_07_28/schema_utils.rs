use crate::generated_schema::*;

use serde::ser::SerializeStruct;
use serde_json::{json, Value};
use std::hash::{Hash, Hasher};
use std::result;
use std::{fmt::Display, str::FromStr};

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

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
    fn method(&self) -> Option<&str>;
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

impl core::fmt::Display for RequestId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            RequestId::String(ref s) => write!(f, "{}", s),
            RequestId::Integer(i) => write!(f, "{}", i),
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
    /// Note: InitializeRequest has been removed from the 2026-07-28 protocol.
    /// This always returns `false`.
    pub fn is_initialize_request(&self) -> bool {
        false
    }

    /// Returns `true` if the message is an `InitializedNotification`
    /// Note: InitializedNotification has been removed from the 2026-07-28 protocol.
    /// This always returns `false`.
    pub fn is_initialized_notification(&self) -> bool {
        false
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
                ClientJsonrpcRequest::Custom(request) => Some(&request.id),
                _ => Some(client_jsonrpc_request.request_id()),
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

     fn method(&self) -> Option<&str> {
        match self {
            ClientMessage::Request(client_jsonrpc_request) => Some(client_jsonrpc_request.method()),
            ClientMessage::Notification(client_jsonrpc_notification) => Some(client_jsonrpc_notification.method()),
            ClientMessage::Response(_) => None,
            ClientMessage::Error(_) => None,
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
    Standard(ClientRequest),
    Custom(JsonrpcRequest),
}

impl ClientJsonrpcRequest {
    pub fn new(id: RequestId, request: RequestFromClient) -> Self {
        let client_request = match request {
            RequestFromClient::ListResourcesRequest(params) => ClientRequest::ListResourcesRequest(ListResourcesRequest::new(id, params)),
            RequestFromClient::ListResourceTemplatesRequest(params) => ClientRequest::ListResourceTemplatesRequest(ListResourceTemplatesRequest::new(id, params)),
            RequestFromClient::ReadResourceRequest(params) => ClientRequest::ReadResourceRequest(ReadResourceRequest::new(id, params)),
            RequestFromClient::ListPromptsRequest(params) => ClientRequest::ListPromptsRequest(ListPromptsRequest::new(id, params)),
            RequestFromClient::GetPromptRequest(params) => ClientRequest::GetPromptRequest(GetPromptRequest::new(id, params)),
            RequestFromClient::ListToolsRequest(params) => ClientRequest::ListToolsRequest(ListToolsRequest::new(id, params)),
            RequestFromClient::CallToolRequest(params) => ClientRequest::CallToolRequest(CallToolRequest::new(id, params)),
            RequestFromClient::CompleteRequest(params) => ClientRequest::CompleteRequest(CompleteRequest::new(id, params)),
            RequestFromClient::DiscoverRequest(params) => ClientRequest::DiscoverRequest(DiscoverRequest::new(id, params)),
            RequestFromClient::SubscriptionsListenRequest(params) => ClientRequest::SubscriptionsListenRequest(SubscriptionsListenRequest::new(id, params)),
            RequestFromClient::CustomRequest(params) => {
                return Self::Custom(JsonrpcRequest::new(id, params.method, params.params))
            }
        };
        Self::Standard(client_request)
    }

    pub fn jsonrpc(&self) -> &::std::string::String {
        match self {
            ClientJsonrpcRequest::Standard(inner) => match inner {
                ClientRequest::ListResourcesRequest(r) => r.jsonrpc(),
                ClientRequest::ListResourceTemplatesRequest(r) => r.jsonrpc(),
                ClientRequest::ReadResourceRequest(r) => r.jsonrpc(),
                ClientRequest::ListPromptsRequest(r) => r.jsonrpc(),
                ClientRequest::GetPromptRequest(r) => r.jsonrpc(),
                ClientRequest::ListToolsRequest(r) => r.jsonrpc(),
                ClientRequest::CallToolRequest(r) => r.jsonrpc(),
                ClientRequest::CompleteRequest(r) => r.jsonrpc(),
                ClientRequest::DiscoverRequest(r) => r.jsonrpc(),
                ClientRequest::SubscriptionsListenRequest(r) => r.jsonrpc(),
            },
            ClientJsonrpcRequest::Custom(request) => request.jsonrpc(),
        }
    }

    pub fn request_id(&self) -> &RequestId {
        match self {
            ClientJsonrpcRequest::Standard(inner) => match inner {
                ClientRequest::ListResourcesRequest(r) => &r.id,
                ClientRequest::ListResourceTemplatesRequest(r) => &r.id,
                ClientRequest::ReadResourceRequest(r) => &r.id,
                ClientRequest::ListPromptsRequest(r) => &r.id,
                ClientRequest::GetPromptRequest(r) => &r.id,
                ClientRequest::ListToolsRequest(r) => &r.id,
                ClientRequest::CallToolRequest(r) => &r.id,
                ClientRequest::CompleteRequest(r) => &r.id,
                ClientRequest::DiscoverRequest(r) => &r.id,
                ClientRequest::SubscriptionsListenRequest(r) => &r.id,
            },
            ClientJsonrpcRequest::Custom(request) => &request.id,
        }
    }

    pub fn method(&self) -> &str {
        match self {
            ClientJsonrpcRequest::Standard(inner) => match inner {
                ClientRequest::ListResourcesRequest(r) => r.method(),
                ClientRequest::ListResourceTemplatesRequest(r) => r.method(),
                ClientRequest::ReadResourceRequest(r) => r.method(),
                ClientRequest::ListPromptsRequest(r) => r.method(),
                ClientRequest::GetPromptRequest(r) => r.method(),
                ClientRequest::ListToolsRequest(r) => r.method(),
                ClientRequest::CallToolRequest(r) => r.method(),
                ClientRequest::CompleteRequest(r) => r.method(),
                ClientRequest::DiscoverRequest(r) => r.method(),
                ClientRequest::SubscriptionsListenRequest(r) => r.method(),
            },
            ClientJsonrpcRequest::Custom(request) => request.method.as_str(),
        }
    }
}


impl From<ClientJsonrpcRequest> for RequestFromClient {
    fn from(request: ClientJsonrpcRequest) -> Self {
        match request {
            ClientJsonrpcRequest::Standard(inner) => match inner {
                ClientRequest::ListResourcesRequest(r) => Self::ListResourcesRequest(r.params),
                ClientRequest::ListResourceTemplatesRequest(r) => Self::ListResourceTemplatesRequest(r.params),
                ClientRequest::ReadResourceRequest(r) => Self::ReadResourceRequest(r.params),
                ClientRequest::ListPromptsRequest(r) => Self::ListPromptsRequest(r.params),
                ClientRequest::GetPromptRequest(r) => Self::GetPromptRequest(r.params),
                ClientRequest::ListToolsRequest(r) => Self::ListToolsRequest(r.params),
                ClientRequest::CallToolRequest(r) => Self::CallToolRequest(r.params),
                ClientRequest::CompleteRequest(r) => Self::CompleteRequest(r.params),
                ClientRequest::DiscoverRequest(r) => Self::DiscoverRequest(r.params),
                ClientRequest::SubscriptionsListenRequest(r) => Self::SubscriptionsListenRequest(r.params),
            },
            ClientJsonrpcRequest::Custom(request) => Self::CustomRequest(CustomRequest {
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
    ListResourcesRequest(PaginatedRequestParams),
    ListResourceTemplatesRequest(PaginatedRequestParams),
    ReadResourceRequest(ReadResourceRequestParams),
    ListPromptsRequest(PaginatedRequestParams),
    GetPromptRequest(GetPromptRequestParams),
    ListToolsRequest(PaginatedRequestParams),
    CallToolRequest(CallToolRequestParams),
    CompleteRequest(CompleteRequestParams),
    DiscoverRequest(RequestParams),
    SubscriptionsListenRequest(SubscriptionsListenRequestParams),
    CustomRequest(CustomRequest),
}

impl RequestFromClient {
    pub fn method(&self) -> &str {
        match self {
            RequestFromClient::ListResourcesRequest(_request) => ListResourcesRequest::method_value(),
            RequestFromClient::ListResourceTemplatesRequest(_request) => ListResourceTemplatesRequest::method_value(),
            RequestFromClient::ReadResourceRequest(_request) => ReadResourceRequest::method_value(),
            RequestFromClient::ListPromptsRequest(_request) => ListPromptsRequest::method_value(),
            RequestFromClient::GetPromptRequest(_request) => GetPromptRequest::method_value(),
            RequestFromClient::ListToolsRequest(_request) => ListToolsRequest::method_value(),
            RequestFromClient::CallToolRequest(_request) => CallToolRequest::method_value(),
            RequestFromClient::CompleteRequest(_request) => CompleteRequest::method_value(),
            RequestFromClient::DiscoverRequest(_request) => DiscoverRequest::method_value(),
            RequestFromClient::SubscriptionsListenRequest(_request) => SubscriptionsListenRequest::method_value(),
            RequestFromClient::CustomRequest(request) => request.method.as_str(),
        }
    }
    /// Returns `true` if the request is an `InitializeRequest`.
    /// Note: InitializeRequest has been removed from the 2026-07-28 protocol.
    /// This always returns `false`.
    pub fn is_initialize_request(&self) -> bool {
        false
    }

    /// Stamps a `RequestMetaObject` (`_meta`) onto the request params.
    ///
    /// `params._meta` is required on every request in the 2026-07-28 protocol, so the SDK
    /// typically builds one connection-level meta (protocol version, client capabilities,
    /// client info) and stamps it onto each outgoing request via this method.
    /// `CustomRequest` carries arbitrary params and is left unchanged.
    pub fn with_meta(mut self, meta: RequestMetaObject) -> Self {
        match &mut self {
            RequestFromClient::ListResourcesRequest(params) => params.meta = meta,
            RequestFromClient::ListResourceTemplatesRequest(params) => params.meta = meta,
            RequestFromClient::ReadResourceRequest(params) => params.meta = meta,
            RequestFromClient::ListPromptsRequest(params) => params.meta = meta,
            RequestFromClient::GetPromptRequest(params) => params.meta = meta,
            RequestFromClient::ListToolsRequest(params) => params.meta = meta,
            RequestFromClient::CallToolRequest(params) => params.meta = meta,
            RequestFromClient::CompleteRequest(params) => params.meta = meta,
            RequestFromClient::DiscoverRequest(params) => params.meta = meta,
            RequestFromClient::SubscriptionsListenRequest(params) => params.meta = meta,
            RequestFromClient::CustomRequest(_) => {}
        }
        self
    }

    /// Returns a reference to the request's `RequestMetaObject` (`_meta`),
    /// or `None` for `CustomRequest`.
    pub fn meta(&self) -> Option<&RequestMetaObject> {
        match self {
            RequestFromClient::ListResourcesRequest(params) => Some(&params.meta),
            RequestFromClient::ListResourceTemplatesRequest(params) => Some(&params.meta),
            RequestFromClient::ReadResourceRequest(params) => Some(&params.meta),
            RequestFromClient::ListPromptsRequest(params) => Some(&params.meta),
            RequestFromClient::GetPromptRequest(params) => Some(&params.meta),
            RequestFromClient::ListToolsRequest(params) => Some(&params.meta),
            RequestFromClient::CallToolRequest(params) => Some(&params.meta),
            RequestFromClient::CompleteRequest(params) => Some(&params.meta),
            RequestFromClient::DiscoverRequest(params) => Some(&params.meta),
            RequestFromClient::SubscriptionsListenRequest(params) => Some(&params.meta),
            RequestFromClient::CustomRequest(_) => None,
        }
    }

    /// Fills in `inputResponses` and `requestState` to retry the request after the server
    /// answered with an `input_required` result (the 2026-07-28 mid-request input flow).
    ///
    /// Only `CallToolRequest`, `GetPromptRequest`, and `ReadResourceRequest` accept input
    /// responses per the spec; any other variant returns an `RpcError`.
    pub fn with_input_responses(
        self,
        input_responses: InputResponses,
        request_state: Option<String>,
    ) -> std::result::Result<Self, RpcError> {
        match self {
            RequestFromClient::CallToolRequest(mut params) => {
                params.input_responses = Some(input_responses);
                params.request_state = request_state;
                Ok(RequestFromClient::CallToolRequest(params))
            }
            RequestFromClient::GetPromptRequest(mut params) => {
                params.input_responses = Some(input_responses);
                params.request_state = request_state;
                Ok(RequestFromClient::GetPromptRequest(params))
            }
            RequestFromClient::ReadResourceRequest(mut params) => {
                params.input_responses = Some(input_responses);
                params.request_state = request_state;
                Ok(RequestFromClient::ReadResourceRequest(params))
            }
            other => Err(RpcError::invalid_params()
                .with_message(format!("`{}` does not accept inputResponses", other.method()))),
        }
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
    CustomNotification(JsonrpcNotification),
}

impl ClientJsonrpcNotification {
    pub fn new(notification: NotificationFromClient) -> Self {
        match notification {
            NotificationFromClient::CancelledNotification(params) => {
                Self::CancelledNotification(CancelledNotification::new(params))
            }
            NotificationFromClient::CustomNotification(params) => {
                Self::CustomNotification(JsonrpcNotification::new(params.method, params.params))
            }
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        match self {
            ClientJsonrpcNotification::CancelledNotification(notification) => notification.jsonrpc(),
            ClientJsonrpcNotification::CustomNotification(notification) => notification.jsonrpc(),
        }
    }

    pub fn method(&self) -> &str {
        match self {
            ClientJsonrpcNotification::CancelledNotification(notification) => notification.method(),
            ClientJsonrpcNotification::CustomNotification(notification) => notification.method.as_str(),
        }
    }
}

impl From<ClientJsonrpcNotification> for NotificationFromClient {
    fn from(notification: ClientJsonrpcNotification) -> Self {
        match notification {
            ClientJsonrpcNotification::CancelledNotification(notification) => {
                Self::CancelledNotification(notification.params)
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
    pub fn method(&self) -> &str {
        match self {
            NotificationFromClient::CancelledNotification(_notification) => CancelledNotification::method_value(),
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
    CreateMessageResult(CreateMessageResult),
    ListRootsResult(ListRootsResult),
    ElicitResult(ElicitResult),
    Result(Result),
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
            ServerMessage::Request(server_jsonrpc_request) => Some(server_jsonrpc_request.request_id()),
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
            ServerMessage::Request(server_jsonrpc_request) => server_jsonrpc_request.jsonrpc(),

            // Notifications do not have request IDs
            ServerMessage::Notification(notification) => notification.jsonrpc(),
            // If the message is a response, return the associated request ID
            ServerMessage::Response(server_jsonrpc_response) => server_jsonrpc_response.jsonrpc(),
            // If the message is an error, return the associated request ID
            ServerMessage::Error(jsonrpc_error) => jsonrpc_error.jsonrpc(),
        }
    }

    fn method(&self) -> Option<&str> {
        match self {
            ServerMessage::Request(server_jsonrpc_request) => Some(server_jsonrpc_request.method()),
            ServerMessage::Notification(server_jsonrpc_notification) => Some(server_jsonrpc_notification.method()),
            ServerMessage::Response(_) => None,
            ServerMessage::Error(_) => None,
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
    CreateMessageRequest {
        id: RequestId,
        #[serde(default = "default_jsonrpc")]
        jsonrpc: String,
        #[serde(flatten)]
        request: CreateMessageRequest,
    },
    ListRootsRequest {
        id: RequestId,
        #[serde(default = "default_jsonrpc")]
        jsonrpc: String,
        #[serde(flatten)]
        request: ListRootsRequest,
    },
    ElicitRequest {
        id: RequestId,
        #[serde(default = "default_jsonrpc")]
        jsonrpc: String,
        #[serde(flatten)]
        request: ElicitRequest,
    },
    CustomRequest(JsonrpcRequest),
}

impl ServerJsonrpcRequest {
    pub fn new(request_id: RequestId, request: RequestFromServer) -> Self {
        match request {
            RequestFromServer::CreateMessageRequest(params) => {
                Self::CreateMessageRequest {
                    id: request_id.clone(),
                    jsonrpc: "2.0".to_string(),
                    request: CreateMessageRequest::new(params),
                }
            }
            RequestFromServer::ListRootsRequest(params) => Self::ListRootsRequest {
                id: request_id.clone(),
                jsonrpc: "2.0".to_string(),
                request: ListRootsRequest::new(params),
            },
            RequestFromServer::ElicitRequest(params) => Self::ElicitRequest {
                id: request_id.clone(),
                jsonrpc: "2.0".to_string(),
                request: ElicitRequest::new(params),
            },
            RequestFromServer::CustomRequest(request) => {
                Self::CustomRequest(JsonrpcRequest::new(request_id, request.method, request.params))
            }
        }
    }

    pub fn request_id(&self) -> &RequestId {
        match self {
            ServerJsonrpcRequest::CreateMessageRequest { id, .. } => id,
            ServerJsonrpcRequest::ListRootsRequest { id, .. } => id,
            ServerJsonrpcRequest::ElicitRequest { id, .. } => id,
            ServerJsonrpcRequest::CustomRequest(request) => &request.id,
        }
    }

    pub fn jsonrpc(&self) -> &str {
        match self {
            ServerJsonrpcRequest::CreateMessageRequest { jsonrpc, .. } => jsonrpc,
            ServerJsonrpcRequest::ListRootsRequest { jsonrpc, .. } => jsonrpc,
            ServerJsonrpcRequest::ElicitRequest { jsonrpc, .. } => jsonrpc,
            ServerJsonrpcRequest::CustomRequest(request) => request.jsonrpc(),
        }
    }

    pub fn method(&self) -> &str {
        match self {
            ServerJsonrpcRequest::CreateMessageRequest { request, .. } => request.method(),
            ServerJsonrpcRequest::ListRootsRequest { request, .. } => request.method(),
            ServerJsonrpcRequest::ElicitRequest { request, .. } => request.method(),
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
    CreateMessageRequest(CreateMessageRequestParams),
    ListRootsRequest(Option<ListRootsRequestParams>),
    ElicitRequest(ElicitRequestParams),
    CustomRequest(CustomRequest),
}

impl From<ServerJsonrpcRequest> for RequestFromServer {
    fn from(request: ServerJsonrpcRequest) -> Self {
        match request {
            ServerJsonrpcRequest::CreateMessageRequest { request, .. } => Self::CreateMessageRequest(request.params),
            ServerJsonrpcRequest::ListRootsRequest { request, .. } => Self::ListRootsRequest(request.params),
            ServerJsonrpcRequest::ElicitRequest { request, .. } => Self::ElicitRequest(request.params),
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
    Standard(ServerNotification),
    Custom(JsonrpcNotification),
}

impl From<ServerJsonrpcNotification> for NotificationFromServer {
    fn from(notification: ServerJsonrpcNotification) -> Self {
        match notification {
            ServerJsonrpcNotification::Standard(inner) => match inner {
                ServerNotification::CancelledNotification(n) => Self::CancelledNotification(n.params),
                ServerNotification::ProgressNotification(n) => Self::ProgressNotification(n.params),
                ServerNotification::ResourceListChangedNotification(n) => Self::ResourceListChangedNotification(n.params),
                ServerNotification::ResourceUpdatedNotification(n) => Self::ResourceUpdatedNotification(n.params),
                ServerNotification::PromptListChangedNotification(n) => Self::PromptListChangedNotification(n.params),
                ServerNotification::ToolListChangedNotification(n) => Self::ToolListChangedNotification(n.params),
                ServerNotification::LoggingMessageNotification(n) => Self::LoggingMessageNotification(n.params),
                ServerNotification::SubscriptionsAcknowledgedNotification(n) => Self::SubscriptionsAcknowledgedNotification(n.params),
            },
            ServerJsonrpcNotification::Custom(notification) => Self::CustomNotification(CustomNotification {
                method: notification.method,
                params: notification.params,
            }),
        }
    }
}

impl ServerJsonrpcNotification {
    pub fn new(notification: NotificationFromServer) -> Self {
        match notification {
            NotificationFromServer::CancelledNotification(params) => {
                Self::Standard(ServerNotification::CancelledNotification(CancelledNotification::new(params)))
            }
            NotificationFromServer::ProgressNotification(params) => {
                Self::Standard(ServerNotification::ProgressNotification(ProgressNotification::new(params)))
            }
            NotificationFromServer::ResourceListChangedNotification(params) => {
                Self::Standard(ServerNotification::ResourceListChangedNotification(ResourceListChangedNotification::new(params)))
            }
            NotificationFromServer::ResourceUpdatedNotification(params) => {
                Self::Standard(ServerNotification::ResourceUpdatedNotification(ResourceUpdatedNotification::new(params)))
            }
            NotificationFromServer::PromptListChangedNotification(params) => {
                Self::Standard(ServerNotification::PromptListChangedNotification(PromptListChangedNotification::new(params)))
            }
            NotificationFromServer::ToolListChangedNotification(params) => {
                Self::Standard(ServerNotification::ToolListChangedNotification(ToolListChangedNotification::new(params)))
            }
            NotificationFromServer::LoggingMessageNotification(params) => {
                Self::Standard(ServerNotification::LoggingMessageNotification(LoggingMessageNotification::new(params)))
            }
            NotificationFromServer::SubscriptionsAcknowledgedNotification(params) => {
                Self::Standard(ServerNotification::SubscriptionsAcknowledgedNotification(SubscriptionsAcknowledgedNotification::new(params)))
            }
            NotificationFromServer::CustomNotification(params) => {
                Self::Custom(JsonrpcNotification::new(params.method, params.params))
            }
        }
    }

    pub fn jsonrpc(&self) -> &::std::string::String {
        match self {
            ServerJsonrpcNotification::Standard(inner) => match inner {
                ServerNotification::CancelledNotification(n) => n.jsonrpc(),
                ServerNotification::ProgressNotification(n) => n.jsonrpc(),
                ServerNotification::ResourceListChangedNotification(n) => n.jsonrpc(),
                ServerNotification::ResourceUpdatedNotification(n) => n.jsonrpc(),
                ServerNotification::PromptListChangedNotification(n) => n.jsonrpc(),
                ServerNotification::ToolListChangedNotification(n) => n.jsonrpc(),
                ServerNotification::LoggingMessageNotification(n) => n.jsonrpc(),
                ServerNotification::SubscriptionsAcknowledgedNotification(n) => n.jsonrpc(),
            },
            ServerJsonrpcNotification::Custom(notification) => notification.jsonrpc(),
        }
    }

    pub fn method(&self) -> &str {
        match self {
            ServerJsonrpcNotification::Standard(inner) => match inner {
                ServerNotification::CancelledNotification(n) => n.method(),
                ServerNotification::ProgressNotification(n) => n.method(),
                ServerNotification::ResourceListChangedNotification(n) => n.method(),
                ServerNotification::ResourceUpdatedNotification(n) => n.method(),
                ServerNotification::PromptListChangedNotification(n) => n.method(),
                ServerNotification::ToolListChangedNotification(n) => n.method(),
                ServerNotification::LoggingMessageNotification(n) => n.method(),
                ServerNotification::SubscriptionsAcknowledgedNotification(n) => n.method(),
            },
            ServerJsonrpcNotification::Custom(notification) => notification.method.as_str(),
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
    LoggingMessageNotification(LoggingMessageNotificationParams),
    SubscriptionsAcknowledgedNotification(SubscriptionsAcknowledgedNotificationParams),
    CustomNotification(CustomNotification),
}

impl NotificationFromServer {
    pub fn method(&self) -> &str {
        match self {
            NotificationFromServer::CancelledNotification(_params) => CancelledNotification::method_value(),
            NotificationFromServer::ProgressNotification(_params) => ProgressNotification::method_value(),
            NotificationFromServer::ResourceListChangedNotification(_params) => ResourceListChangedNotification::method_value(),
            NotificationFromServer::ResourceUpdatedNotification(_params) => ResourceUpdatedNotification::method_value(),
            NotificationFromServer::PromptListChangedNotification(_params) => PromptListChangedNotification::method_value(),
            NotificationFromServer::ToolListChangedNotification(_params) => ToolListChangedNotification::method_value(),
            NotificationFromServer::LoggingMessageNotification(_params) => LoggingMessageNotification::method_value(),
            NotificationFromServer::SubscriptionsAcknowledgedNotification(_params) => SubscriptionsAcknowledgedNotification::method_value(),
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
    pub result: ServerResult,
}

impl ServerJsonrpcResponse {
    pub fn new(id: RequestId, result: ServerResult) -> Self {
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
//**      ServerResult     **//
//*******************************//
// ServerResult is now ServerResult from the schema-generated types.
// The schema's ServerResult enum covers all result types.
// See mcp_schema.rs for the definition.

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
    ServerResult(ServerResult),
    NotificationFromServer(NotificationFromServer),
    Error(RpcError),
}

impl From<RequestFromServer> for MessageFromServer {
    fn from(value: RequestFromServer) -> Self {
        Self::RequestFromServer(value)
    }
}

impl From<ServerResult> for MessageFromServer {
    fn from(value: ServerResult) -> Self {
        Self::ServerResult(value)
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
        matches!(self, MessageFromServer::ServerResult(_))
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
            MessageFromServer::ServerResult(_) => MessageTypes::Response,
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
                    RequestFromServer::CreateMessageRequest(params) => {
                        ServerJsonrpcRequest::CreateMessageRequest {
                            id: request_id,
                            jsonrpc: "2.0".to_string(),
                            request: CreateMessageRequest::new(params),
                        }
                    }
                    RequestFromServer::ListRootsRequest(params) => {
                        ServerJsonrpcRequest::ListRootsRequest {
                            id: request_id,
                            jsonrpc: "2.0".to_string(),
                            request: ListRootsRequest::new(params),
                        }
                    }
                    RequestFromServer::ElicitRequest(params) => {
                        ServerJsonrpcRequest::ElicitRequest {
                            id: request_id,
                            jsonrpc: "2.0".to_string(),
                            request: ElicitRequest::new(params),
                        }
                    }
                    RequestFromServer::CustomRequest(params) => {
                        ServerJsonrpcRequest::CustomRequest(JsonrpcRequest::new(request_id, params.method, params.params))
                    }
                };

                Ok(ServerMessage::Request(rpc_message))
            }
            MessageFromServer::ServerResult(result_from_server) => {
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
    /// Note: InitializeRequest has been removed from the 2026-07-28 protocol.
    /// This always returns `false`.
    pub fn is_initialize_request(&self) -> bool {
        false
    }

    /// Returns `true` if the message is an `InitializedNotification`
    /// Note: InitializedNotification has been removed from the 2026-07-28 protocol.
    /// This always returns `false`.
    pub fn is_initialized_notification(&self) -> bool {
        false
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
            result_type: "complete".to_string(),
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
    pub fn with_meta(mut self, meta: MetaObject) -> Self {
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
    pub fn with_meta(mut self, meta: MetaObject) -> Self {
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
                let maximum = value.get("maximum").and_then(|v| v.as_number().and_then(|n| n.as_f64()));
                let minimum = value.get("minimum").and_then(|v| v.as_number().and_then(|n| n.as_f64()));
                let default = value.get("default").and_then(|v| v.as_number().and_then(|n| n.as_f64()));

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

impl ElicitRequestParams {
    pub fn message(&self) -> &str {
        match self {
            ElicitRequestParams::UrlParams(elicit_request_url_params) => elicit_request_url_params.message.as_str(),
            ElicitRequestParams::FormParams(elicit_request_form_params) => elicit_request_form_params.message.as_str(),
        }
    }
}

impl ServerCapabilities {
    pub fn can_handle_request(&self, client_request: &ClientJsonrpcRequest) -> std::result::Result<(), RpcError> {
        let request_method = client_request.method();

        fn create_error(capability: &str, method: &str) -> RpcError {
            RpcError::internal_error().with_message(create_unsupported_capability_message("Server", capability, method))
        }

        match client_request {
            ClientJsonrpcRequest::Standard(inner) => match inner {
                ClientRequest::GetPromptRequest(_) | ClientRequest::ListPromptsRequest(_)
                    if self.prompts.is_none() =>
                {
                    return Err(create_error("prompts", request_method));
                }
                ClientRequest::ListResourcesRequest(_)
                | ClientRequest::ListResourceTemplatesRequest(_)
                | ClientRequest::ReadResourceRequest(_)
                    if self.resources.is_none() =>
                {
                    return Err(create_error("resources", request_method));
                }
                ClientRequest::CallToolRequest(_) | ClientRequest::ListToolsRequest(_) if self.tools.is_none() => {
                    return Err(create_error("tools", request_method));
                }
                ClientRequest::CompleteRequest(_) if self.completions.is_none() => {
                    return Err(create_error("completions", request_method));
                }
                ClientRequest::DiscoverRequest(_) => {},
                ClientRequest::SubscriptionsListenRequest(_) => {},
                _ => {}
            },
            ClientJsonrpcRequest::Custom(_) => {}
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
    /// Asserts that the client supports the requested server-initiated operation.
    ///
    /// In the 2026-07-28 protocol, server→client requests arrive as `InputRequest`s
    /// (mid-request) or standalone JSON-RPC requests. Each maps to a client capability:
    /// - `sampling/createMessage` → `sampling`
    /// - `roots/list` → `roots`
    /// - `elicitation/create` → `elicitation`
    pub fn can_handle_request(&self, server_jsonrpc_request: &ServerJsonrpcRequest) -> std::result::Result<(), RpcError> {
        let entity = "Client";
        match server_jsonrpc_request {
            ServerJsonrpcRequest::CreateMessageRequest { .. } if self.sampling.is_none() => {
                Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                    entity,
                    "sampling",
                    CreateMessageRequest::method_value(),
                )))
            }
            ServerJsonrpcRequest::ListRootsRequest { .. } if self.roots.is_none() => {
                Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                    entity,
                    "roots",
                    ListRootsRequest::method_value(),
                )))
            }
            ServerJsonrpcRequest::ElicitRequest { .. } if self.elicitation.is_none() => {
                Err(RpcError::internal_error().with_message(create_unsupported_capability_message(
                    entity,
                    "elicitation",
                    ElicitRequest::method_value(),
                )))
            }
            _ => Ok(()),
        }
    }

    /// Asserts that the client can emit the given notification.
    ///
    /// Client→server notifications (`notifications/cancelled`, `notifications/progress`)
    /// are not gated by any client capability, so this always succeeds.
    pub fn can_accept_notification(&self, _notification_method: &str) -> std::result::Result<(), RpcError> {
        Ok(())
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

impl ServerResult {
    /// Returns `true` if this result is an `InputRequiredResult`, i.e. the server is
    /// requesting additional client input (sampling, roots, or elicitation) before it can
    /// complete the original request. This is the 2026-07-28 mid-request interaction mechanism.
    pub fn is_input_required(&self) -> bool {
        matches!(self, ServerResult::InputRequiredResult(_))
    }

    /// Returns `true` if this result is a final (complete) result rather than an
    /// `InputRequiredResult`. For results received from pre-2026 peers (which carry no
    /// `resultType`), this is always `true`.
    pub fn is_complete(&self) -> bool {
        !self.is_input_required()
    }

    /// Returns the inner `InputRequiredResult` if this is an input-required response,
    /// otherwise `None`. Use this to drive the elicitation/retry loop.
    pub fn as_input_required(&self) -> Option<&InputRequiredResult> {
        match self {
            ServerResult::InputRequiredResult(result) => Some(result),
            _ => None,
        }
    }
}

//*********************************//
//**  RequestMetaObject helpers  **//
//*********************************//

impl RequestMetaObject {
    /// Creates the connection-level `_meta` required on every 2026-07-28 request,
    /// with the two mandatory fields: protocol version and client capabilities.
    pub fn new<T: Into<String>>(protocol_version: T, client_capabilities: ClientCapabilities) -> Self {
        Self {
            client_capabilities,
            client_info: None,
            log_level: None,
            protocol_version: protocol_version.into(),
            progress_token: None,
            extra: None,
        }
    }

    pub fn with_client_info(mut self, client_info: Implementation) -> Self {
        self.client_info = Some(client_info);
        self
    }

    pub fn with_log_level(mut self, log_level: LoggingLevel) -> Self {
        self.log_level = Some(log_level);
        self
    }

    pub fn with_progress_token(mut self, progress_token: ProgressToken) -> Self {
        self.progress_token = Some(progress_token);
        self
    }

    // --- SEP-414 OpenTelemetry trace-context accessors ------------------------
    //
    // `traceparent`, `tracestate` and `baggage` are reserved `_meta` keys
    // (W3C Trace Context / Baggage) carried through the `extra` catch-all map.
    // Values are opaque strings here — format validation is intentionally left
    // to the SDK layer so this crate stays dependency-free and lossless.

    /// Returns the W3C `traceparent` value, if present.
    pub fn traceparent(&self) -> Option<&str> {
        self.extra
            .as_ref()
            .and_then(|m| m.get("traceparent"))
            .and_then(|v| v.as_str())
    }

    /// Sets the W3C `traceparent` value.
    pub fn with_traceparent(mut self, traceparent: impl Into<String>) -> Self {
        self.extra_mut().insert(
            "traceparent".to_string(),
            serde_json::Value::String(traceparent.into()),
        );
        self
    }

    /// Returns the W3C `tracestate` value, if present.
    pub fn tracestate(&self) -> Option<&str> {
        self.extra
            .as_ref()
            .and_then(|m| m.get("tracestate"))
            .and_then(|v| v.as_str())
    }

    /// Sets the W3C `tracestate` value.
    pub fn with_tracestate(mut self, tracestate: impl Into<String>) -> Self {
        self.extra_mut().insert(
            "tracestate".to_string(),
            serde_json::Value::String(tracestate.into()),
        );
        self
    }

    /// Returns the W3C `baggage` value, if present.
    pub fn baggage(&self) -> Option<&str> {
        self.extra
            .as_ref()
            .and_then(|m| m.get("baggage"))
            .and_then(|v| v.as_str())
    }

    /// Sets the W3C `baggage` value.
    pub fn with_baggage(mut self, baggage: impl Into<String>) -> Self {
        self.extra_mut().insert(
            "baggage".to_string(),
            serde_json::Value::String(baggage.into()),
        );
        self
    }

    /// Internal helper: get-or-create the `extra` catch-all map.
    fn extra_mut(&mut self) -> &mut ::serde_json::Map<String, ::serde_json::Value> {
        self.extra.get_or_insert_with(Default::default)
    }
}

//*********************************//
//**  InputRequired helpers      **//
//*********************************//

impl InputRequests {
    /// Iterates over the `(key, InputRequest)` pairs the server is asking the client to resolve.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &InputRequest)> {
        self.0.iter()
    }

    /// Iterates over the request keys (used to correlate each `InputResponse` in the retry).
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// Returns the `InputRequest` registered under `key`, if any.
    pub fn get(&self, key: &str) -> Option<&InputRequest> {
        self.0.get(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl InputResponses {
    pub fn new() -> Self {
        Self(std::collections::BTreeMap::new())
    }

    /// Adds a resolved response under the key of its originating `InputRequest`
    /// (chainable builder for the InputRequired retry flow).
    pub fn insert<K: Into<String>, V: Into<InputResponse>>(mut self, key: K, response: V) -> Self {
        self.0.insert(key.into(), response.into());
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for InputResponses {
    fn default() -> Self {
        Self::new()
    }
}

impl InputRequest {
    /// Returns the inner request if this is a `sampling/createMessage` input request.
    pub fn as_create_message(&self) -> Option<&CreateMessageRequest> {
        match self {
            InputRequest::CreateMessageRequest(request) => Some(request),
            _ => None,
        }
    }

    /// Returns the inner request if this is a `roots/list` input request.
    pub fn as_list_roots(&self) -> Option<&ListRootsRequest> {
        match self {
            InputRequest::ListRootsRequest(request) => Some(request),
            _ => None,
        }
    }

    /// Returns the inner request if this is an `elicitation/create` input request.
    pub fn as_elicit(&self) -> Option<&ElicitRequest> {
        match self {
            InputRequest::ElicitRequest(request) => Some(request),
            _ => None,
        }
    }

    /// The JSON-RPC method of the wrapped input request
    /// (`sampling/createMessage`, `roots/list`, or `elicitation/create`).
    pub fn method(&self) -> &str {
        match self {
            InputRequest::CreateMessageRequest(request) => request.method(),
            InputRequest::ListRootsRequest(request) => request.method(),
            InputRequest::ElicitRequest(request) => request.method(),
        }
    }
}

/// BEGIN AUTO GENERATED
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
impl From<Result> for ResultFromClient {
    fn from(value: Result) -> Self {
        Self::Result(value)
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
impl From<Result> for MessageFromClient {
    fn from(value: Result) -> Self {
        MessageFromClient::ResultFromClient(value.into())
    }
}
impl From<DiscoverRequest> for ClientJsonrpcRequest {
    fn from(value: DiscoverRequest) -> Self {
        Self::Standard(ClientRequest::DiscoverRequest(value))
    }
}
impl From<ListResourcesRequest> for ClientJsonrpcRequest {
    fn from(value: ListResourcesRequest) -> Self {
        Self::Standard(ClientRequest::ListResourcesRequest(value))
    }
}
impl From<ListResourceTemplatesRequest> for ClientJsonrpcRequest {
    fn from(value: ListResourceTemplatesRequest) -> Self {
        Self::Standard(ClientRequest::ListResourceTemplatesRequest(value))
    }
}
impl From<ReadResourceRequest> for ClientJsonrpcRequest {
    fn from(value: ReadResourceRequest) -> Self {
        Self::Standard(ClientRequest::ReadResourceRequest(value))
    }
}
impl From<SubscriptionsListenRequest> for ClientJsonrpcRequest {
    fn from(value: SubscriptionsListenRequest) -> Self {
        Self::Standard(ClientRequest::SubscriptionsListenRequest(value))
    }
}
impl From<ListPromptsRequest> for ClientJsonrpcRequest {
    fn from(value: ListPromptsRequest) -> Self {
        Self::Standard(ClientRequest::ListPromptsRequest(value))
    }
}
impl From<GetPromptRequest> for ClientJsonrpcRequest {
    fn from(value: GetPromptRequest) -> Self {
        Self::Standard(ClientRequest::GetPromptRequest(value))
    }
}
impl From<ListToolsRequest> for ClientJsonrpcRequest {
    fn from(value: ListToolsRequest) -> Self {
        Self::Standard(ClientRequest::ListToolsRequest(value))
    }
}
impl From<CallToolRequest> for ClientJsonrpcRequest {
    fn from(value: CallToolRequest) -> Self {
        Self::Standard(ClientRequest::CallToolRequest(value))
    }
}
impl From<CompleteRequest> for ClientJsonrpcRequest {
    fn from(value: CompleteRequest) -> Self {
        Self::Standard(ClientRequest::CompleteRequest(value))
    }
}
/// Enum representing SDK error codes.
///
/// Grouping (reconciled with the 2026-07-28 schema):
/// - **SDK-internal** (transport/session; not in the MCP spec): `CONNECTION_CLOSED`,
///   `REQUEST_TIMEOUT`, `RESOURCE_NOT_FOUND`, `BAD_REQUEST`, `SESSION_NOT_FOUND`.
/// - **MCP-spec** (also schema-derived in `RpcErrorCodes` and as typed error structs
///   `MissingRequiredClientCapabilityError` / `UnsupportedProtocolVersionError`):
///   `MISSING_REQUIRED_CLIENT_CAPABILITY`, `UNSUPPORTED_PROTOCOL_VERSION`.
/// - **JSON-RPC standard** (also schema-derived in `RpcErrorCodes` and as typed error
///   structs `ParseError`/`InvalidRequestError`/etc.): `INVALID_REQUEST`,
///   `METHOD_NOT_FOUND`, `INVALID_PARAMS`, `INTERNAL_ERROR`, `PARSE_ERROR`.
///
/// Prefer `RpcErrorCodes` (schema-derived) for protocol errors on the wire; use
/// `SdkErrorCodes` for SDK-internal transport/session failures.
#[allow(non_camel_case_types)]
pub enum SdkErrorCodes {
    CONNECTION_CLOSED = -32000,
    REQUEST_TIMEOUT = -32001,
    RESOURCE_NOT_FOUND = -32002,
    BAD_REQUEST = -32015,
    SESSION_NOT_FOUND = -32016,
    MISSING_REQUIRED_CLIENT_CAPABILITY = -32021,
    UNSUPPORTED_PROTOCOL_VERSION = -32022,
    INVALID_REQUEST = -32600,
    METHOD_NOT_FOUND = -32601,
    INVALID_PARAMS = -32602,
    INTERNAL_ERROR = -32603,
    PARSE_ERROR = -32700,
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
            SdkErrorCodes::MISSING_REQUIRED_CLIENT_CAPABILITY => {
                write!(f, "Missing required client capability")
            }
            SdkErrorCodes::UNSUPPORTED_PROTOCOL_VERSION => {
                write!(f, "Unsupported protocol version")
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
    pub code: i64,
    pub data: ::std::option::Option<::serde_json::Value>,
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
#[allow(non_camel_case_types)]
pub enum RpcErrorCodes {
    PARSE_ERROR = -32700isize,
    INVALID_REQUEST = -32600isize,
    METHOD_NOT_FOUND = -32601isize,
    INVALID_PARAMS = -32602isize,
    INTERNAL_ERROR = -32603isize,
    HEADER_MISMATCH = -32020isize,
    MISSING_REQUIRED_CLIENT_CAPABILITY = -32021isize,
    UNSUPPORTED_PROTOCOL_VERSION = -32022isize,
}
impl From<RpcErrorCodes> for i64 {
    fn from(code: RpcErrorCodes) -> Self {
        code as i64
    }
}
impl RpcError {
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
    pub fn method_not_found() -> Self {
        Self {
            code: RpcErrorCodes::METHOD_NOT_FOUND.into(),
            data: None,
            message: "Method not found".to_string(),
        }
    }
    pub fn invalid_params() -> Self {
        Self {
            code: RpcErrorCodes::INVALID_PARAMS.into(),
            data: None,
            message: "Invalid params".to_string(),
        }
    }
    pub fn invalid_request() -> Self {
        Self {
            code: RpcErrorCodes::INVALID_REQUEST.into(),
            data: None,
            message: "Invalid request".to_string(),
        }
    }
    pub fn internal_error() -> Self {
        Self {
            code: RpcErrorCodes::INTERNAL_ERROR.into(),
            data: None,
            message: "Internal error".to_string(),
        }
    }
    pub fn parse_error() -> Self {
        Self {
            code: RpcErrorCodes::PARSE_ERROR.into(),
            data: None,
            message: "Parse error".to_string(),
        }
    }
    pub fn with_message<T: Into<String>>(mut self, message: T) -> Self {
        self.message = message.into();
        self
    }
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
impl From<MissingRequiredClientCapabilityError> for RpcError {
    fn from(value: MissingRequiredClientCapabilityError) -> Self {
        RpcError {
            code: value.error.code,
            data: serde_json::to_value(value.error.data).ok(),
            message: value.error.message,
        }
    }
}
impl From<UnsupportedProtocolVersionError> for RpcError {
    fn from(value: UnsupportedProtocolVersionError) -> Self {
        RpcError {
            code: value.error.code,
            data: serde_json::to_value(value.error.data).ok(),
            message: value.error.message,
        }
    }
}
impl From<Result> for MessageFromServer {
    fn from(value: Result) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<InputRequiredResult> for MessageFromServer {
    fn from(value: InputRequiredResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<DiscoverResult> for MessageFromServer {
    fn from(value: DiscoverResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<ListResourcesResult> for MessageFromServer {
    fn from(value: ListResourcesResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<ListResourceTemplatesResult> for MessageFromServer {
    fn from(value: ListResourceTemplatesResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<ReadResourceResult> for MessageFromServer {
    fn from(value: ReadResourceResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<SubscriptionsListenResult> for MessageFromServer {
    fn from(value: SubscriptionsListenResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<ListPromptsResult> for MessageFromServer {
    fn from(value: ListPromptsResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<GetPromptResult> for MessageFromServer {
    fn from(value: GetPromptResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<ListToolsResult> for MessageFromServer {
    fn from(value: ListToolsResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<CallToolResult> for MessageFromServer {
    fn from(value: CallToolResult) -> Self {
        MessageFromServer::ServerResult(value.into())
    }
}
impl From<CompleteResult> for MessageFromServer {
    fn from(value: CompleteResult) -> Self {
        MessageFromServer::ServerResult(value.into())
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
impl TryFrom<ResultFromClient> for GenericResult {
    type Error = RpcError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        match value {
            ResultFromClient::Result(result) => Ok(result),
            _ => Err(RpcError::internal_error().with_message("Not a Result".to_string())),
        }
    }
}
impl TryFrom<ServerResult> for GenericResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        match value {
            ServerResult::Result(result) => Ok(result),
            _ => Err(RpcError::internal_error().with_message("Not a Result".to_string())),
        }
    }
}
impl TryFrom<ServerResult> for InputRequiredResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::InputRequiredResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a InputRequiredResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for DiscoverResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::DiscoverResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a DiscoverResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for ListResourcesResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListResourcesResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListResourcesResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for ListResourceTemplatesResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListResourceTemplatesResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListResourceTemplatesResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for ReadResourceResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ReadResourceResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ReadResourceResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for SubscriptionsListenResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::SubscriptionsListenResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a SubscriptionsListenResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for ListPromptsResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListPromptsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListPromptsResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for GetPromptResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::GetPromptResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a GetPromptResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for ListToolsResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::ListToolsResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a ListToolsResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for CallToolResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
        if let ServerResult::CallToolResult(result) = value {
            Ok(result)
        } else {
            Err(RpcError::internal_error().with_message("Not a CallToolResult".to_string()))
        }
    }
}
impl TryFrom<ServerResult> for CompleteResult {
    type Error = RpcError;
    fn try_from(value: ServerResult) -> std::result::Result<Self, Self::Error> {
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
            result_type: "complete".to_string(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn image_content(content: Vec<ImageContent>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            result_type: "complete".to_string(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn audio_content(content: Vec<AudioContent>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            result_type: "complete".to_string(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn resource_link(content: Vec<ResourceLink>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            result_type: "complete".to_string(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn embedded_resource(content: Vec<EmbeddedResource>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
            result_type: "complete".to_string(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn with_error(error: CallToolError) -> Self {
        Self {
            content: vec![ContentBlock::TextContent(TextContent::new(error.to_string(), None, None))],
            result_type: "complete".to_string(),
            is_error: Some(true),
            meta: None,
            structured_content: None,
        }
    }
    pub fn with_meta(mut self, meta: Option<ResultMetaObject>) -> Self {
        self.meta = meta;
        self
    }
    pub fn with_structured_content(
        mut self,
        structured_content: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    ) -> Self {
        self.structured_content = Some(::serde_json::Value::Object(structured_content));
        self
    }
    pub fn from_content(content: Vec<ContentBlock>) -> Self {
        Self {
            content,
            result_type: "complete".to_string(),
            is_error: None,
            meta: None,
            structured_content: None,
        }
    }
    pub fn add_content(mut self, content: ContentBlock) -> Self {
        self.content.push(content);
        self
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
        struct ClientJsonrpcResponseVisitor;
        impl<'de> Visitor<'de> for ClientJsonrpcResponseVisitor {
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
        deserializer.deserialize_struct("JsonrpcResponse", &["id", "jsonrpc", "result"], ClientJsonrpcResponseVisitor)
    }
}
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
        struct ServerJsonrpcResponseVisitor;
        impl<'de> Visitor<'de> for ServerJsonrpcResponseVisitor {
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
                let result = serde_json::from_value::<ServerResult>(result).map_err(de::Error::custom)?;
                Ok(ServerJsonrpcResponse { id, jsonrpc, result })
            }
        }
        deserializer.deserialize_struct("JsonrpcResponse", &["id", "jsonrpc", "result"], ServerJsonrpcResponseVisitor)
    }
}
impl CallToolRequestParams {
    pub fn new<T>(tool_name: T, meta: RequestMetaObject) -> Self
    where
        T: ToString,
    {
        Self {
            name: tool_name.to_string(),
            arguments: None,
            input_responses: None,
            meta,
            request_state: None,
        }
    }
    pub fn with_arguments(mut self, arguments: serde_json::Map<String, Value>) -> Self {
        self.arguments = Some(arguments);
        self
    }
}
impl CallToolRequestParams {
    /// Sets the resolved `inputResponses` for retrying this request after an
    /// `input_required` result.
    pub fn with_input_responses(mut self, input_responses: InputResponses) -> Self {
        self.input_responses = Some(input_responses);
        self
    }
    /// Sets the opaque `requestState` returned by the server, so it can
    /// correlate the retry with the original request.
    pub fn with_request_state<T: Into<String>>(mut self, request_state: T) -> Self {
        self.request_state = Some(request_state.into());
        self
    }
}
impl GetPromptRequestParams {
    /// Sets the resolved `inputResponses` for retrying this request after an
    /// `input_required` result.
    pub fn with_input_responses(mut self, input_responses: InputResponses) -> Self {
        self.input_responses = Some(input_responses);
        self
    }
    /// Sets the opaque `requestState` returned by the server, so it can
    /// correlate the retry with the original request.
    pub fn with_request_state<T: Into<String>>(mut self, request_state: T) -> Self {
        self.request_state = Some(request_state.into());
        self
    }
}
impl InputRequiredResult {
    /// Sets the opaque `requestState` returned by the server, so it can
    /// correlate the retry with the original request.
    pub fn with_request_state<T: Into<String>>(mut self, request_state: T) -> Self {
        self.request_state = Some(request_state.into());
        self
    }
    /// Sets the `inputRequests` the server asks the client to resolve before
    /// the original request can complete.
    pub fn with_input_requests(mut self, input_requests: InputRequests) -> Self {
        self.input_requests = Some(input_requests);
        self
    }
}
impl InputResponseRequestParams {
    /// Sets the resolved `inputResponses` for retrying this request after an
    /// `input_required` result.
    pub fn with_input_responses(mut self, input_responses: InputResponses) -> Self {
        self.input_responses = Some(input_responses);
        self
    }
    /// Sets the opaque `requestState` returned by the server, so it can
    /// correlate the retry with the original request.
    pub fn with_request_state<T: Into<String>>(mut self, request_state: T) -> Self {
        self.request_state = Some(request_state.into());
        self
    }
}
impl ReadResourceRequestParams {
    /// Sets the resolved `inputResponses` for retrying this request after an
    /// `input_required` result.
    pub fn with_input_responses(mut self, input_responses: InputResponses) -> Self {
        self.input_responses = Some(input_responses);
        self
    }
    /// Sets the opaque `requestState` returned by the server, so it can
    /// correlate the retry with the original request.
    pub fn with_request_state<T: Into<String>>(mut self, request_state: T) -> Self {
        self.request_state = Some(request_state.into());
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
        // custom request
        let result = detect_message_type(&json!({
            "id":0,
            "method":"add_numbers",
            "params":{},
            "jsonrpc":"2.0"
        }));
        assert!(matches!(result, MessageTypes::Request));

        // custom notification
        let result = detect_message_type(&json!({
            "method":"notifications/email_sent",
            "jsonrpc":"2.0"
        }));
        assert!(matches!(result, MessageTypes::Notification));

        // standard response
        let message = ClientJsonrpcResponse::new(
            RequestId::Integer(0),
            ResultFromClient::Result(Result {
                meta: None,
                result_type: "complete".to_string(),
                extra: None,
            }),
        );
        let result = detect_message_type(&json!(message));
        assert!(matches!(result, MessageTypes::Response));

        // custom response
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
