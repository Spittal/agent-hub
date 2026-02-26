use std::path::PathBuf;

use tracing::info;

pub mod connections;
pub mod data_management;
pub mod discovery;
pub mod integrations;
pub mod memories;
pub mod memory;
pub mod oauth;
pub mod plugins;
pub mod proxy;
pub mod registry;
pub mod servers;
pub mod skills;
pub mod skills_config;
pub mod stats;
pub mod status;
pub mod tools;

// ---------------------------------------------------------------------------
// Shared CLI helpers
// ---------------------------------------------------------------------------

/// Resolve the `claude` binary path.
///
/// macOS GUI apps (DMG installs) don't inherit the user's shell PATH, so a bare
/// `"claude"` lookup fails even when the CLI is installed. Check well-known
/// locations first, then fall back to a bare name for PATH resolution.
pub(crate) fn resolve_claude_binary() -> String {
    let candidates: Vec<PathBuf> = [
        dirs::home_dir().map(|h| h.join(".local/bin/claude")),
        Some(PathBuf::from("/usr/local/bin/claude")),
        Some(PathBuf::from("/opt/homebrew/bin/claude")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if path.exists() {
            info!("Resolved claude CLI at {}", path.display());
            return path.to_string_lossy().into_owned();
        }
    }

    // Fall back to bare name — works when PATH is inherited (e.g. `pnpm tauri dev`)
    "claude".to_string()
}
