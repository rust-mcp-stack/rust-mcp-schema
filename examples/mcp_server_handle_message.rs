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
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "sampling": {},
            "roots": {
                "listChanged": true
            }
        },
        "clientInfo": {
            "name": "mcp-inspector",
            "version": "0.1.0"
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
    // Deserialize message into ClientMessage.
    // ClientMessage represents a message sent by an MCP Client and received by an MCP Server.
    let mcp_message = ClientMessage::from_str(message_payload)?;

    match mcp_message {
        // Check if the message is a Request
        ClientMessage::Request(client_message) => match client_message.request {
            // Check if it's a standard ClientRequest (not a CustomRequest)
            RequestFromClient::ClientRequest(client_request) => match client_request {
                ClientRequest::InitializeRequest(initialize_request) => {
                    dbg!(initialize_request);
                }

                ClientRequest::PingRequest(ping_request) => {
                    dbg!(ping_request);
                }
                ClientRequest::ListResourcesRequest(list_resources_request) => {
                    dbg!(list_resources_request);
                }

                ClientRequest::ListResourceTemplatesRequest(list_resource_templates_request) => {
                    dbg!(list_resource_templates_request);
                }

                ClientRequest::ReadResourceRequest(read_resource_request) => {
                    dbg!(read_resource_request);
                }

                ClientRequest::SubscribeRequest(subscribe_request) => {
                    dbg!(subscribe_request);
                }

                ClientRequest::UnsubscribeRequest(unsubscribe_request) => {
                    dbg!(unsubscribe_request);
                }

                ClientRequest::ListPromptsRequest(list_prompts_request) => {
                    dbg!(list_prompts_request);
                }

                ClientRequest::GetPromptRequest(get_prompt_request) => {
                    dbg!(get_prompt_request);
                }

                ClientRequest::ListToolsRequest(list_tools_request) => {
                    dbg!(list_tools_request);
                }

                ClientRequest::CallToolRequest(call_tool_request) => {
                    dbg!(call_tool_request);
                }

                ClientRequest::SetLevelRequest(set_level_request) => {
                    dbg!(set_level_request);
                }

                ClientRequest::CompleteRequest(complete_request) => {
                    dbg!(complete_request);
                }
            },

            // Check if it's a CustomRequest; the value can be deserialized into your own custom types.
            RequestFromClient::CustomRequest(value) => {
                dbg!(value);
            }
        },

        // Check if the message is a Notification
        ClientMessage::Notification(client_message) => match client_message.notification {
            // Check if it's a standard ClientNotification (not a CustomNotification)
            NotificationFromClient::ClientNotification(client_notification) => {
                // Handle different ClientNotifications
                match client_notification {
                    ClientNotification::CancelledNotification(cancelled_notification) => {
                        dbg!(cancelled_notification);
                    }

                    ClientNotification::InitializedNotification(initialized_notification) => {
                        dbg!(initialized_notification);
                    }

                    ClientNotification::ProgressNotification(progress_notification) => {
                        dbg!(progress_notification);
                    }

                    ClientNotification::RootsListChangedNotification(progress_notification) => {
                        dbg!(progress_notification);
                    }
                }
            }

            // Check if it's a CustomNotification; the value can be deserialized into your custom types.
            NotificationFromClient::CustomNotification(value) => {
                dbg!(value);
            }
        },

        // Check if the message is a Response
        ClientMessage::Response(client_message) => match client_message.result {
            // Check if it's a standard ClientResult (not a CustomResult)
            ResultFromClient::ClientResult(client_result) => match client_result {
                ClientResult::Result(_) => {
                    dbg!(client_result);
                }
                ClientResult::CreateMessageResult(create_message_result) => {
                    dbg!(create_message_result);
                }
                ClientResult::ListRootsResult(list_roots_result) => {
                    dbg!(list_roots_result);
                }
                #[cfg(any(feature = "2025_06_18", feature = "draft"))]
                ClientResult::ElicitResult(elicit_result) => {
                    dbg!(elicit_result);
                }
            },

            // Check if it's a CustomResult; the value can be deserialized into your custom types.
            ResultFromClient::CustomResult(value) => {
                dbg!(value);
            }
        },

        // Check if it's a Error message
        ClientMessage::Error(client_error) => {
            dbg!(client_error);
        }
    }
    Ok(())
}
