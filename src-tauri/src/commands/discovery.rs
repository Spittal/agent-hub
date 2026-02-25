use serde::Serialize;
use tauri::{AppHandle, State};

use crate::commands::integrations::update_all_integration_configs;
use crate::commands::skills::{install_managed_skill, uninstall_managed_skill};
use crate::error::AppError;
use crate::mcp::proxy::ProxyState;
use crate::persistence::save_tool_discovery;
use crate::state::SharedState;

pub(crate) const DISCOVERY_SKILL_ID: &str = "using-discovery";
pub(crate) const DISCOVERY_SKILL_CONTENT: &str =
    include_str!("../../resources/using-discovery-SKILL.md");

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryStatus {
    pub enabled: bool,
}

#[tauri::command]
pub async fn get_discovery_mode(state: State<'_, SharedState>) -> Result<DiscoveryStatus, AppError> {
    let s = state.lock().unwrap();
    Ok(DiscoveryStatus {
        enabled: s.tool_discovery_enabled,
    })
}

#[tauri::command]
pub async fn set_discovery_mode(
    app: AppHandle,
    state: State<'_, SharedState>,
    proxy_state: State<'_, ProxyState>,
    enabled: bool,
) -> Result<DiscoveryStatus, AppError> {
    {
        let mut s = state.lock().unwrap();
        s.tool_discovery_enabled = enabled;
    }

    save_tool_discovery(&app, enabled);

    if enabled {
        install_managed_skill(
            &app,
            &state,
            DISCOVERY_SKILL_ID,
            "using-discovery",
            "Find and use MCP tools through the discovery endpoint",
            DISCOVERY_SKILL_CONTENT,
            "discovery",
        );
    } else {
        uninstall_managed_skill(&app, &state, DISCOVERY_SKILL_ID, "discovery");
    }

    let port = proxy_state.port().await;
    if let Err(e) = update_all_integration_configs(&app, port) {
        tracing::warn!("Failed to update integration configs after discovery toggle: {e}");
    }

    Ok(DiscoveryStatus { enabled })
}
