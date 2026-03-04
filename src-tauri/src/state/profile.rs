use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A profile is a named filter over servers, skills, plugins, and features.
/// When active, only the profile's referenced items appear in AI tool configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub features: ProfileFeatures,
    /// AI tool IDs this profile writes to (e.g. "claude-code", "cursor").
    pub integration_ids: Vec<String>,
    /// ServerConfig IDs included in this profile.
    pub server_ids: Vec<String>,
    /// InstalledSkill IDs included in this profile.
    pub skill_ids: Vec<String>,
    /// InstalledPlugin IDs included in this profile.
    pub plugin_ids: Vec<String>,
    /// Directories that auto-activate this profile (per-project configs).
    pub directory_paths: Vec<String>,
    /// Display order in the switcher.
    pub sort_order: u32,
}

/// Feature toggles for a profile. Extensible without changing Profile struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileFeatures {
    /// Whether memory is enabled for this profile.
    pub memory: bool,
    /// Redis db number (0-15) for this profile's memory isolation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_db: Option<u8>,
    /// Whether discovery mode is used for this profile.
    pub discovery: bool,
}

impl Default for ProfileFeatures {
    fn default() -> Self {
        Self {
            memory: true,
            memory_db: None,
            discovery: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Export / Import types
// ---------------------------------------------------------------------------

/// A server config with secrets stripped, for sharing profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileExportServer {
    pub name: String,
    pub transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// A skill reference for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileExportSkill {
    pub name: String,
    pub skill_id: String,
    pub source: String,
}

/// Full-featured shareable profile format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileExport {
    /// Format version.
    pub agent_hub_profile: String,
    pub name: String,
    pub features: ProfileFeatures,
    pub integrations: Vec<String>,
    pub servers: Vec<ProfileExportServer>,
    pub skills: Vec<ProfileExportSkill>,
    pub plugins: Vec<String>,
    pub directory_paths: Vec<String>,
}
