#![cfg_attr(rustfmt, rustfmt_skip)]
use rust_mcp_schema::{schema_utils::*, *};
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

fn main() {
    if let Err(error) = handle_message(SAMPLE_PAYLOAD) {
        eprintln!("Error occured: {:?}", error);
    }
}


   
// Determine if the message is an Error        
fn handle_message(message_payload: &str) -> std::result::Result<(), AppError> {
    // Deserialize message into ServerMessage.
    //ServerMessage is a message sent by an MCP Server to an MCP Client.
    let mcp_message = ServerMessage::from_str(message_payload)?;
    match mcp_message {
        // Determine if the message is a Request
        ServerMessage::Request(request) => match request {
            ServerJsonrpcRequest::PingRequest(ping_request) => println!("Ping request received: {:?}", ping_request),
            ServerJsonrpcRequest::GetTaskRequest(get_task_request) => println!("GetTaskRequest request received: {:?}", get_task_request),            
            ServerJsonrpcRequest::GetTaskPayloadRequest(get_task_payload_request) => println!("GetTaskPayloadRequest request received: {:?}", get_task_payload_request),
            ServerJsonrpcRequest::CancelTaskRequest(cancel_task_request) => println!("CancelTaskRequest request received: {:?}", cancel_task_request),
            ServerJsonrpcRequest::ListTasksRequest(list_tasks_request) => println!("ListTasksRequest request received: {:?}", list_tasks_request),
            ServerJsonrpcRequest::CreateMessageRequest(create_message_request) => println!("CreateMessageRequest request received: {:?}", create_message_request),
            ServerJsonrpcRequest::ListRootsRequest(list_roots_request) => println!("ListRootsRequest request received: {:?}", list_roots_request),
            ServerJsonrpcRequest::ElicitRequest(elicit_request) => println!("ElicitRequest request received: {:?}", elicit_request),            
            ServerJsonrpcRequest::CustomRequest(jsonrpc_request) => println!("CustomRequest request received: {:?}", jsonrpc_request),
        },
        // Determine if the message is a Notification
        ServerMessage::Notification(notification) => match notification {
            ServerJsonrpcNotification::CancelledNotification(cancelled_notification) => println!("CancelledNotification notification received: {:?}", cancelled_notification),            
            ServerJsonrpcNotification::ProgressNotification(progress_notification) => println!("ProgressNotification notification received: {:?}", progress_notification),    
            ServerJsonrpcNotification::ResourceListChangedNotification(resource_list_changed_notification) => println!("ResourceListChangedNotification notification received: {:?}",resource_list_changed_notification),
            ServerJsonrpcNotification::ResourceUpdatedNotification(resource_updated_notification) => println!("ResourceUpdatedNotification notification received: {:?}",resource_updated_notification),
            ServerJsonrpcNotification::PromptListChangedNotification(prompt_list_changed_notification) => println!("PromptListChangedNotification notification received: {:?}",prompt_list_changed_notification),
            ServerJsonrpcNotification::ToolListChangedNotification(tool_list_changed_notification) => println!("ToolListChangedNotification notification received: {:?}",tool_list_changed_notification),
            ServerJsonrpcNotification::TaskStatusNotification(task_status_notification) => {println!("TaskStatusNotification notification received: {:?}", task_status_notification)}
            ServerJsonrpcNotification::LoggingMessageNotification(logging_message_notification) => println!("LoggingMessageNotification notification received: {:?}",logging_message_notification),
            ServerJsonrpcNotification::ElicitationCompleteNotification(elicitation_complete_notification) => println!("ElicitationCompleteNotification notification received: {:?}",elicitation_complete_notification),
            ServerJsonrpcNotification::CustomNotification(jsonrpc_notification) => println!("CustomNotification notification received: {:?}", jsonrpc_notification)
        },
    // Determine if the message is a Response 
    ServerMessage::Response(response) => match &response.result {
            ServerResult::InitializeResult(_initialize_result) =>  println!("InitializeResult response received: {:?}", response),            
            ServerResult::ListResourcesResult(_list_resources_result) =>  println!("ListResourcesResult response received: {:?}", response),            
            ServerResult::ListResourceTemplatesResult(_list_resource_templates_result) =>  println!("ListResourceTemplatesResult response received: {:?}", response),            
            ServerResult::ReadResourceResult(_read_resource_result) =>  println!("ReadResourceResult response received: {:?}", response),            
            ServerResult::ListPromptsResult(_list_prompts_result) =>  println!("ListPromptsResult response received: {:?}", response),        
            ServerResult::GetPromptResult(_get_prompt_result) => println!("GetPromptResult response received: {:?}", response),
            ServerResult::ListToolsResult(_list_tools_result) => println!("ListToolsResult response received: {:?}", response),
            ServerResult::CallToolResult(_call_tool_result) => println!("CallToolResult response received: {:?}", response),
            ServerResult::GetTaskResult(_get_task_result) => println!("GetTaskResult response received: {:?}", response),
            ServerResult::CancelTaskResult(_cancel_task_result) => println!("CancelTaskResult response received: {:?}", response),            
            ServerResult::ListTasksResult(_list_tasks_result) => println!("ListTasksResult response received: {:?}", response),
            ServerResult::CompleteResult(_complete_result) => println!("CompleteResult response received: {:?}", response),
            ServerResult::Result(_generic_result) => println!("Generic Result response received: {:?}", response),
            ServerResult::GetTaskPayloadResult(_generic_result) => println!("Generic Result response received: {:?}", response),
        },
        ServerMessage::Error(error_response) => {
            println!("Error response received: {:?}", error_response)
        }
    }

    Ok(())
}
