use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, State};
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::AppError;
use crate::mcp::proxy::ProxyState;
use crate::persistence::save_servers;
use crate::state::{ServerConfig, ServerStatus, ServerTransport, SharedState};

/// Internal definition for a supported AI tool.
struct ToolDef {
    id: String,
    name: String,
    config_path: PathBuf,
    detection_paths: Vec<PathBuf>,
}

/// An existing MCP server found in a tool's config file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingMcpServer {
    /// The key in the mcpServers object (e.g. "grafana-dev").
    pub name: String,
    pub transport: String,
    /// For stdio: the command.
    pub command: Option<String>,
    /// For stdio: arguments.
    pub args: Option<Vec<String>>,
    /// For http: the URL.
    pub url: Option<String>,
}

/// Info about an AI tool, sent to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiToolInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub enabled: bool,
    pub config_path: String,
    pub configured_port: u16,
    /// Existing MCP servers in this tool's config that could be migrated.
    pub existing_servers: Vec<ExistingMcpServer>,
}

fn get_tool_definitions(home: &Path) -> Vec<ToolDef> {
    let mut tools = vec![
        ToolDef {
            id: "claude-code".into(),
            name: "Claude Code".into(),
            config_path: home.join(".claude").join("mcp.json"),
            detection_paths: vec![home.join(".claude")],
        },
        ToolDef {
            id: "cursor".into(),
            name: "Cursor".into(),
            config_path: home.join(".cursor").join("mcp.json"),
            detection_paths: vec![
                home.join(".cursor"),
                PathBuf::from("/Applications/Cursor.app"),
            ],
        },
        ToolDef {
            id: "claude-desktop".into(),
            name: "Claude Desktop".into(),
            config_path: home
                .join("Library/Application Support/Claude/claude_desktop_config.json"),
            detection_paths: vec![PathBuf::from("/Applications/Claude.app")],
        },
    ];

    // Windsurf: check both possible locations, prefer existing config
    let codeium_path = home.join(".codeium/windsurf/mcp_config.json");
    let windsurf_path = home.join(".windsurf/mcp.json");
    let config_path = if windsurf_path.exists() {
        windsurf_path
    } else {
        codeium_path
    };

    tools.push(ToolDef {
        id: "windsurf".into(),
        name: "Windsurf".into(),
        config_path,
        detection_paths: vec![
            home.join(".codeium/windsurf"),
            home.join(".windsurf"),
            PathBuf::from("/Applications/Windsurf.app"),
        ],
    });

    tools
}

fn find_tool_def(home: &Path, id: &str) -> Result<ToolDef, AppError> {
    get_tool_definitions(home)
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| AppError::IntegrationNotFound(id.to_string()))
}

/// Parse a tool's config file and return (enabled, port, existing_servers).
fn parse_config(path: &Path) -> (bool, u16, Vec<ExistingMcpServer>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (false, 0, Vec::new()),
    };
    let config: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (false, 0, Vec::new()),
    };

    let servers_obj = match config.get("mcpServers").and_then(|v| v.as_object()) {
        Some(obj) => obj,
        None => return (false, 0, Vec::new()),
    };

    let mut enabled = false;
    let mut port: u16 = 0;
    let mut existing = Vec::new();

    for (key, value) in servers_obj {
        if key == "mcp-manager" {
            enabled = true;
            if let Some(url) = value.get("url").and_then(|u| u.as_str()) {
                port = extract_port_from_url(url);
            }
            continue;
        }

        // Determine transport type and build ExistingMcpServer
        let has_url = value.get("url").and_then(|v| v.as_str()).is_some();
        let has_command = value.get("command").and_then(|v| v.as_str()).is_some();

        existing.push(ExistingMcpServer {
            name: key.clone(),
            transport: if has_url { "http".into() } else { "stdio".into() },
            command: value.get("command").and_then(|v| v.as_str()).map(String::from),
            args: value.get("args").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
            url: if has_url {
                value.get("url").and_then(|v| v.as_str()).map(String::from)
            } else if !has_command {
                // Some entries might only have url
                None
            } else {
                None
            },
        });
    }

    (enabled, port, existing)
}

/// Extract port number from a URL like "http://localhost:12345/mcp".
fn extract_port_from_url(url: &str) -> u16 {
    if let Ok(parsed) = url::Url::parse(url) {
        return parsed.port().unwrap_or(0);
    }
    0
}

fn home_dir() -> Result<PathBuf, AppError> {
    dirs::home_dir().ok_or_else(|| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Home directory not found",
        ))
    })
}

#[tauri::command]
pub async fn detect_integrations(
    proxy_state: State<'_, ProxyState>,
) -> Result<Vec<AiToolInfo>, AppError> {
    let home = home_dir()?;
    let tools = get_tool_definitions(&home);
    let _port = proxy_state.port().await;

    let mut results = Vec::new();
    for tool in tools {
        let installed = tool.detection_paths.iter().any(|p| p.exists());
        let (enabled, configured_port, existing_servers) = if installed {
            parse_config(&tool.config_path)
        } else {
            (false, 0, Vec::new())
        };

        results.push(AiToolInfo {
            id: tool.id,
            name: tool.name,
            installed,
            enabled,
            config_path: tool.config_path.display().to_string(),
            configured_port,
            existing_servers,
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn enable_integration(
    app: AppHandle,
    proxy_state: State<'_, ProxyState>,
    state: State<'_, SharedState>,
    id: String,
) -> Result<AiToolInfo, AppError> {
    let home = home_dir()?;
    let tool = find_tool_def(&home, &id)?;
    let port = proxy_state.port().await;

    // Read existing config to find servers to migrate
    let existing_config: serde_json::Value = if tool.config_path.exists() {
        let content = std::fs::read_to_string(&tool.config_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Import existing MCP servers into MCP Manager
    let mut imported_count = 0;
    if let Some(servers_obj) = existing_config.get("mcpServers").and_then(|v| v.as_object()) {
        let mut s = state.lock().unwrap();
        let existing_names: Vec<String> = s.servers.iter().map(|srv| srv.name.clone()).collect();

        for (key, value) in servers_obj {
            if key == "mcp-manager" {
                continue;
            }

            // Skip if a server with this name already exists in MCP Manager
            if existing_names.contains(key) {
                info!("Skipping import of '{key}' — already exists in MCP Manager");
                continue;
            }

            let has_url = value.get("url").and_then(|v| v.as_str()).is_some();

            let server = ServerConfig {
                id: Uuid::new_v4().to_string(),
                name: key.clone(),
                enabled: true,
                transport: if has_url {
                    ServerTransport::Http
                } else {
                    ServerTransport::Stdio
                },
                command: value
                    .get("command")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                args: value.get("args").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                }),
                env: value.get("env").and_then(|v| v.as_object()).map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<String, String>>()
                }),
                url: if has_url {
                    value.get("url").and_then(|v| v.as_str()).map(String::from)
                } else {
                    None
                },
                headers: value.get("headers").and_then(|v| v.as_object()).map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<String, String>>()
                }),
                tags: None,
                status: Some(ServerStatus::Disconnected),
                last_connected: None,
                managed: None,
            };

            info!("Imported MCP server '{}' from {}", key, tool.name);
            s.servers.push(server);
            imported_count += 1;
        }

        save_servers(&app, &s.servers);
    }

    if imported_count > 0 {
        info!(
            "Imported {imported_count} MCP server(s) from {}",
            tool.name
        );
        crate::tray::rebuild_tray_menu(&app);
    }

    // Write config with ONLY the mcp-manager proxy entry
    // (imported servers are now managed by MCP Manager)
    let config = serde_json::json!({
        "mcpServers": {
            "mcp-manager": {
                "url": format!("http://localhost:{port}/mcp")
            }
        }
    });

    if let Some(parent) = tool.config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&tool.config_path, content)?;

    info!(
        "Enabled MCP Manager integration for {} (port {})",
        tool.name, port
    );

    Ok(AiToolInfo {
        id: tool.id,
        name: tool.name,
        installed: true,
        enabled: true,
        config_path: tool.config_path.display().to_string(),
        configured_port: port,
        existing_servers: Vec::new(),
    })
}

#[tauri::command]
pub async fn disable_integration(id: String) -> Result<AiToolInfo, AppError> {
    let home = home_dir()?;
    let tool = find_tool_def(&home, &id)?;

    if !tool.config_path.exists() {
        return Ok(AiToolInfo {
            id: tool.id,
            name: tool.name,
            installed: true,
            enabled: false,
            config_path: tool.config_path.display().to_string(),
            configured_port: 0,
            existing_servers: Vec::new(),
        });
    }

    let content = std::fs::read_to_string(&tool.config_path)?;
    let mut config: serde_json::Value = serde_json::from_str(&content)?;

    // Remove only the mcp-manager key
    if let Some(servers) = config.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
        servers.remove("mcp-manager");
    }

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&tool.config_path, content)?;

    info!("Disabled MCP Manager integration for {}", tool.name);

    let (_, _, existing_servers) = parse_config(&tool.config_path);

    Ok(AiToolInfo {
        id: tool.id,
        name: tool.name,
        installed: true,
        enabled: false,
        config_path: tool.config_path.display().to_string(),
        configured_port: 0,
        existing_servers,
    })
}

/// Update the proxy port in all enabled integration configs.
/// Called from proxy startup — not a Tauri command.
pub fn update_enabled_integration_ports(port: u16) -> Result<(), AppError> {
    let home = home_dir()?;
    let tools = get_tool_definitions(&home);

    for tool in tools {
        if !tool.config_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&tool.config_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut config: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Only update if mcp-manager is already configured
        let has_entry = config
            .get("mcpServers")
            .and_then(|s| s.get("mcp-manager"))
            .is_some();

        if has_entry {
            config["mcpServers"]["mcp-manager"] = serde_json::json!({
                "url": format!("http://localhost:{port}/mcp")
            });

            match serde_json::to_string_pretty(&config) {
                Ok(updated) => {
                    if let Err(e) = std::fs::write(&tool.config_path, updated) {
                        warn!("Failed to update port for {}: {e}", tool.name);
                    } else {
                        info!("Updated {} config with proxy port {port}", tool.name);
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize config for {}: {e}", tool.name);
                }
            }
        }
    }

    Ok(())
}
