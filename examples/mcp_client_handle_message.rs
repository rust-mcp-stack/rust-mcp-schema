#[cfg(feature = "latest")]
mod schema {
    pub use rust_mcp_schema::schema_utils::*;
    pub use rust_mcp_schema::*;
}

#[cfg(feature = "2024_11_05")]
mod schema {
    pub use rust_mcp_schema::mcp_2024_11_05::schema_utils::*;
    pub use rust_mcp_schema::mcp_2024_11_05::*;
}

#[cfg(feature = "2025_03_26")]
mod schema {
    pub use rust_mcp_schema::mcp_2025_03_26::schema_utils::*;
    pub use rust_mcp_schema::mcp_2025_03_26::*;
}

#[cfg(feature = "draft")]
mod schema {
    pub use rust_mcp_schema::mcp_draft::schema_utils::*;
    pub use rust_mcp_schema::mcp_draft::*;
}

use schema::*;

use std::str::FromStr;

type AppError = RpcError;

const SAMPLE_PAYLOAD: &str = r#"
{
    "id": 0,
    "jsonrpc": "2.0",
    "result": {
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "prompts": {},
            "resources": {
                "subscribe": true
            },
            "tools": {},
            "logging": {}
        },
        "serverInfo": {
            "name": "example-servers/everything",
            "version": "1.0.0"
        }
    }
}
"#;

fn main() -> std::result::Result<(), AppError> {
    handle_message(SAMPLE_PAYLOAD)?;
    Ok(())
}

/// Deserialize the JSON-RPC message into the appropriate MCP type and print it with dbg!() macro .
fn handle_message(message_payload: &str) -> std::result::Result<(), AppError> {
    // Deserialize message into ServerMessage.
    // ServerMessage represents a message sent by an MCP Server and received by an MCP Client.
    let mcp_message = ServerMessage::from_str(message_payload)?;

    match mcp_message {
        // Check if the message is a Request
        ServerMessage::Request(server_message) => match server_message.request {
            // Check if it's a standard ServerRequest (not a CustomRequest)
            RequestFromServer::ServerRequest(server_request) => {
                // Handle different ServerNotifications
                match server_request {
                    ServerRequest::PingRequest(ping_request) => {
                        dbg!(ping_request);
                    }
                    ServerRequest::CreateMessageRequest(create_message_request) => {
                        dbg!(create_message_request);
                    }
                    ServerRequest::ListRootsRequest(list_roots_request) => {
                        dbg!(list_roots_request);
                    }
                    #[cfg(any(feature = "2025_06_18", feature = "draft"))]
                    ServerRequest::ElicitRequest(elicit_request) => {
                        dbg!(elicit_request);
                    }
                }
            }
            // Check if it's a CustomRequest; the value can be deserialized into your own custom types.
            RequestFromServer::CustomRequest(value) => {
                dbg!(value);
            }
        },
        // Check if the message is a Notification
        ServerMessage::Notification(server_message) => match server_message.notification {
            // Check if it's a standard ServerNotification (not a CustomNotification)
            NotificationFromServer::ServerNotification(server_notification) => {
                // Handle different ServerNotifications
                match server_notification {
                    ServerNotification::CancelledNotification(cancelled_notification) => {
                        dbg!(cancelled_notification);
                    }
                    ServerNotification::ProgressNotification(progress_notification) => {
                        dbg!(progress_notification);
                    }
                    ServerNotification::ResourceListChangedNotification(resource_list_changed_notification) => {
                        dbg!(resource_list_changed_notification);
                    }
                    ServerNotification::ResourceUpdatedNotification(resource_updated_notification) => {
                        dbg!(resource_updated_notification);
                    }
                    ServerNotification::PromptListChangedNotification(prompt_list_changed_notification) => {
                        dbg!(prompt_list_changed_notification);
                    }
                    ServerNotification::ToolListChangedNotification(tool_list_changed_notification) => {
                        dbg!(tool_list_changed_notification);
                    }
                    ServerNotification::LoggingMessageNotification(logging_message_notification) => {
                        dbg!(logging_message_notification);
                    }
                }
            }
            // Check if it's a CustomNotification; the value can be deserialized into your custom types.
            NotificationFromServer::CustomNotification(value) => {
                dbg!(value);
            }
        },
        // Check if the message is a Response
        ServerMessage::Response(server_message) => match server_message.result {
            // Check if it's a standard ServerResult (not a CustomResult)
            ResultFromServer::ServerResult(server_result) => match server_result {
                ServerResult::Result(result) => {
                    dbg!(result);
                }
                ServerResult::InitializeResult(initialize_result) => {
                    dbg!(initialize_result);
                }
                ServerResult::ListResourcesResult(list_resources_result) => {
                    dbg!(list_resources_result);
                }
                ServerResult::ListResourceTemplatesResult(list_resource_templates_result) => {
                    dbg!(list_resource_templates_result);
                }
                ServerResult::ReadResourceResult(read_resource_result) => {
                    dbg!(read_resource_result);
                }
                ServerResult::ListPromptsResult(list_prompts_result) => {
                    dbg!(list_prompts_result);
                }
                ServerResult::GetPromptResult(get_prompt_result) => {
                    dbg!(get_prompt_result);
                }
                ServerResult::ListToolsResult(list_tools_result) => {
                    dbg!(list_tools_result);
                }
                ServerResult::CallToolResult(call_tool_result) => {
                    dbg!(call_tool_result);
                }
                ServerResult::CompleteResult(complete_result) => {
                    dbg!(complete_result);
                }
            },
            // Check if it's a CustomResult; the value can be deserialized into your custom types.
            ResultFromServer::CustomResult(value) => {
                dbg!(value);
            }
        },
        // Check if it's a Error message
        ServerMessage::Error(server_message) => {
            dbg!(server_message);
        }
    }
    Ok(())
}
