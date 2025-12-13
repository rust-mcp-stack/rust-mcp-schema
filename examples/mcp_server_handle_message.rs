#![cfg_attr(rustfmt, rustfmt_skip)]
#[cfg(feature = "latest")]
use rust_mcp_schema::{schema_utils::*, *};
use std::str::FromStr;

#[cfg(feature = "latest")]
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

fn main() {
    #[cfg(feature = "latest")]
    if let Err(error) = handle_message(SAMPLE_PAYLOAD) {
        eprintln!("Error occurred: {:?}", error);
    }
}

#[cfg(feature = "latest")]
/// Deserialize the JSON-RPC message into the appropriate MCP type and print it with dbg!() macro .
fn handle_message(message_payload: &str) -> std::result::Result<(), AppError> {
    // Deserialize message into ClientMessage.
    // ClientMessage represents a message sent by an MCP Client and received by an MCP Server.
    let mcp_message = ClientMessage::from_str(message_payload)?;

    match mcp_message {
        // Determine if the message is a Request
        ClientMessage::Request(request) => match request {
            ClientJsonrpcRequest::InitializeRequest(initialize_request) => println!("InitializeRequest request received: {:?}", initialize_request),
            ClientJsonrpcRequest::PingRequest(ping_request) => println!("PingRequest request received: {:?}", ping_request),
            ClientJsonrpcRequest::ListResourcesRequest(list_resources_request) => println!("ListResourcesRequest request received: {:?}", list_resources_request),
            ClientJsonrpcRequest::ListResourceTemplatesRequest(list_resource_templates_request) => println!("ListResourceTemplatesRequest request received: {:?}",list_resource_templates_request),
            ClientJsonrpcRequest::ReadResourceRequest(read_resource_request) => println!("ReadResourceRequest request received: {:?}", read_resource_request),
            ClientJsonrpcRequest::SubscribeRequest(subscribe_request) => println!("SubscribeRequest request received: {:?}", subscribe_request),
            ClientJsonrpcRequest::UnsubscribeRequest(unsubscribe_request) => println!("UnsubscribeRequest request received: {:?}", unsubscribe_request),
            ClientJsonrpcRequest::ListPromptsRequest(list_prompts_request) => println!("ListPromptsRequest request received: {:?}", list_prompts_request),
            ClientJsonrpcRequest::GetPromptRequest(get_prompt_request) => println!("GetPromptRequest request received: {:?}", get_prompt_request),
            ClientJsonrpcRequest::ListToolsRequest(list_tools_request) => println!("ListToolsRequest request received: {:?}", list_tools_request),
            ClientJsonrpcRequest::CallToolRequest(call_tool_request) => println!("CallToolRequest request received: {:?}", call_tool_request),
            ClientJsonrpcRequest::GetTaskRequest(get_task_request) => println!("GetTaskRequest request received: {:?}", get_task_request),
            ClientJsonrpcRequest::GetTaskPayloadRequest(get_task_payload_request) => println!("GetTaskPayloadRequest request received: {:?}", get_task_payload_request),
            ClientJsonrpcRequest::CancelTaskRequest(cancel_task_request) => println!("CancelTaskRequest request received: {:?}", cancel_task_request),
            ClientJsonrpcRequest::ListTasksRequest(list_tasks_request) => println!("ListTasksRequest request received: {:?}", list_tasks_request),
            ClientJsonrpcRequest::SetLevelRequest(set_level_request) => println!("SetLevelRequest request received: {:?}", set_level_request),
            ClientJsonrpcRequest::CompleteRequest(complete_request) => println!("CompleteRequest request received: {:?}", complete_request),
            ClientJsonrpcRequest::CustomRequest(jsonrpc_request) => println!("CustomRequest request received: {:?}", jsonrpc_request),
        },
        // Determine if the message is a Notification
        ClientMessage::Notification(notification) => match notification {
            ClientJsonrpcNotification::CancelledNotification(cancelled_notification) => println!("CancelledNotification notification received: {:?}", cancelled_notification),
            ClientJsonrpcNotification::InitializedNotification(initialized_notification) => println!("InitializedNotification notification received: {:?}",initialized_notification),
            ClientJsonrpcNotification::ProgressNotification(progress_notification) => println!("ProgressNotification notification received: {:?}", progress_notification),
            ClientJsonrpcNotification::TaskStatusNotification(task_status_notification) => println!("TaskStatusNotification notification received: {:?}", task_status_notification),
            ClientJsonrpcNotification::RootsListChangedNotification(roots_list_changed_notification) => println!("RootsListChangedNotification notification received: {:?}",roots_list_changed_notification),
            ClientJsonrpcNotification::CustomNotification(jsonrpc_notification) => println!("CustomNotification notification received: {:?}", jsonrpc_notification),
        },
        // Determine if the message is a Response
        ClientMessage::Response(response) => match &response.result {
            ClientResult::GetTaskResult(_get_task_result) => println!("GetTaskResult  response received: {:?}", response),
            ClientResult::CancelTaskResult(_cancel_task_result) => println!("CancelTaskResult  response received: {:?}", response),
            ClientResult::ListTasksResult(_list_tasks_result) => println!("ListTasksResult  response received: {:?}", response),
            ClientResult::CreateMessageResult(_create_message_result) => println!("CreateMessageResult  response received: {:?}", response),
            ClientResult::ListRootsResult(_list_roots_result) => println!("ListRootsResult  response received: {:?}", response),
            ClientResult::ElicitResult(_elicit_result) => println!("ElicitResult  response received: {:?}", response),
            ClientResult::Result(_generic_result) => println!("Generic Result response received: {:?}", response),
            ClientResult::GetTaskPayloadResult(_generic_result) => println!("Generic Result response received: {:?}", response),
        },
        // Determine if the message is an Error
        ClientMessage::Error(error_response) => {
            println!("Error response received: {:?}", error_response)
        }
    }

    Ok(())
}
