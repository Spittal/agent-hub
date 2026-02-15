use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::mcp::client::{CallToolResult, McpClient, SharedConnections};
use crate::state::{McpTool, SharedState};

#[tauri::command]
pub async fn list_tools(
    state: State<'_, SharedState>,
    id: String,
) -> Result<Vec<McpTool>, AppError> {
    let s = state.lock().unwrap();
    let conn = s
        .connections
        .get(&id)
        .ok_or_else(|| AppError::ServerNotFound(id.clone()))?;
    Ok(conn.tools.clone())
}

#[tauri::command]
pub async fn list_all_tools(state: State<'_, SharedState>) -> Result<Vec<McpTool>, AppError> {
    let s = state.lock().unwrap();
    let mut all_tools: Vec<McpTool> = Vec::new();
    for conn in s.connections.values() {
        for tool in &conn.tools {
            let mut namespaced = tool.clone();
            namespaced.name = format!("{}.{}", tool.server_name, tool.name);
            all_tools.push(namespaced);
        }
    }
    Ok(all_tools)
}

#[tauri::command]
pub async fn call_tool(
    connections: State<'_, SharedConnections>,
    server_id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<CallToolResult, AppError> {
    // Clone the Arc handle and drop the lock before async I/O
    let client: Arc<McpClient> = {
        let conns = connections.lock().await;
        conns
            .get(&server_id)
            .cloned()
            .ok_or_else(|| AppError::ServerNotFound(server_id.clone()))?
    };
    client.call_tool(&tool_name, arguments).await
}
