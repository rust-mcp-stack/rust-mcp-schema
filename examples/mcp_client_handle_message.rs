#![cfg_attr(rustfmt, rustfmt_skip)]
//! Example: handling messages received by an MCP **client** (sent by an MCP **server**)
//!
//! Demonstrates:
//! - deserializing a `ServerMessage` from a JSON-RPC payload,
//! - the delegate enums (`ServerJsonrpcNotification::Known` wraps the schema `ServerNotification`),
//! - the `resultType` discriminator and the `is_input_required()` / `as_input_required()`
//!   helpers that drive the 2026-07-28 mid-request (elicitation) flow.
use rust_mcp_schema::{schema_utils::*, *};
use std::str::FromStr;

type AppError = RpcError;

// A server→client *response* to a `tools/call` request.
// 2026-07-28: every result carries a required `resultType` ("complete" | "input_required").
const SAMPLE_PAYLOAD: &str = r#"
{
    "id": 0,
    "jsonrpc": "2.0",
    "result": {
        "resultType": "complete",
        "content": [{ "type": "text", "text": "Hello from the tool" }]
    }
}
"#;

fn main() {
    if let Err(error) = handle_message(SAMPLE_PAYLOAD) {
        eprintln!("Error occurred: {:?}", error);
    }
}

fn handle_message(message_payload: &str) -> std::result::Result<(), AppError> {
    // Deserialize into `ServerMessage` (a message sent by an MCP Server to an MCP Client).
    let mcp_message = ServerMessage::from_str(message_payload)?;

    match mcp_message {
        // Server→client request. In 2026-07-28 these arrive either standalone or embedded
        // inside an `InputRequiredResult` (see Response handling below).
        ServerMessage::Request(request) => match request {
            ServerJsonrpcRequest::CreateMessageRequest { request, .. } => println!("CreateMessage (sampling) request: {:?}", request),
            ServerJsonrpcRequest::ListRootsRequest { request, .. } => println!("ListRoots request: {:?}", request),
            ServerJsonrpcRequest::ElicitRequest { request, .. } => println!("Elicit request: {:?}", request),
            ServerJsonrpcRequest::CustomRequest(custom) => println!("Custom request: {:?}", custom),
        },

        // Notifications delegate to the schema-generated `ServerNotification` enum.
        ServerMessage::Notification(notification) => match notification {
            ServerJsonrpcNotification::Standard(standard) => match standard {
                ServerNotification::CancelledNotification(n) => println!("Cancelled: {:?}", n),
                ServerNotification::ProgressNotification(n) => println!("Progress: {:?}", n),
                ServerNotification::ResourceListChangedNotification(n) => println!("ResourceListChanged: {:?}", n),
                ServerNotification::ResourceUpdatedNotification(n) => println!("ResourceUpdated (deprecated): {:?}", n),
                ServerNotification::PromptListChangedNotification(n) => println!("PromptListChanged: {:?}", n),
                ServerNotification::ToolListChangedNotification(n) => println!("ToolListChanged: {:?}", n),
                ServerNotification::LoggingMessageNotification(n) => println!("LoggingMessage: {:?}", n),
                ServerNotification::SubscriptionsAcknowledgedNotification(n) => println!("SubscriptionsAcknowledged: {:?}", n),
            },
            ServerJsonrpcNotification::Custom(custom) => println!("Custom notification: {:?}", custom),
        },

        // Result of a client→server request. `resultType` distinguishes a final result
        // ("complete") from a request for more input ("input_required").
        ServerMessage::Response(response) => {
            let result = &response.result;

            if result.is_input_required() {
                // Mid-request elicitation: the server paused and needs the client to answer
                // one or more `InputRequest`s before it can complete the original request.
                let input_required = result.as_input_required().expect("checked is_input_required()");
                let pending = input_required.input_requests.as_ref().map(|r| r.0.len()).unwrap_or(0);
                println!("Input required — resolve {} request(s) and retry: {:?}", pending, input_required);
            } else {
                match result {
                    ServerResult::CallToolResult(r) => println!("CallToolResult: {:?}", r),
                    ServerResult::DiscoverResult(r) => println!("DiscoverResult: {:?}", r),
                    ServerResult::ListResourcesResult(r) => println!("ListResourcesResult: {:?}", r),
                    ServerResult::ListResourceTemplatesResult(r) => println!("ListResourceTemplatesResult: {:?}", r),
                    ServerResult::ReadResourceResult(r) => println!("ReadResourceResult: {:?}", r),
                    ServerResult::ListPromptsResult(r) => println!("ListPromptsResult: {:?}", r),
                    ServerResult::GetPromptResult(r) => println!("GetPromptResult: {:?}", r),
                    ServerResult::ListToolsResult(r) => println!("ListToolsResult: {:?}", r),
                    ServerResult::CompleteResult(r) => println!("CompleteResult: {:?}", r),
                    ServerResult::SubscriptionsListenResult(r) => println!("SubscriptionsListenResult: {:?}", r),
                    ServerResult::Result(r) => println!("Generic Result: {:?}", r),
                    ServerResult::InputRequiredResult(_) => unreachable!("handled by is_input_required() above"),
                }
            }
        }

        ServerMessage::Error(error_response) => println!("Error response: {:?}", error_response),
    }

    Ok(())
}
