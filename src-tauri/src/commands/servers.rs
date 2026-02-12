use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::error::AppError;
use crate::persistence::save_servers;
use crate::state::{ServerConfig, ServerConfigInput, ServerStatus, SharedState};

#[tauri::command]
pub async fn list_servers(state: State<'_, SharedState>) -> Result<Vec<ServerConfig>, AppError> {
    let state = state.lock().unwrap();
    Ok(state.servers.clone())
}

#[tauri::command]
pub async fn add_server(
    app: AppHandle,
    state: State<'_, SharedState>,
    input: ServerConfigInput,
) -> Result<ServerConfig, AppError> {
    let server = ServerConfig {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        enabled: input.enabled,
        transport: input.transport,
        command: input.command,
        args: input.args,
        env: input.env,
        url: input.url,
        headers: input.headers,
        tags: input.tags,
        status: Some(ServerStatus::Disconnected),
        last_connected: None,
    };

    let mut state = state.lock().unwrap();
    state.servers.push(server.clone());
    save_servers(&app, &state.servers);
    Ok(server)
}

#[tauri::command]
pub async fn remove_server(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: String,
) -> Result<(), AppError> {
    let mut state = state.lock().unwrap();
    let len_before = state.servers.len();
    state.servers.retain(|s| s.id != id);
    if state.servers.len() == len_before {
        return Err(AppError::ServerNotFound(id));
    }
    state.connections.remove(&id);
    save_servers(&app, &state.servers);
    Ok(())
}

#[tauri::command]
pub async fn update_server(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: String,
    input: ServerConfigInput,
) -> Result<ServerConfig, AppError> {
    let mut s = state.lock().unwrap();
    let server = s
        .servers
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| AppError::ServerNotFound(id.clone()))?;

    server.name = input.name;
    server.transport = input.transport;
    server.command = input.command;
    server.args = input.args;
    server.env = input.env;
    server.url = input.url;
    server.headers = input.headers;
    server.enabled = input.enabled;
    server.tags = input.tags;

    let updated = server.clone();
    save_servers(&app, &s.servers);
    Ok(updated)
}
