use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tracing::{error, info};

use crate::state::ServerConfig;

const STORE_FILE: &str = "config.json";
const SERVERS_KEY: &str = "servers";

/// Load saved server configurations from the persistent store.
/// Returns an empty Vec if no data is stored yet or deserialization fails.
pub fn load_servers(app: &AppHandle) -> Vec<ServerConfig> {
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to open store: {e}");
            return Vec::new();
        }
    };

    match store.get(SERVERS_KEY) {
        Some(value) => match serde_json::from_value::<Vec<ServerConfig>>(value.clone()) {
            Ok(servers) => {
                info!("Loaded {} server configs from store", servers.len());
                servers
            }
            Err(e) => {
                error!("Failed to deserialize servers from store: {e}");
                Vec::new()
            }
        },
        None => {
            info!("No saved servers found in store");
            Vec::new()
        }
    }
}

/// Save server configurations to the persistent store.
pub fn save_servers(app: &AppHandle, servers: &[ServerConfig]) {
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to open store for saving: {e}");
            return;
        }
    };

    let value = match serde_json::to_value(servers) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to serialize servers: {e}");
            return;
        }
    };

    store.set(SERVERS_KEY, value);

    if let Err(e) = store.save() {
        error!("Failed to save store to disk: {e}");
    } else {
        info!("Saved {} server configs to store", servers.len());
    }
}
