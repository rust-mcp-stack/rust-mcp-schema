#![cfg_attr(rustfmt, rustfmt_skip)]
//! Example: handling messages received by an MCP **server** (sent by an MCP **client**)
//!
//! Demonstrates:
//! - deserializing a `ClientMessage` from a JSON-RPC payload,
//! - the delegate enums (`ClientJsonrpcRequest::Known` wraps the schema `ClientRequest`),
//! - the 2026-07-28 requirement that `params` and `params._meta` are always present,
//!   with `_meta` (`RequestMetaObject`) carrying protocolVersion/clientInfo/clientCapabilities.
use rust_mcp_schema::{schema_utils::*, *};
use std::str::FromStr;

type AppError = RpcError;

// A client→server `tools/call` request.
// 2026-07-28: `params` and `params._meta` are required; `_meta` is a `RequestMetaObject`
// whose namespaced keys carry the protocol handshake fields on every request.
const SAMPLE_PAYLOAD: &str = r#"
{
    "id": 0,
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "get_weather",
        "arguments": { "city": "Paris" },
        "_meta": {
            "io.modelcontextprotocol/protocolVersion": "2026-07-28",
            "io.modelcontextprotocol/clientInfo": { "name": "example-client", "version": "1.0.0" },
            "io.modelcontextprotocol/clientCapabilities": {}
        }
    }
}
"#;

fn main() {
    if let Err(error) = handle_message(SAMPLE_PAYLOAD) {
        eprintln!("Error occurred: {:?}", error);
    }
}

fn handle_message(message_payload: &str) -> std::result::Result<(), AppError> {
    // Deserialize into `ClientMessage` (a message sent by an MCP Client to an MCP Server).
    let mcp_message = ClientMessage::from_str(message_payload)?;

    match mcp_message {
        // Requests delegate to the schema-generated `ClientRequest` enum.
        ClientMessage::Request(request) => match request {
            ClientJsonrpcRequest::Standard(standard) => match standard {
                ClientRequest::DiscoverRequest(r) => println!("Discover (server/discover): {:?}", r),
                ClientRequest::ListResourcesRequest(r) => println!("ListResources: {:?}", r),
                ClientRequest::ListResourceTemplatesRequest(r) => println!("ListResourceTemplates: {:?}", r),
                ClientRequest::ReadResourceRequest(r) => println!("ReadResource: {:?}", r),
                ClientRequest::SubscriptionsListenRequest(r) => println!("SubscriptionsListen: {:?}", r),
                ClientRequest::ListPromptsRequest(r) => println!("ListPrompts: {:?}", r),
                ClientRequest::GetPromptRequest(r) => println!("GetPrompt: {:?}", r),
                ClientRequest::ListToolsRequest(r) => println!("ListTools: {:?}", r),
                ClientRequest::CallToolRequest(r) => println!("CallTool: {:?}", r),
                ClientRequest::CompleteRequest(r) => println!("Complete: {:?}", r),
            },
            ClientJsonrpcRequest::Custom(custom) => println!("Custom request: {:?}", custom),
        },

        // Client→server notifications (task/roots-list-changed/initialized removed in 2026-07-28).
        ClientMessage::Notification(notification) => match notification {
            ClientJsonrpcNotification::CancelledNotification(n) => println!("Cancelled: {:?}", n),
            ClientJsonrpcNotification::CustomNotification(custom) => println!("Custom notification: {:?}", custom),
        },

        // Results of server→client requests (the client's answers to sampling/roots/elicitation).
        ClientMessage::Response(response) => match &response.result {
            ResultFromClient::CreateMessageResult(r) => println!("CreateMessageResult: {:?}", r),
            ResultFromClient::ListRootsResult(r) => println!("ListRootsResult: {:?}", r),
            ResultFromClient::ElicitResult(r) => println!("ElicitResult: {:?}", r),
            ResultFromClient::Result(r) => println!("Generic Result: {:?}", r),
        },

        ClientMessage::Error(error_response) => println!("Error response: {:?}", error_response),
    }

    Ok(())
}
