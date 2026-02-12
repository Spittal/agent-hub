use tauri::{AppHandle, Emitter, State};
use tracing::{error, info};

use crate::error::AppError;
use crate::mcp::client::{McpClient, SharedConnections};
use crate::state::{
    ConnectionState, McpTool, ServerStatus, ServerTransport, SharedState,
};

#[tauri::command]
pub async fn connect_server(
    app: AppHandle,
    state: State<'_, SharedState>,
    connections: State<'_, SharedConnections>,
    id: String,
) -> Result<(), AppError> {
    // Read config while holding the lock briefly
    let (command, args, env) = {
        let mut s = state.lock().unwrap();
        let server = s
            .servers
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| AppError::ServerNotFound(id.clone()))?;

        if server.status == Some(ServerStatus::Connected) {
            return Err(AppError::AlreadyConnected(id.clone()));
        }

        match server.transport {
            ServerTransport::Http => {
                return Err(AppError::Transport(
                    "HTTP transport not yet implemented".into(),
                ));
            }
            ServerTransport::Stdio => {}
        }

        let command = server
            .command
            .clone()
            .ok_or_else(|| AppError::ConnectionFailed("No command specified".into()))?;
        let args = server.args.clone().unwrap_or_default();
        let env = server.env.clone().unwrap_or_default();

        server.status = Some(ServerStatus::Connecting);

        (command, args, env)
    };

    let _ = app.emit(
        "server-status-changed",
        serde_json::json!({ "serverId": id, "status": "connecting" }),
    );

    // Do the async connection work WITHOUT holding either lock
    let client_result = McpClient::connect_stdio(&app, &command, &args, &env).await;

    match client_result {
        Ok(client) => {
            let child_pid = client.child_pid();
            let server_name;

            // Convert discovered tools to McpTool for storage in AppState
            let tools: Vec<McpTool> = {
                let s = state.lock().unwrap();
                let srv = s.servers.iter().find(|s| s.id == id);
                server_name = srv.map(|s| s.name.clone()).unwrap_or_default();

                client
                    .tools
                    .iter()
                    .map(|t| McpTool {
                        name: t.name.clone(),
                        title: t.title.clone(),
                        description: t.description.clone(),
                        input_schema: t.input_schema.clone(),
                        server_id: id.clone(),
                        server_name: server_name.clone(),
                    })
                    .collect()
            };

            info!(
                "Connected to server {id} with {} tools",
                tools.len()
            );

            // Store connection state in AppState
            {
                let mut s = state.lock().unwrap();
                if let Some(server) = s.servers.iter_mut().find(|s| s.id == id) {
                    server.status = Some(ServerStatus::Connected);
                    server.last_connected = Some(chrono_now());
                }
                s.connections.insert(
                    id.clone(),
                    ConnectionState {
                        tools: tools.clone(),
                        child_pid,
                    },
                );
            }

            // Store the live client in the connections map
            {
                let mut conns = connections.lock().await;
                conns.insert(id.clone(), client);
            }

            let _ = app.emit(
                "server-status-changed",
                serde_json::json!({ "serverId": id, "status": "connected" }),
            );
            let _ = app.emit(
                "tools-updated",
                serde_json::json!({ "serverId": id, "tools": tools }),
            );

            Ok(())
        }
        Err(e) => {
            error!("Failed to connect to server {id}: {e}");

            {
                let mut s = state.lock().unwrap();
                if let Some(server) = s.servers.iter_mut().find(|s| s.id == id) {
                    server.status = Some(ServerStatus::Error);
                }
            }

            let _ = app.emit(
                "server-status-changed",
                serde_json::json!({ "serverId": id, "status": "error", "error": e.to_string() }),
            );

            Err(e)
        }
    }
}

#[tauri::command]
pub async fn disconnect_server(
    app: AppHandle,
    state: State<'_, SharedState>,
    connections: State<'_, SharedConnections>,
    id: String,
) -> Result<(), AppError> {
    // Remove and shut down the live MCP client
    {
        let mut conns = connections.lock().await;
        if let Some(client) = conns.remove(&id) {
            client.shutdown();
        }
    }

    // Update AppState
    {
        let mut s = state.lock().unwrap();
        let server = s
            .servers
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| AppError::ServerNotFound(id.clone()))?;
        server.status = Some(ServerStatus::Disconnected);
        s.connections.remove(&id);
    }

    let _ = app.emit(
        "server-status-changed",
        serde_json::json!({ "serverId": id, "status": "disconnected" }),
    );

    info!("Disconnected server {id}");

    Ok(())
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", now)
}
