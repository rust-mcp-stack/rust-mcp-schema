use crate::generated_schema::*;
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

/// Represents a generic MCP (Model Content Protocol) message.
/// This trait defines methods to classify and extract information from messages.
pub trait RPCMessage: MCPMessage {
    fn request_id(&self) -> Option<&RequestId>;
    fn jsonrpc(&self) -> &str;
}

pub trait MCPMessage {
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
    fn from_message(message: T, request_id: Option<RequestId>) -> std::result::Result<Self, JsonrpcErrorError>;
}

pub trait ToMessage<T>
where
    T: FromMessage<Self>,
    Self: Sized,
{
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<T, JsonrpcErrorError>;
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
    Error(JsonrpcError),
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
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_response(self) -> std::result::Result<ClientJsonrpcResponse, JsonrpcErrorError> {
        if let Self::Response(response) = self {
            Ok(response)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
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
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_request(self) -> std::result::Result<ClientJsonrpcRequest, JsonrpcErrorError> {
        if let Self::Request(request) = self {
            Ok(request)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
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
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_notification(self) -> std::result::Result<ClientJsonrpcNotification, JsonrpcErrorError> {
        if let Self::Notification(notification) = self {
            Ok(notification)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
                    "Invalid message type, expected: \"{}\" received\"{}\"",
                    MessageTypes::Notification,
                    self.message_type()
                ))
                )
        }
    }

    /// Converts the current message into a `JsonrpcError` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Error`. If so, it returns the
    /// `JsonrpcError` wrapped in a `Result::Ok`. If the message is not a `Error`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(JsonrpcError)` if the message is a valid `Error`.
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_error(self) -> std::result::Result<JsonrpcError, JsonrpcErrorError> {
        if let Self::Error(error) = self {
            Ok(error)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
                    "Invalid message type, expected: \"{}\" received\"{}\"",
                    MessageTypes::Error,
                    self.message_type()
                ))
                )
        }
    }
}

impl RPCMessage for ClientMessage {
    // Retrieves the request ID associated with the message, if applicable
    fn request_id(&self) -> Option<&RequestId> {
        match self {
            // If the message is a request, return the associated request ID
            ClientMessage::Request(client_jsonrpc_request) => Some(&client_jsonrpc_request.id),
            // Notifications do not have request IDs
            ClientMessage::Notification(_) => None,
            // If the message is a response, return the associated request ID
            ClientMessage::Response(client_jsonrpc_response) => Some(&client_jsonrpc_response.id),
            // If the message is an error, return the associated request ID
            ClientMessage::Error(jsonrpc_error) => Some(&jsonrpc_error.id),
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

// Implementing the `MCPMessage` trait for `ClientMessage`
impl MCPMessage for ClientMessage {
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
#[derive(Clone, Debug)]
pub struct ClientJsonrpcRequest {
    pub id: RequestId,
    jsonrpc: ::std::string::String,
    pub method: String,
    pub request: RequestFromClient,
}

impl ClientJsonrpcRequest {
    pub fn new(id: RequestId, request: RequestFromClient) -> Self {
        let method = request.method().to_string();
        Self {
            id,
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            request,
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        &self.jsonrpc
    }
}

/// Formats the ClientJsonrpcRequest as a JSON string.
impl Display for ClientJsonrpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

impl FromStr for ClientJsonrpcRequest {
    type Err = JsonrpcErrorError;

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
    /// * `Err(JsonrpcErrorError)` if the string is not valid JSON.
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
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

//*************************//
//** Request From Client **//
//*************************//

/// To determine standard and custom request from the client side
/// Custom requests are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum RequestFromClient {
    ClientRequest(ClientRequest),
    CustomRequest(serde_json::Value),
}

impl TryFrom<RequestFromClient> for ClientRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> result::Result<Self, Self::Error> {
        if let RequestFromClient::ClientRequest(client_request) = value {
            Ok(client_request)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ClientRequest".to_string()))
        }
    }
}

impl RequestFromClient {
    #[deprecated(since = "0.1.4", note = "Use `method()` instead.")]
    pub fn get_method(&self) -> &str {
        match self {
            RequestFromClient::ClientRequest(request) => request.method(),
            RequestFromClient::CustomRequest(request) => request["method"].as_str().unwrap(),
        }
    }

    fn method(&self) -> &str {
        match self {
            RequestFromClient::ClientRequest(request) => request.method(),
            RequestFromClient::CustomRequest(request) => request["method"].as_str().unwrap(),
        }
    }
}

impl From<ClientRequest> for RequestFromClient {
    fn from(value: ClientRequest) -> Self {
        Self::ClientRequest(value)
    }
}

impl From<serde_json::Value> for RequestFromClient {
    fn from(value: serde_json::Value) -> Self {
        Self::CustomRequest(value)
    }
}

impl<'de> serde::Deserialize<'de> for RequestFromClient {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_value = Value::deserialize(deserializer)?;

        let client_result = ClientRequest::deserialize(&raw_value);

        match client_result {
            Ok(client_request) => Ok(Self::ClientRequest(client_request)),
            Err(_) => Ok(Self::CustomRequest(raw_value)),
        }
    }
}

//*******************************//
//** ClientJsonrpcNotification **//
//*******************************//

/// "Similar to JsonrpcNotification , but with the variants restricted to client-side notifications."
#[derive(Clone, Debug)]
pub struct ClientJsonrpcNotification {
    jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub notification: NotificationFromClient,
}

impl ClientJsonrpcNotification {
    pub fn new(notification: NotificationFromClient) -> Self {
        let method = notification.method().to_string();
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            notification,
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        &self.jsonrpc
    }
}

/// Formats the ClientJsonrpcNotification as a JSON string.
impl Display for ClientJsonrpcNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

impl FromStr for ClientJsonrpcNotification {
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

//*******************************//
//**  NotificationFromClient   **//
//*******************************//

/// To determine standard and custom notifications received from the MCP Client
/// Custom notifications are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum NotificationFromClient {
    ClientNotification(ClientNotification),
    CustomNotification(serde_json::Value),
}

impl TryFrom<NotificationFromClient> for ClientNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromClient) -> result::Result<Self, Self::Error> {
        if let NotificationFromClient::ClientNotification(client_notification) = value {
            Ok(client_notification)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ClientNotification".to_string()))
        }
    }
}

impl NotificationFromClient {
    #[deprecated(since = "0.1.4", note = "Use `method()` instead.")]
    pub fn get_method(&self) -> &str {
        match self {
            NotificationFromClient::ClientNotification(notification) => notification.method(),
            NotificationFromClient::CustomNotification(notification) => notification["method"].as_str().unwrap(),
        }
    }

    fn method(&self) -> &str {
        match self {
            NotificationFromClient::ClientNotification(notification) => notification.method(),
            NotificationFromClient::CustomNotification(notification) => notification["method"].as_str().unwrap(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for NotificationFromClient {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_value = Value::deserialize(deserializer)?;

        let result = ClientNotification::deserialize(&raw_value);

        match result {
            Ok(client_notification) => Ok(Self::ClientNotification(client_notification)),
            Err(_) => Ok(Self::CustomNotification(raw_value)),
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
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

impl FromStr for ClientJsonrpcResponse {
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
//*******************************//
//**      ResultFromClient     **//
//*******************************//

/// To determine standard and custom results from the client side
/// Custom results (CustomResult) are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResultFromClient {
    ClientResult(ClientResult),
    CustomResult(serde_json::Value),
}

impl TryFrom<ResultFromClient> for ClientResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromClient) -> result::Result<Self, Self::Error> {
        if let ResultFromClient::ClientResult(client_result) = value {
            Ok(client_result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ClientResult".to_string()))
        }
    }
}

impl From<ClientResult> for ResultFromClient {
    fn from(value: ClientResult) -> Self {
        Self::ClientResult(value)
    }
}

impl From<serde_json::Value> for ResultFromClient {
    fn from(value: serde_json::Value) -> Self {
        Self::CustomResult(value)
    }
}

impl<'de> serde::Deserialize<'de> for ResultFromClient {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_value = Value::deserialize(deserializer)?;

        let result = ClientResult::deserialize(&raw_value);

        match result {
            Ok(client_result) => Ok(Self::ClientResult(client_result)),
            Err(_) => Ok(Self::CustomResult(raw_value)),
        }
    }
}

//*******************************//
//**       ClientMessage       **//
//*******************************//

impl FromStr for ClientMessage {
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
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
    Error(JsonrpcError),
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
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_response(self) -> std::result::Result<ServerJsonrpcResponse, JsonrpcErrorError> {
        if let Self::Response(response) = self {
            Ok(response)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
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
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_request(self) -> std::result::Result<ServerJsonrpcRequest, JsonrpcErrorError> {
        if let Self::Request(request) = self {
            Ok(request)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
                    "Invalid message type, expected: \"{}\" received\"{}\"",
                    MessageTypes::Request,
                    self.message_type()
                ))
                )
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
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_notification(self) -> std::result::Result<ServerJsonrpcNotification, JsonrpcErrorError> {
        if let Self::Notification(notification) = self {
            Ok(notification)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
                    "Invalid message type, expected: \"{}\" received\"{}\"",
                    MessageTypes::Notification,
                    self.message_type()
                ))
                )
        }
    }

    /// Converts the current message into a `JsonrpcError` if it's of the correct type.
    ///
    /// This function checks if the current message is of type `Error`. If so, it returns the
    /// `JsonrpcError` wrapped in a `Result::Ok`. If the message is not a `Error`,
    /// it returns an error with a descriptive message indicating the mismatch in expected message types.
    ///
    /// # Returns
    /// - `Ok(JsonrpcError)` if the message is a valid `Error`.
    /// - `Err(JsonrpcErrorError)` if the message type is invalid
    pub fn as_error(self) -> std::result::Result<JsonrpcError, JsonrpcErrorError> {
        if let Self::Error(error) = self {
            Ok(error)
        } else {
            Err(JsonrpcErrorError::internal_error()
                .with_message(format!(
                    "Invalid message type, expected: \"{}\" received\"{}\"",
                    MessageTypes::Error,
                    self.message_type()
                ))
                )
        }
    }
}

impl RPCMessage for ServerMessage {
    // Retrieves the request ID associated with the message, if applicable
    fn request_id(&self) -> Option<&RequestId> {
        match self {
            // If the message is a request, return the associated request ID
            ServerMessage::Request(server_jsonrpc_request) => Some(&server_jsonrpc_request.id),
            // Notifications do not have request IDs
            ServerMessage::Notification(_) => None,
            // If the message is a response, return the associated request ID
            ServerMessage::Response(server_jsonrpc_response) => Some(&server_jsonrpc_response.id),
            // If the message is an error, return the associated request ID
            ServerMessage::Error(jsonrpc_error) => Some(&jsonrpc_error.id),
        }
    }

    fn jsonrpc(&self) -> &str {
        match self {
            // If the message is a request, return the associated request ID
            ServerMessage::Request(server_jsonrpc_request) => server_jsonrpc_request.jsonrpc(),
            // Notifications do not have request IDs
            ServerMessage::Notification(notification) => notification.jsonrpc(),
            // If the message is a response, return the associated request ID
            ServerMessage::Response(server_jsonrpc_response) => server_jsonrpc_response.jsonrpc(),
            // If the message is an error, return the associated request ID
            ServerMessage::Error(jsonrpc_error) => jsonrpc_error.jsonrpc(),
        }
    }
}

// Implementing the `MCPMessage` trait for `ServerMessage`
impl MCPMessage for ServerMessage {
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
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

//**************************//
//** ServerJsonrpcRequest **//
//**************************//

/// "Similar to JsonrpcRequest , but with the variants restricted to client-side requests."
#[derive(Clone, Debug)]
pub struct ServerJsonrpcRequest {
    pub id: RequestId,
    jsonrpc: ::std::string::String,
    pub method: String,
    pub request: RequestFromServer,
}

impl ServerJsonrpcRequest {
    pub fn new(id: RequestId, request: RequestFromServer) -> Self {
        let method = request.method().to_string();
        Self {
            id,
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            request,
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        &self.jsonrpc
    }
}

/// Formats the ServerJsonrpcRequest as a JSON string.
impl Display for ServerJsonrpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

impl FromStr for ServerJsonrpcRequest {
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
//*************************//
//** Request From Server **//
//*************************//

/// To determine standard and custom request from the server side
/// Custom requests are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum RequestFromServer {
    ServerRequest(ServerRequest),
    CustomRequest(serde_json::Value),
}

impl TryFrom<RequestFromServer> for ServerRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromServer) -> result::Result<Self, Self::Error> {
        if let RequestFromServer::ServerRequest(server_request) = value {
            Ok(server_request)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ServerRequest".to_string()))
        }
    }
}

impl RequestFromServer {
    #[deprecated(since = "0.1.4", note = "Use `method()` instead.")]
    pub fn get_method(&self) -> &str {
        match self {
            RequestFromServer::ServerRequest(request) => request.method(),
            RequestFromServer::CustomRequest(request) => request["method"].as_str().unwrap(),
        }
    }

    fn method(&self) -> &str {
        match self {
            RequestFromServer::ServerRequest(request) => request.method(),
            RequestFromServer::CustomRequest(request) => request["method"].as_str().unwrap(),
        }
    }
}

impl From<ServerRequest> for RequestFromServer {
    fn from(value: ServerRequest) -> Self {
        Self::ServerRequest(value)
    }
}

impl From<serde_json::Value> for RequestFromServer {
    fn from(value: serde_json::Value) -> Self {
        Self::CustomRequest(value)
    }
}

impl<'de> serde::Deserialize<'de> for RequestFromServer {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_value = Value::deserialize(deserializer)?;

        let server_result = ServerRequest::deserialize(&raw_value);

        match server_result {
            Ok(server_request) => Ok(Self::ServerRequest(server_request)),
            Err(_) => Ok(Self::CustomRequest(raw_value)),
        }
    }
}

//*******************************//
//** ServerJsonrpcNotification **//
//*******************************//

/// "Similar to JsonrpcNotification , but with the variants restricted to server-side notifications."
#[derive(Clone, Debug)]
pub struct ServerJsonrpcNotification {
    jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub notification: NotificationFromServer,
}

impl ServerJsonrpcNotification {
    pub fn new(notification: NotificationFromServer) -> Self {
        let method = notification.method().to_string();
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            notification,
        }
    }
    pub fn jsonrpc(&self) -> &::std::string::String {
        &self.jsonrpc
    }
}

/// Formats the ServerJsonrpcNotification as a JSON string.
impl Display for ServerJsonrpcNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

impl FromStr for ServerJsonrpcNotification {
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
//*******************************//
//**  NotificationFromServer   **//
//*******************************//

/// To determine standard and custom notifications received from the MCP Server
/// Custom notifications are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum NotificationFromServer {
    ServerNotification(ServerNotification),
    CustomNotification(serde_json::Value),
}

impl TryFrom<NotificationFromServer> for ServerNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> result::Result<Self, Self::Error> {
        if let NotificationFromServer::ServerNotification(server_notification) = value {
            Ok(server_notification)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ServerNotification".to_string()))
        }
    }
}

impl NotificationFromServer {
    #[deprecated(since = "0.1.4", note = "Use `method()` instead.")]
    pub fn get_method(&self) -> &str {
        match self {
            NotificationFromServer::ServerNotification(notification) => notification.method(),
            NotificationFromServer::CustomNotification(notification) => notification["method"].as_str().unwrap(),
        }
    }

    fn method(&self) -> &str {
        match self {
            NotificationFromServer::ServerNotification(notification) => notification.method(),
            NotificationFromServer::CustomNotification(notification) => notification["method"].as_str().unwrap(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for NotificationFromServer {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_value = Value::deserialize(deserializer)?;

        let result = ServerNotification::deserialize(&raw_value);

        match result {
            Ok(client_notification) => Ok(Self::ServerNotification(client_notification)),
            Err(_) => Ok(Self::CustomNotification(raw_value)),
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
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

impl FromStr for ServerJsonrpcResponse {
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
//*******************************//
//**      ResultFromServer     **//
//*******************************//

/// To determine standard and custom results from the server side
/// Custom results (CustomResult) are of type serde_json::Value and can be deserialized into any custom type.
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResultFromServer {
    ServerResult(ServerResult),
    CustomResult(serde_json::Value),
}

impl TryFrom<ResultFromServer> for ServerResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> result::Result<Self, Self::Error> {
        if let ResultFromServer::ServerResult(server_result) = value {
            Ok(server_result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ServerResult".to_string()))
        }
    }
}

impl From<ServerResult> for ResultFromServer {
    fn from(value: ServerResult) -> Self {
        Self::ServerResult(value)
    }
}

impl From<serde_json::Value> for ResultFromServer {
    fn from(value: serde_json::Value) -> Self {
        Self::CustomResult(value)
    }
}

impl<'de> serde::Deserialize<'de> for ResultFromServer {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_value = Value::deserialize(deserializer)?;

        let result = ServerResult::deserialize(&raw_value);

        match result {
            Ok(server_result) => {
                if matches!(server_result, ServerResult::Result(_)) {
                    Ok(Self::CustomResult(raw_value))
                } else {
                    Ok(Self::ServerResult(server_result))
                }
            }
            Err(_) => Ok(Self::CustomResult(raw_value)),
        }
    }
}

//***************************//
//** impl for JsonrpcError **//
//***************************//

/// Formats the ServerJsonrpcResponse as a JSON string.
impl Display for JsonrpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}

impl FromStr for JsonrpcError {
    type Err = JsonrpcErrorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}

//**************************//
//**  MessageFromServer   **//
//**************************//

/// An enum representing various types of messages that can be sent from an MCP Server.
/// It provides a typed structure for the message payload while skipping internal details like
/// `requestId` and protocol version, which are used solely by the transport layer and
/// do not need to be exposed to the user.
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum MessageFromServer {
    RequestFromServer(RequestFromServer),
    ResultFromServer(ResultFromServer),
    NotificationFromServer(NotificationFromServer),
    Error(JsonrpcErrorError),
}

impl MCPMessage for MessageFromServer {
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
    fn from_message(
        message: MessageFromServer,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        match message {
            MessageFromServer::RequestFromServer(request_from_server) => {
                let request_id = request_id
                    .ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ServerMessage::Request(ServerJsonrpcRequest::new(
                    request_id,
                    request_from_server,
                )))
            }
            MessageFromServer::ResultFromServer(result_from_server) => {
                let request_id = request_id
                    .ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
                    request_id,
                    result_from_server,
                )))
            }
            MessageFromServer::NotificationFromServer(notification_from_server) => {
                if request_id.is_some() {
                    return Err(JsonrpcErrorError::internal_error()
                        .with_message("request_id expected to be None for Notifications!".to_string()));
                }
                Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(
                    notification_from_server,
                )))
            }
            MessageFromServer::Error(jsonrpc_error_error) => {
                let request_id = request_id
                    .ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ServerMessage::Error(JsonrpcError::new(jsonrpc_error_error, request_id)))
            }
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
#[derive(::serde::Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum MessageFromClient {
    RequestFromClient(RequestFromClient),
    ResultFromClient(ResultFromClient),
    NotificationFromClient(NotificationFromClient),
    Error(JsonrpcErrorError),
}

impl FromMessage<MessageFromClient> for ClientMessage {
    fn from_message(
        message: MessageFromClient,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        match message {
            MessageFromClient::RequestFromClient(request_from_client) => {
                let request_id = request_id
                    .ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ClientMessage::Request(ClientJsonrpcRequest::new(
                    request_id,
                    request_from_client,
                )))
            }
            MessageFromClient::ResultFromClient(result_from_client) => {
                let request_id = request_id
                    .ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ClientMessage::Response(ClientJsonrpcResponse::new(
                    request_id,
                    result_from_client,
                )))
            }
            MessageFromClient::NotificationFromClient(notification_from_client) => {
                if request_id.is_some() {
                    return Err(JsonrpcErrorError::internal_error()
                        .with_message("request_id expected to be None for Notifications!".to_string()));
                }

                Ok(ClientMessage::Notification(ClientJsonrpcNotification::new(
                    notification_from_client,
                )))
            }
            MessageFromClient::Error(jsonrpc_error_error) => {
                let request_id = request_id
                    .ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
                Ok(ClientMessage::Error(JsonrpcError::new(jsonrpc_error_error, request_id)))
            }
        }
    }
}

/// BEGIN AUTO GENERATED
impl ::serde::Serialize for ClientJsonrpcRequest {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_struct("JsonrpcRequest", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("jsonrpc", &self.jsonrpc)?;
        state.serialize_field("method", &self.method)?;
        use ClientRequest::*;
        match &self.request {
            RequestFromClient::ClientRequest(message) => match message {
                InitializeRequest(msg) => state.serialize_field("params", &msg.params)?,
                PingRequest(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                ListResourcesRequest(msg) => state.serialize_field("params", &msg.params)?,
                ListResourceTemplatesRequest(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                ReadResourceRequest(msg) => state.serialize_field("params", &msg.params)?,
                SubscribeRequest(msg) => state.serialize_field("params", &msg.params)?,
                UnsubscribeRequest(msg) => state.serialize_field("params", &msg.params)?,
                ListPromptsRequest(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                GetPromptRequest(msg) => state.serialize_field("params", &msg.params)?,
                ListToolsRequest(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                CallToolRequest(msg) => state.serialize_field("params", &msg.params)?,
                SetLevelRequest(msg) => state.serialize_field("params", &msg.params)?,
                CompleteRequest(msg) => state.serialize_field("params", &msg.params)?,
            },
            RequestFromClient::CustomRequest(value) => state.serialize_field("params", value)?,
        }
        state.end()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClientJsonrpcRequest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        struct ClientJsonrpcRequestVisitor;
        impl<'de> Visitor<'de> for ClientJsonrpcRequestVisitor {
            type Value = ClientJsonrpcRequest;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid JSON-RPC request object")
            }
            fn visit_map<M>(self, mut map: M) -> std::result::Result<ClientJsonrpcRequest, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut id: Option<RequestId> = None;
                let mut jsonrpc: Option<String> = None;
                let mut method: Option<String> = None;
                let mut params: Option<Value> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "jsonrpc" => jsonrpc = Some(map.next_value()?),
                        "method" => method = Some(map.next_value()?),
                        "params" => params = Some(map.next_value()?),
                        _ => {
                            return Err(de::Error::unknown_field(&key, &["id", "jsonrpc", "method", "params"]));
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                let method = method.ok_or_else(|| de::Error::missing_field("method"))?;
                let params = params.unwrap_or_default();
                let req_object = json!({ "method" : method, "params" : params });
                let request = serde_json::from_value::<RequestFromClient>(req_object).map_err(de::Error::custom)?;
                Ok(ClientJsonrpcRequest {
                    id,
                    jsonrpc,
                    method,
                    request,
                })
            }
        }
        deserializer.deserialize_struct(
            "JsonrpcRequest",
            &["id", "jsonrpc", "method", "params"],
            ClientJsonrpcRequestVisitor,
        )
    }
}
impl ::serde::Serialize for ServerJsonrpcRequest {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_struct("JsonrpcRequest", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("jsonrpc", &self.jsonrpc)?;
        state.serialize_field("method", &self.method)?;
        use ServerRequest::*;
        match &self.request {
            RequestFromServer::ServerRequest(message) => match message {
                PingRequest(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                CreateMessageRequest(msg) => state.serialize_field("params", &msg.params)?,
                ListRootsRequest(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
            },
            RequestFromServer::CustomRequest(value) => state.serialize_field("params", value)?,
        }
        state.end()
    }
}
impl<'de> ::serde::Deserialize<'de> for ServerJsonrpcRequest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        struct ServerJsonrpcRequestVisitor;
        impl<'de> Visitor<'de> for ServerJsonrpcRequestVisitor {
            type Value = ServerJsonrpcRequest;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid JSON-RPC request object")
            }
            fn visit_map<M>(self, mut map: M) -> std::result::Result<ServerJsonrpcRequest, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut id: Option<RequestId> = None;
                let mut jsonrpc: Option<String> = None;
                let mut method: Option<String> = None;
                let mut params: Option<Value> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "jsonrpc" => jsonrpc = Some(map.next_value()?),
                        "method" => method = Some(map.next_value()?),
                        "params" => params = Some(map.next_value()?),
                        _ => {
                            return Err(de::Error::unknown_field(&key, &["id", "jsonrpc", "method", "params"]));
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                let method = method.ok_or_else(|| de::Error::missing_field("method"))?;
                let params = params.unwrap_or_default();
                let req_object = json!({ "method" : method, "params" : params });
                let request = serde_json::from_value::<RequestFromServer>(req_object).map_err(de::Error::custom)?;
                Ok(ServerJsonrpcRequest {
                    id,
                    jsonrpc,
                    method,
                    request,
                })
            }
        }
        deserializer.deserialize_struct(
            "JsonrpcRequest",
            &["id", "jsonrpc", "method", "params"],
            ServerJsonrpcRequestVisitor,
        )
    }
}
impl ::serde::Serialize for ClientJsonrpcNotification {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_struct("JsonrpcNotification", 3)?;
        state.serialize_field("jsonrpc", &self.jsonrpc)?;
        state.serialize_field("method", &self.method)?;
        use ClientNotification::*;
        match &self.notification {
            NotificationFromClient::ClientNotification(message) => match message {
                CancelledNotification(msg) => state.serialize_field("params", &msg.params)?,
                InitializedNotification(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                ProgressNotification(msg) => state.serialize_field("params", &msg.params)?,
                RootsListChangedNotification(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
            },
            NotificationFromClient::CustomNotification(value) => state.serialize_field("params", value)?,
        }
        state.end()
    }
}
impl<'de> ::serde::Deserialize<'de> for ClientJsonrpcNotification {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        struct ClientJsonrpcNotificationVisitor;
        impl<'de> Visitor<'de> for ClientJsonrpcNotificationVisitor {
            type Value = ClientJsonrpcNotification;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid JSON-RPC notification object")
            }
            fn visit_map<M>(self, mut map: M) -> std::result::Result<ClientJsonrpcNotification, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut jsonrpc: Option<String> = None;
                let mut method: Option<String> = None;
                let mut params: Option<Value> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "jsonrpc" => jsonrpc = Some(map.next_value()?),
                        "method" => method = Some(map.next_value()?),
                        "params" => params = Some(map.next_value()?),
                        _ => {
                            return Err(de::Error::unknown_field(&key, &["id", "jsonrpc", "method", "params"]));
                        }
                    }
                }
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                let method = method.ok_or_else(|| de::Error::missing_field("method"))?;
                let params = params.unwrap_or_default();
                let req_object = json!({ "method" : method, "params" : params });
                let notification =
                    serde_json::from_value::<NotificationFromClient>(req_object).map_err(de::Error::custom)?;
                Ok(ClientJsonrpcNotification {
                    jsonrpc,
                    method,
                    notification,
                })
            }
        }
        deserializer.deserialize_struct(
            "JsonrpcRequest",
            &["jsonrpc", "method", "params"],
            ClientJsonrpcNotificationVisitor,
        )
    }
}
impl ::serde::Serialize for ServerJsonrpcNotification {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        let mut state = serializer.serialize_struct("JsonrpcNotification", 3)?;
        state.serialize_field("jsonrpc", &self.jsonrpc)?;
        state.serialize_field("method", &self.method)?;
        use ServerNotification::*;
        match &self.notification {
            NotificationFromServer::ServerNotification(message) => match message {
                CancelledNotification(msg) => state.serialize_field("params", &msg.params)?,
                ProgressNotification(msg) => state.serialize_field("params", &msg.params)?,
                ResourceListChangedNotification(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                ResourceUpdatedNotification(msg) => state.serialize_field("params", &msg.params)?,
                PromptListChangedNotification(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                ToolListChangedNotification(msg) => {
                    if let Some(params) = &msg.params {
                        state.serialize_field("params", params)?
                    }
                }
                LoggingMessageNotification(msg) => state.serialize_field("params", &msg.params)?,
            },
            NotificationFromServer::CustomNotification(value) => state.serialize_field("params", value)?,
        }
        state.end()
    }
}
impl<'de> ::serde::Deserialize<'de> for ServerJsonrpcNotification {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        struct ServerJsonrpcNotificationVisitor;
        impl<'de> Visitor<'de> for ServerJsonrpcNotificationVisitor {
            type Value = ServerJsonrpcNotification;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid JSON-RPC notification object")
            }
            fn visit_map<M>(self, mut map: M) -> std::result::Result<ServerJsonrpcNotification, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut jsonrpc: Option<String> = None;
                let mut method: Option<String> = None;
                let mut params: Option<Value> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "jsonrpc" => jsonrpc = Some(map.next_value()?),
                        "method" => method = Some(map.next_value()?),
                        "params" => params = Some(map.next_value()?),
                        _ => {
                            return Err(de::Error::unknown_field(&key, &["id", "jsonrpc", "method", "params"]));
                        }
                    }
                }
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                let method = method.ok_or_else(|| de::Error::missing_field("method"))?;
                let params = params.unwrap_or_default();
                let req_object = json!({ "method" : method, "params" : params });
                let notification =
                    serde_json::from_value::<NotificationFromServer>(req_object).map_err(de::Error::custom)?;
                Ok(ServerJsonrpcNotification {
                    jsonrpc,
                    method,
                    notification,
                })
            }
        }
        deserializer.deserialize_struct(
            "JsonrpcRequest",
            &["jsonrpc", "method", "params"],
            ServerJsonrpcNotificationVisitor,
        )
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
impl From<InitializeRequest> for RequestFromClient {
    fn from(value: InitializeRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<PingRequest> for RequestFromClient {
    fn from(value: PingRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<ListResourcesRequest> for RequestFromClient {
    fn from(value: ListResourcesRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<ListResourceTemplatesRequest> for RequestFromClient {
    fn from(value: ListResourceTemplatesRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<ReadResourceRequest> for RequestFromClient {
    fn from(value: ReadResourceRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<SubscribeRequest> for RequestFromClient {
    fn from(value: SubscribeRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<UnsubscribeRequest> for RequestFromClient {
    fn from(value: UnsubscribeRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<ListPromptsRequest> for RequestFromClient {
    fn from(value: ListPromptsRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<GetPromptRequest> for RequestFromClient {
    fn from(value: GetPromptRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<ListToolsRequest> for RequestFromClient {
    fn from(value: ListToolsRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<CallToolRequest> for RequestFromClient {
    fn from(value: CallToolRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<SetLevelRequest> for RequestFromClient {
    fn from(value: SetLevelRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<CompleteRequest> for RequestFromClient {
    fn from(value: CompleteRequest) -> Self {
        Self::ClientRequest(value.into())
    }
}
impl From<InitializeRequest> for MessageFromClient {
    fn from(value: InitializeRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<PingRequest> for MessageFromClient {
    fn from(value: PingRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<ListResourcesRequest> for MessageFromClient {
    fn from(value: ListResourcesRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<ListResourceTemplatesRequest> for MessageFromClient {
    fn from(value: ListResourceTemplatesRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<ReadResourceRequest> for MessageFromClient {
    fn from(value: ReadResourceRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<SubscribeRequest> for MessageFromClient {
    fn from(value: SubscribeRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<UnsubscribeRequest> for MessageFromClient {
    fn from(value: UnsubscribeRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<ListPromptsRequest> for MessageFromClient {
    fn from(value: ListPromptsRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<GetPromptRequest> for MessageFromClient {
    fn from(value: GetPromptRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<ListToolsRequest> for MessageFromClient {
    fn from(value: ListToolsRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<CallToolRequest> for MessageFromClient {
    fn from(value: CallToolRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<SetLevelRequest> for MessageFromClient {
    fn from(value: SetLevelRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<CompleteRequest> for MessageFromClient {
    fn from(value: CompleteRequest) -> Self {
        MessageFromClient::RequestFromClient(value.into())
    }
}
impl From<CancelledNotification> for NotificationFromClient {
    fn from(value: CancelledNotification) -> Self {
        Self::ClientNotification(value.into())
    }
}
impl From<InitializedNotification> for NotificationFromClient {
    fn from(value: InitializedNotification) -> Self {
        Self::ClientNotification(value.into())
    }
}
impl From<ProgressNotification> for NotificationFromClient {
    fn from(value: ProgressNotification) -> Self {
        Self::ClientNotification(value.into())
    }
}
impl From<RootsListChangedNotification> for NotificationFromClient {
    fn from(value: RootsListChangedNotification) -> Self {
        Self::ClientNotification(value.into())
    }
}
impl From<CancelledNotification> for ClientJsonrpcNotification {
    fn from(value: CancelledNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<InitializedNotification> for ClientJsonrpcNotification {
    fn from(value: InitializedNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<ProgressNotification> for ClientJsonrpcNotification {
    fn from(value: ProgressNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<RootsListChangedNotification> for ClientJsonrpcNotification {
    fn from(value: RootsListChangedNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<CancelledNotification> for MessageFromClient {
    fn from(value: CancelledNotification) -> Self {
        MessageFromClient::NotificationFromClient(value.into())
    }
}
impl From<InitializedNotification> for MessageFromClient {
    fn from(value: InitializedNotification) -> Self {
        MessageFromClient::NotificationFromClient(value.into())
    }
}
impl From<ProgressNotification> for MessageFromClient {
    fn from(value: ProgressNotification) -> Self {
        MessageFromClient::NotificationFromClient(value.into())
    }
}
impl From<RootsListChangedNotification> for MessageFromClient {
    fn from(value: RootsListChangedNotification) -> Self {
        MessageFromClient::NotificationFromClient(value.into())
    }
}
impl From<Result> for ResultFromClient {
    fn from(value: Result) -> Self {
        Self::ClientResult(value.into())
    }
}
impl From<CreateMessageResult> for ResultFromClient {
    fn from(value: CreateMessageResult) -> Self {
        Self::ClientResult(value.into())
    }
}
impl From<ListRootsResult> for ResultFromClient {
    fn from(value: ListRootsResult) -> Self {
        Self::ClientResult(value.into())
    }
}
impl From<Result> for MessageFromClient {
    fn from(value: Result) -> Self {
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
/// Enum representing standard JSON-RPC error codes.
#[allow(non_camel_case_types)]
pub enum RpcErrorCodes {
    PARSE_ERROR = -32700isize,
    INVALID_REQUEST = -32600isize,
    METHOD_NOT_FOUND = -32601isize,
    INVALID_PARAMS = -32602isize,
    INTERNAL_ERROR = -32603isize,
}
impl From<RpcErrorCodes> for i64 {
    fn from(code: RpcErrorCodes) -> Self {
        code as i64
    }
}
impl JsonrpcErrorError {
    /// Constructs a new `JsonrpcErrorError` with the provided arguments.
    ///
    /// # Arguments
    /// * `error_code` - The JSON-RPC error code.
    /// * `message` - A descriptive error message.
    /// * `data` - Optional additional data.
    ///
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use rust_mcp_schema::{JsonrpcErrorError, schema_utils::RpcErrorCodes};
    ///
    /// let error = JsonrpcErrorError::new(RpcErrorCodes::INVALID_PARAMS, "Invalid params!".to_string(), Some(json!({"details": "Missing method field"})));
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
    /// Creates a new `JsonrpcErrorError` for "Method not found".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::JsonrpcErrorError;
    ///
    /// let error = JsonrpcErrorError::method_not_found();
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
    /// Creates a new `JsonrpcErrorError` for "Invalid parameters".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::JsonrpcErrorError;
    ///
    /// let error = JsonrpcErrorError::invalid_params();
    /// assert_eq!(error.code, -32602);
    /// ```
    pub fn invalid_params() -> Self {
        Self {
            code: RpcErrorCodes::INVALID_PARAMS.into(),
            data: None,
            message: "Invalid params".to_string(),
        }
    }
    /// Creates a new `JsonrpcErrorError` for "Invalid request".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::JsonrpcErrorError;
    ///
    /// let error = JsonrpcErrorError::invalid_request();
    /// assert_eq!(error.code, -32600);
    /// ```
    pub fn invalid_request() -> Self {
        Self {
            code: RpcErrorCodes::INVALID_REQUEST.into(),
            data: None,
            message: "Invalid request".to_string(),
        }
    }
    /// Creates a new `JsonrpcErrorError` for "Internal error".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::JsonrpcErrorError;
    ///
    /// let error = JsonrpcErrorError::internal_error();
    /// assert_eq!(error.code, -32603);
    /// ```
    pub fn internal_error() -> Self {
        Self {
            code: RpcErrorCodes::INTERNAL_ERROR.into(),
            data: None,
            message: "Internal error".to_string(),
        }
    }
    /// Creates a new `JsonrpcErrorError` for "Parse error".
    ///
    /// # Example
    /// ```
    /// use rust_mcp_schema::JsonrpcErrorError;
    ///
    /// let error = JsonrpcErrorError::parse_error();
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
    /// use rust_mcp_schema::JsonrpcErrorError;
    ///
    /// let error = JsonrpcErrorError::invalid_request().with_message("Request format is invalid".to_string());
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
    /// use rust_mcp_schema::JsonrpcErrorError;
    ///
    /// let error = JsonrpcErrorError::invalid_request().with_data(Some(json!({"reason": "Missing ID"})));
    /// assert!(error.data.is_some());
    /// ```
    pub fn with_data(mut self, data: ::std::option::Option<::serde_json::Value>) -> Self {
        self.data = data;
        self
    }
}
impl std::error::Error for JsonrpcErrorError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl Display for JsonrpcErrorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|err| format!("Serialization error: {}", err))
        )
    }
}
impl FromStr for JsonrpcErrorError {
    type Err = JsonrpcErrorError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|error| JsonrpcErrorError::parse_error().with_data(Some(json!({ "details" : error.to_string() }))))
    }
}
/// Constructs a new JsonrpcError using the provided arguments.
impl JsonrpcError {
    pub fn create(
        id: RequestId,
        error_code: RpcErrorCodes,
        error_message: ::std::string::String,
        error_data: ::std::option::Option<::serde_json::Value>,
    ) -> Self {
        Self::new(JsonrpcErrorError::new(error_code, error_message, error_data), id)
    }
}
impl From<CancelledNotification> for NotificationFromServer {
    fn from(value: CancelledNotification) -> Self {
        Self::ServerNotification(value.into())
    }
}
impl From<ProgressNotification> for NotificationFromServer {
    fn from(value: ProgressNotification) -> Self {
        Self::ServerNotification(value.into())
    }
}
impl From<ResourceListChangedNotification> for NotificationFromServer {
    fn from(value: ResourceListChangedNotification) -> Self {
        Self::ServerNotification(value.into())
    }
}
impl From<ResourceUpdatedNotification> for NotificationFromServer {
    fn from(value: ResourceUpdatedNotification) -> Self {
        Self::ServerNotification(value.into())
    }
}
impl From<PromptListChangedNotification> for NotificationFromServer {
    fn from(value: PromptListChangedNotification) -> Self {
        Self::ServerNotification(value.into())
    }
}
impl From<ToolListChangedNotification> for NotificationFromServer {
    fn from(value: ToolListChangedNotification) -> Self {
        Self::ServerNotification(value.into())
    }
}
impl From<LoggingMessageNotification> for NotificationFromServer {
    fn from(value: LoggingMessageNotification) -> Self {
        Self::ServerNotification(value.into())
    }
}
impl From<CancelledNotification> for ServerJsonrpcNotification {
    fn from(value: CancelledNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<ProgressNotification> for ServerJsonrpcNotification {
    fn from(value: ProgressNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<ResourceListChangedNotification> for ServerJsonrpcNotification {
    fn from(value: ResourceListChangedNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<ResourceUpdatedNotification> for ServerJsonrpcNotification {
    fn from(value: ResourceUpdatedNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<PromptListChangedNotification> for ServerJsonrpcNotification {
    fn from(value: PromptListChangedNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<ToolListChangedNotification> for ServerJsonrpcNotification {
    fn from(value: ToolListChangedNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<LoggingMessageNotification> for ServerJsonrpcNotification {
    fn from(value: LoggingMessageNotification) -> Self {
        Self::new(value.into())
    }
}
impl From<CancelledNotification> for MessageFromServer {
    fn from(value: CancelledNotification) -> Self {
        MessageFromServer::NotificationFromServer(value.into())
    }
}
impl From<ProgressNotification> for MessageFromServer {
    fn from(value: ProgressNotification) -> Self {
        MessageFromServer::NotificationFromServer(value.into())
    }
}
impl From<ResourceListChangedNotification> for MessageFromServer {
    fn from(value: ResourceListChangedNotification) -> Self {
        MessageFromServer::NotificationFromServer(value.into())
    }
}
impl From<ResourceUpdatedNotification> for MessageFromServer {
    fn from(value: ResourceUpdatedNotification) -> Self {
        MessageFromServer::NotificationFromServer(value.into())
    }
}
impl From<PromptListChangedNotification> for MessageFromServer {
    fn from(value: PromptListChangedNotification) -> Self {
        MessageFromServer::NotificationFromServer(value.into())
    }
}
impl From<ToolListChangedNotification> for MessageFromServer {
    fn from(value: ToolListChangedNotification) -> Self {
        MessageFromServer::NotificationFromServer(value.into())
    }
}
impl From<LoggingMessageNotification> for MessageFromServer {
    fn from(value: LoggingMessageNotification) -> Self {
        MessageFromServer::NotificationFromServer(value.into())
    }
}
impl From<PingRequest> for RequestFromServer {
    fn from(value: PingRequest) -> Self {
        Self::ServerRequest(value.into())
    }
}
impl From<CreateMessageRequest> for RequestFromServer {
    fn from(value: CreateMessageRequest) -> Self {
        Self::ServerRequest(value.into())
    }
}
impl From<ListRootsRequest> for RequestFromServer {
    fn from(value: ListRootsRequest) -> Self {
        Self::ServerRequest(value.into())
    }
}
impl From<PingRequest> for MessageFromServer {
    fn from(value: PingRequest) -> Self {
        MessageFromServer::RequestFromServer(value.into())
    }
}
impl From<CreateMessageRequest> for MessageFromServer {
    fn from(value: CreateMessageRequest) -> Self {
        MessageFromServer::RequestFromServer(value.into())
    }
}
impl From<ListRootsRequest> for MessageFromServer {
    fn from(value: ListRootsRequest) -> Self {
        MessageFromServer::RequestFromServer(value.into())
    }
}
impl From<Result> for ResultFromServer {
    fn from(value: Result) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<InitializeResult> for ResultFromServer {
    fn from(value: InitializeResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<ListResourcesResult> for ResultFromServer {
    fn from(value: ListResourcesResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<ListResourceTemplatesResult> for ResultFromServer {
    fn from(value: ListResourceTemplatesResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<ReadResourceResult> for ResultFromServer {
    fn from(value: ReadResourceResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<ListPromptsResult> for ResultFromServer {
    fn from(value: ListPromptsResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<GetPromptResult> for ResultFromServer {
    fn from(value: GetPromptResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<ListToolsResult> for ResultFromServer {
    fn from(value: ListToolsResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<CallToolResult> for ResultFromServer {
    fn from(value: CallToolResult) -> Self {
        Self::ServerResult(value.into())
    }
}
impl From<CompleteResult> for ResultFromServer {
    fn from(value: CompleteResult) -> Self {
        Self::ServerResult(value.into())
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
impl From<CompleteResult> for MessageFromServer {
    fn from(value: CompleteResult) -> Self {
        MessageFromServer::ResultFromServer(value.into())
    }
}
impl FromMessage<InitializeRequest> for ClientMessage {
    fn from_message(
        message: InitializeRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for InitializeRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<PingRequest> for ClientMessage {
    fn from_message(message: PingRequest, request_id: Option<RequestId>) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for PingRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListResourcesRequest> for ClientMessage {
    fn from_message(
        message: ListResourcesRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for ListResourcesRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListResourceTemplatesRequest> for ClientMessage {
    fn from_message(
        message: ListResourceTemplatesRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for ListResourceTemplatesRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<ReadResourceRequest> for ClientMessage {
    fn from_message(
        message: ReadResourceRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for ReadResourceRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<SubscribeRequest> for ClientMessage {
    fn from_message(
        message: SubscribeRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for SubscribeRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<UnsubscribeRequest> for ClientMessage {
    fn from_message(
        message: UnsubscribeRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for UnsubscribeRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListPromptsRequest> for ClientMessage {
    fn from_message(
        message: ListPromptsRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for ListPromptsRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<GetPromptRequest> for ClientMessage {
    fn from_message(
        message: GetPromptRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for GetPromptRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListToolsRequest> for ClientMessage {
    fn from_message(
        message: ListToolsRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for ListToolsRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<CallToolRequest> for ClientMessage {
    fn from_message(
        message: CallToolRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for CallToolRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<SetLevelRequest> for ClientMessage {
    fn from_message(
        message: SetLevelRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for SetLevelRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<CompleteRequest> for ClientMessage {
    fn from_message(
        message: CompleteRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Request(ClientJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ClientMessage> for CompleteRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<Result> for ClientMessage {
    fn from_message(message: Result, request_id: Option<RequestId>) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Response(ClientJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ClientMessage> for Result {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<CreateMessageResult> for ClientMessage {
    fn from_message(
        message: CreateMessageResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Response(ClientJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ClientMessage> for CreateMessageResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListRootsResult> for ClientMessage {
    fn from_message(
        message: ListRootsResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ClientMessage::Response(ClientJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ClientMessage> for ListRootsResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<CancelledNotification> for ClientMessage {
    fn from_message(
        message: CancelledNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ClientMessage::Notification(ClientJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ClientMessage> for CancelledNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<InitializedNotification> for ClientMessage {
    fn from_message(
        message: InitializedNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ClientMessage::Notification(ClientJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ClientMessage> for InitializedNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<ProgressNotification> for ClientMessage {
    fn from_message(
        message: ProgressNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ClientMessage::Notification(ClientJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ClientMessage> for ProgressNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<RootsListChangedNotification> for ClientMessage {
    fn from_message(
        message: RootsListChangedNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ClientMessage::Notification(ClientJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ClientMessage> for RootsListChangedNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ClientMessage, JsonrpcErrorError> {
        ClientMessage::from_message(self, request_id)
    }
}
impl FromMessage<PingRequest> for ServerMessage {
    fn from_message(message: PingRequest, request_id: Option<RequestId>) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Request(ServerJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ServerMessage> for PingRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<CreateMessageRequest> for ServerMessage {
    fn from_message(
        message: CreateMessageRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Request(ServerJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ServerMessage> for CreateMessageRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListRootsRequest> for ServerMessage {
    fn from_message(
        message: ListRootsRequest,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Request(ServerJsonrpcRequest::new(request_id, message.into())))
    }
}
impl ToMessage<ServerMessage> for ListRootsRequest {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<Result> for ServerMessage {
    fn from_message(message: Result, request_id: Option<RequestId>) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for Result {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<InitializeResult> for ServerMessage {
    fn from_message(
        message: InitializeResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for InitializeResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListResourcesResult> for ServerMessage {
    fn from_message(
        message: ListResourcesResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for ListResourcesResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListResourceTemplatesResult> for ServerMessage {
    fn from_message(
        message: ListResourceTemplatesResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for ListResourceTemplatesResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ReadResourceResult> for ServerMessage {
    fn from_message(
        message: ReadResourceResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for ReadResourceResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListPromptsResult> for ServerMessage {
    fn from_message(
        message: ListPromptsResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for ListPromptsResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<GetPromptResult> for ServerMessage {
    fn from_message(
        message: GetPromptResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for GetPromptResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ListToolsResult> for ServerMessage {
    fn from_message(
        message: ListToolsResult,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for ListToolsResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<CallToolResult> for ServerMessage {
    fn from_message(message: CallToolResult, request_id: Option<RequestId>) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for CallToolResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<CompleteResult> for ServerMessage {
    fn from_message(message: CompleteResult, request_id: Option<RequestId>) -> std::result::Result<Self, JsonrpcErrorError> {
        let request_id =
            request_id.ok_or_else(|| JsonrpcErrorError::internal_error().with_message("request_id is None!".to_string()))?;
        Ok(ServerMessage::Response(ServerJsonrpcResponse::new(
            request_id,
            message.into(),
        )))
    }
}
impl ToMessage<ServerMessage> for CompleteResult {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<CancelledNotification> for ServerMessage {
    fn from_message(
        message: CancelledNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ServerMessage> for CancelledNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ProgressNotification> for ServerMessage {
    fn from_message(
        message: ProgressNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ServerMessage> for ProgressNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ResourceListChangedNotification> for ServerMessage {
    fn from_message(
        message: ResourceListChangedNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ServerMessage> for ResourceListChangedNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ResourceUpdatedNotification> for ServerMessage {
    fn from_message(
        message: ResourceUpdatedNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ServerMessage> for ResourceUpdatedNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<PromptListChangedNotification> for ServerMessage {
    fn from_message(
        message: PromptListChangedNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ServerMessage> for PromptListChangedNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<ToolListChangedNotification> for ServerMessage {
    fn from_message(
        message: ToolListChangedNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ServerMessage> for ToolListChangedNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl FromMessage<LoggingMessageNotification> for ServerMessage {
    fn from_message(
        message: LoggingMessageNotification,
        request_id: Option<RequestId>,
    ) -> std::result::Result<Self, JsonrpcErrorError> {
        if request_id.is_some() {
            return Err(JsonrpcErrorError::internal_error()
                .with_message("request_id expected to be None for Notifications!".to_string()));
        }
        Ok(ServerMessage::Notification(ServerJsonrpcNotification::new(message.into())))
    }
}
impl ToMessage<ServerMessage> for LoggingMessageNotification {
    fn to_message(self, request_id: Option<RequestId>) -> std::result::Result<ServerMessage, JsonrpcErrorError> {
        ServerMessage::from_message(self, request_id)
    }
}
impl TryFrom<RequestFromClient> for InitializeRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::InitializeRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a InitializeRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for PingRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::PingRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a PingRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for ListResourcesRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::ListResourcesRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListResourcesRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for ListResourceTemplatesRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::ListResourceTemplatesRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListResourceTemplatesRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for ReadResourceRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::ReadResourceRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ReadResourceRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for SubscribeRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::SubscribeRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a SubscribeRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for UnsubscribeRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::UnsubscribeRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a UnsubscribeRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for ListPromptsRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::ListPromptsRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListPromptsRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for GetPromptRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::GetPromptRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a GetPromptRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for ListToolsRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::ListToolsRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListToolsRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for CallToolRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::CallToolRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CallToolRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for SetLevelRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::SetLevelRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a SetLevelRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromClient> for CompleteRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientRequest = value.try_into()?;
        if let ClientRequest::CompleteRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CompleteRequest".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for Result {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientResult = value.try_into()?;
        if let ClientResult::Result(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a Result".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for CreateMessageResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientResult = value.try_into()?;
        if let ClientResult::CreateMessageResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CreateMessageResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromClient> for ListRootsResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientResult = value.try_into()?;
        if let ClientResult::ListRootsResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListRootsResult".to_string()))
        }
    }
}
impl TryFrom<NotificationFromClient> for CancelledNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientNotification = value.try_into()?;
        if let ClientNotification::CancelledNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CancelledNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromClient> for InitializedNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientNotification = value.try_into()?;
        if let ClientNotification::InitializedNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a InitializedNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromClient> for ProgressNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientNotification = value.try_into()?;
        if let ClientNotification::ProgressNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ProgressNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromClient> for RootsListChangedNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromClient) -> std::result::Result<Self, Self::Error> {
        let matched_type: ClientNotification = value.try_into()?;
        if let ClientNotification::RootsListChangedNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a RootsListChangedNotification".to_string()))
        }
    }
}
impl TryFrom<RequestFromServer> for PingRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerRequest = value.try_into()?;
        if let ServerRequest::PingRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a PingRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromServer> for CreateMessageRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerRequest = value.try_into()?;
        if let ServerRequest::CreateMessageRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CreateMessageRequest".to_string()))
        }
    }
}
impl TryFrom<RequestFromServer> for ListRootsRequest {
    type Error = JsonrpcErrorError;
    fn try_from(value: RequestFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerRequest = value.try_into()?;
        if let ServerRequest::ListRootsRequest(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListRootsRequest".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for Result {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::Result(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a Result".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for InitializeResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::InitializeResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a InitializeResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListResourcesResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::ListResourcesResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListResourcesResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListResourceTemplatesResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::ListResourceTemplatesResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListResourceTemplatesResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ReadResourceResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::ReadResourceResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ReadResourceResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListPromptsResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::ListPromptsResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListPromptsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for GetPromptResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::GetPromptResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a GetPromptResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for ListToolsResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::ListToolsResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ListToolsResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CallToolResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::CallToolResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CallToolResult".to_string()))
        }
    }
}
impl TryFrom<ResultFromServer> for CompleteResult {
    type Error = JsonrpcErrorError;
    fn try_from(value: ResultFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerResult = value.try_into()?;
        if let ServerResult::CompleteResult(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CompleteResult".to_string()))
        }
    }
}
impl TryFrom<NotificationFromServer> for CancelledNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerNotification = value.try_into()?;
        if let ServerNotification::CancelledNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a CancelledNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromServer> for ProgressNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerNotification = value.try_into()?;
        if let ServerNotification::ProgressNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ProgressNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromServer> for ResourceListChangedNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerNotification = value.try_into()?;
        if let ServerNotification::ResourceListChangedNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ResourceListChangedNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromServer> for ResourceUpdatedNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerNotification = value.try_into()?;
        if let ServerNotification::ResourceUpdatedNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ResourceUpdatedNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromServer> for PromptListChangedNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerNotification = value.try_into()?;
        if let ServerNotification::PromptListChangedNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a PromptListChangedNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromServer> for ToolListChangedNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerNotification = value.try_into()?;
        if let ServerNotification::ToolListChangedNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a ToolListChangedNotification".to_string()))
        }
    }
}
impl TryFrom<NotificationFromServer> for LoggingMessageNotification {
    type Error = JsonrpcErrorError;
    fn try_from(value: NotificationFromServer) -> std::result::Result<Self, Self::Error> {
        let matched_type: ServerNotification = value.try_into()?;
        if let ServerNotification::LoggingMessageNotification(result) = matched_type {
            Ok(result)
        } else {
            Err(JsonrpcErrorError::internal_error().with_message("Not a LoggingMessageNotification".to_string()))
        }
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
        let message = ClientJsonrpcRequest::new(RequestId::Integer(0), PingRequest::new(None).into());
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
        let message = ClientJsonrpcNotification::new(RootsListChangedNotification::new(None).into());
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
        let message = JsonrpcError::create(
            RequestId::Integer(0),
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
