use tauri::{AppHandle, Manager, State};
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::AppError;
use crate::mcp::proxy::ProxyState;
use crate::persistence;
use crate::state::{
    Profile, ProfileExport, ProfileExportServer, ProfileExportSkill, ProfileFeatures,
    ServerTransport, SharedState,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Re-sync integration configs and skill files after a profile change.
fn resync_after_profile_change(app: &AppHandle, port: u16) {
    if let Err(e) = super::integrations::update_all_integration_configs(app, port) {
        warn!("Failed to update integration configs after profile change: {e}");
    }

    // Re-sync skills for each enabled skill integration
    let (installed_skills, enabled_skill_integrations) = {
        let state = app.state::<SharedState>();
        let s = state.lock().unwrap();
        (s.installed_skills.clone(), s.enabled_skill_integrations.clone())
    };

    for tool_id in &enabled_skill_integrations {
        let filtered = filter_skills_for_active_profile(&installed_skills);
        if let Err(e) = super::skills_config::sync_skills_for_tool(tool_id, &filtered) {
            warn!("Failed to sync skills for {tool_id} after profile change: {e}");
        }
    }
}

/// Return all installed skills (no profile filtering — profiles no longer gate skills globally).
pub fn filter_skills_for_active_profile(
    installed_skills: &[crate::state::InstalledSkill],
) -> Vec<crate::state::InstalledSkill> {
    installed_skills.to_vec()
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_profiles(state: State<'_, SharedState>) -> Result<Vec<Profile>, AppError> {
    let s = state.lock().unwrap();
    Ok(s.profiles.clone())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileInput {
    pub name: String,
    #[serde(default)]
    pub integration_ids: Vec<String>,
    #[serde(default)]
    pub server_ids: Vec<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub plugin_ids: Vec<String>,
    #[serde(default)]
    pub features: Option<ProfileFeatures>,
}

#[tauri::command]
pub async fn create_profile(
    app: AppHandle,
    state: State<'_, SharedState>,
    input: CreateProfileInput,
) -> Result<Profile, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Profile name cannot be empty".into()));
    }

    let profile = Profile {
        id: Uuid::new_v4().to_string(),
        name: input.name,
        features: input.features.unwrap_or_default(),
        integration_ids: input.integration_ids,
        server_ids: input.server_ids,
        skill_ids: input.skill_ids,
        plugin_ids: input.plugin_ids,
        directory_paths: Vec::new(),
        sort_order: {
            let s = state.lock().unwrap();
            s.profiles.len() as u32
        },
    };

    {
        let mut s = state.lock().unwrap();
        s.profiles.push(profile.clone());
        persistence::save_profiles(&app, &s.profiles);
    }

    info!("Created profile '{}' ({})", profile.name, profile.id);
    Ok(profile)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub features: Option<ProfileFeatures>,
    #[serde(default)]
    pub integration_ids: Option<Vec<String>>,
    #[serde(default)]
    pub server_ids: Option<Vec<String>>,
    #[serde(default)]
    pub skill_ids: Option<Vec<String>>,
    #[serde(default)]
    pub plugin_ids: Option<Vec<String>>,
    #[serde(default)]
    pub directory_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sort_order: Option<u32>,
}

#[tauri::command]
pub async fn update_profile(
    app: AppHandle,
    state: State<'_, SharedState>,
    proxy_state: State<'_, ProxyState>,
    id: String,
    input: UpdateProfileInput,
) -> Result<Profile, AppError> {
    let updated = {
        let mut s = state.lock().unwrap();

        let idx = s
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| AppError::Validation(format!("Profile not found: {id}")))?;

        let profile = &mut s.profiles[idx];

        if let Some(name) = input.name {
            if name.trim().is_empty() {
                return Err(AppError::Validation("Profile name cannot be empty".into()));
            }
            profile.name = name;
        }
        if let Some(features) = input.features {
            profile.features = features;
        }
        if let Some(ids) = input.integration_ids {
            profile.integration_ids = ids;
        }
        if let Some(ids) = input.server_ids {
            profile.server_ids = ids;
        }
        if let Some(ids) = input.skill_ids {
            profile.skill_ids = ids;
        }
        if let Some(ids) = input.plugin_ids {
            profile.plugin_ids = ids;
        }
        if let Some(paths) = input.directory_paths {
            profile.directory_paths = paths;
        }
        if let Some(order) = input.sort_order {
            profile.sort_order = order;
        }

        let updated = profile.clone();
        persistence::save_profiles(&app, &s.profiles);
        updated
    };

    // Always re-sync configs when a profile changes
    let port = proxy_state.port().await;
    resync_after_profile_change(&app, port);

    // Sync per-DB memory containers in case memory_db changed
    if let Err(e) = super::memory::sync_memory_containers(&app).await {
        warn!("Failed to sync memory containers after profile update: {e}");
    }

    crate::tray::rebuild_tray_menu(&app);
    info!("Updated profile '{}' ({})", updated.name, updated.id);
    Ok(updated)
}

#[tauri::command]
pub async fn delete_profile(
    app: AppHandle,
    state: State<'_, SharedState>,
    proxy_state: State<'_, ProxyState>,
    id: String,
) -> Result<(), AppError> {
    let directory_paths;
    {
        let mut s = state.lock().unwrap();
        // Grab the profile's directories before removing it so we can clean up
        directory_paths = s
            .profiles
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.directory_paths.clone())
            .unwrap_or_default();

        let before = s.profiles.len();
        s.profiles.retain(|p| p.id != id);
        if s.profiles.len() == before {
            return Err(AppError::Validation(format!("Profile not found: {id}")));
        }
        persistence::save_profiles(&app, &s.profiles);
    }

    // Clean up per-project configs for the deleted profile's directories
    for dir in &directory_paths {
        super::integrations::clean_project_configs(dir);
    }

    // Always resync after deletion
    let port = proxy_state.port().await;
    resync_after_profile_change(&app, port);

    // Sync per-DB memory containers in case deleted profile had a unique DB
    if let Err(e) = super::memory::sync_memory_containers(&app).await {
        warn!("Failed to sync memory containers after profile delete: {e}");
    }

    crate::tray::rebuild_tray_menu(&app);
    info!("Deleted profile {id}");
    Ok(())
}

#[tauri::command]
pub async fn add_directory_to_profile(
    app: AppHandle,
    state: State<'_, SharedState>,
    proxy_state: State<'_, ProxyState>,
    profile_id: String,
    directory: String,
) -> Result<Profile, AppError> {
    let updated = {
        let mut s = state.lock().unwrap();

        let idx = s
            .profiles
            .iter()
            .position(|p| p.id == profile_id)
            .ok_or_else(|| AppError::Validation(format!("Profile not found: {profile_id}")))?;

        let profile = &mut s.profiles[idx];

        if !profile.directory_paths.contains(&directory) {
            profile.directory_paths.push(directory.clone());
        }

        let updated = profile.clone();
        persistence::save_profiles(&app, &s.profiles);
        updated
    };

    // Always write per-project configs for the new directory
    let port = proxy_state.port().await;
    resync_after_profile_change(&app, port);

    info!(
        "Added directory '{}' to profile '{}'",
        directory, updated.name
    );
    Ok(updated)
}

#[tauri::command]
pub async fn remove_directory_from_profile(
    app: AppHandle,
    state: State<'_, SharedState>,
    proxy_state: State<'_, ProxyState>,
    profile_id: String,
    directory: String,
) -> Result<Profile, AppError> {
    let updated = {
        let mut s = state.lock().unwrap();

        let idx = s
            .profiles
            .iter()
            .position(|p| p.id == profile_id)
            .ok_or_else(|| AppError::Validation(format!("Profile not found: {profile_id}")))?;

        let profile = &mut s.profiles[idx];
        profile.directory_paths.retain(|d| d != &directory);

        let updated = profile.clone();
        persistence::save_profiles(&app, &s.profiles);
        updated
    };

    // Clean up per-project config files in the removed directory
    super::integrations::clean_project_configs(&directory);

    // Always resync after directory removal
    let port = proxy_state.port().await;
    resync_after_profile_change(&app, port);

    info!(
        "Removed directory '{}' from profile '{}'",
        directory, updated.name
    );
    Ok(updated)
}

#[tauri::command]
pub async fn export_profile(
    state: State<'_, SharedState>,
    profile_id: String,
) -> Result<ProfileExport, AppError> {
    let s = state.lock().unwrap();
    let profile = s
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| AppError::Validation(format!("Profile not found: {profile_id}")))?;

    // Build server exports with secrets stripped
    let servers: Vec<ProfileExportServer> = profile
        .server_ids
        .iter()
        .filter_map(|sid| s.servers.iter().find(|srv| srv.id == *sid))
        .map(|srv| {
            let transport = match srv.transport {
                ServerTransport::Stdio => "stdio".to_string(),
                ServerTransport::Http => "http".to_string(),
            };
            ProfileExportServer {
                name: srv.name.clone(),
                transport,
                command: srv.command.clone(),
                args: srv.args.clone(),
                env: srv.env.as_ref().map(|env| {
                    env.iter()
                        .map(|(k, _)| (k.clone(), String::new()))
                        .collect()
                }),
                url: srv.url.clone(),
                headers: srv.headers.as_ref().map(|h| {
                    h.iter()
                        .map(|(k, _)| (k.clone(), String::new()))
                        .collect()
                }),
            }
        })
        .collect();

    // Build skill exports
    let skills: Vec<ProfileExportSkill> = profile
        .skill_ids
        .iter()
        .filter_map(|sid| s.installed_skills.iter().find(|sk| sk.id == *sid))
        .map(|sk| ProfileExportSkill {
            name: sk.name.clone(),
            skill_id: sk.skill_id.clone(),
            source: sk.source.clone(),
        })
        .collect();

    // Build plugin name list
    let plugins: Vec<String> = profile.plugin_ids.clone();

    Ok(ProfileExport {
        agent_hub_profile: "1.0".to_string(),
        name: profile.name.clone(),
        features: profile.features.clone(),
        integrations: profile.integration_ids.clone(),
        servers,
        skills,
        plugins,
        directory_paths: profile.directory_paths.clone(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub profile: Profile,
    pub matched_servers: usize,
    pub created_servers: usize,
    pub matched_skills: usize,
    pub unmatched_skills: Vec<String>,
    pub matched_plugins: usize,
    pub unmatched_plugins: Vec<String>,
}

#[tauri::command]
pub async fn import_profile(
    app: AppHandle,
    state: State<'_, SharedState>,
    data: ProfileExport,
) -> Result<ImportResult, AppError> {
    if data.name.trim().is_empty() {
        return Err(AppError::Validation(
            "Imported profile name cannot be empty".into(),
        ));
    }

    let mut matched_servers = 0usize;
    let mut created_servers = 0usize;
    let mut server_ids = Vec::new();
    let mut matched_skills = 0usize;
    let mut unmatched_skills = Vec::new();
    let mut skill_ids = Vec::new();
    let mut matched_plugins = 0usize;
    let unmatched_plugins = Vec::new();
    let mut plugin_ids = Vec::new();

    {
        let mut s = state.lock().unwrap();

        // Match servers by name
        for export_server in &data.servers {
            if let Some(existing) = s.servers.iter().find(|srv| srv.name == export_server.name) {
                server_ids.push(existing.id.clone());
                matched_servers += 1;
            } else {
                // Create a new server from the export data
                let transport = if export_server.transport == "http" {
                    ServerTransport::Http
                } else {
                    ServerTransport::Stdio
                };
                let new_server = crate::state::ServerConfig {
                    id: Uuid::new_v4().to_string(),
                    name: export_server.name.clone(),
                    enabled: true,
                    transport,
                    command: export_server.command.clone(),
                    args: export_server.args.clone(),
                    env: export_server.env.clone(),
                    url: export_server.url.clone(),
                    headers: export_server.headers.clone(),
                    tags: None,
                    status: Some(crate::state::ServerStatus::Disconnected),
                    last_connected: None,
                    managed: None,
                    managed_by: None,
                    registry_name: None,
                };
                server_ids.push(new_server.id.clone());
                s.servers.push(new_server);
                created_servers += 1;
            }
        }

        // Match skills by id
        for export_skill in &data.skills {
            if let Some(existing) = s
                .installed_skills
                .iter()
                .find(|sk| sk.skill_id == export_skill.skill_id)
            {
                skill_ids.push(existing.id.clone());
                matched_skills += 1;
            } else {
                unmatched_skills.push(export_skill.name.clone());
            }
        }

        // Match plugins by id
        for plugin_name in &data.plugins {
            // Plugins use ID directly
            plugin_ids.push(plugin_name.clone());
            // We can't easily verify installed plugins without another state lookup,
            // so just track all as matched for now
            matched_plugins += 1;
        }

        if created_servers > 0 {
            persistence::save_servers(&app, &s.servers);
        }
    }

    // Create the profile
    let profile = Profile {
        id: Uuid::new_v4().to_string(),
        name: data.name,
        features: data.features,
        integration_ids: data.integrations,
        server_ids,
        skill_ids,
        plugin_ids,
        directory_paths: data.directory_paths,
        sort_order: {
            let s = state.lock().unwrap();
            s.profiles.len() as u32
        },
    };

    {
        let mut s = state.lock().unwrap();
        s.profiles.push(profile.clone());
        persistence::save_profiles(&app, &s.profiles);
    }

    if created_servers > 0 {
        crate::tray::rebuild_tray_menu(&app);
    }

    info!(
        "Imported profile '{}': {} servers matched, {} created, {} skills matched, {} unmatched",
        profile.name,
        matched_servers,
        created_servers,
        matched_skills,
        unmatched_skills.len()
    );

    Ok(ImportResult {
        profile,
        matched_servers,
        created_servers,
        matched_skills,
        unmatched_skills,
        matched_plugins,
        unmatched_plugins,
    })
}

#[tauri::command]
pub async fn export_profile_to_file(
    state: State<'_, SharedState>,
    profile_id: String,
    path: String,
) -> Result<(), AppError> {
    let data = export_profile(state, profile_id).await?;
    let json = serde_json::to_string_pretty(&data)?;
    std::fs::write(&path, json)?;
    info!("Exported profile '{}' to {path}", data.name);
    Ok(())
}

#[tauri::command]
pub async fn import_profile_from_file(
    app: AppHandle,
    state: State<'_, SharedState>,
    path: String,
) -> Result<ImportResult, AppError> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Io(e))?;
    let data: ProfileExport = serde_json::from_str(&content)
        .map_err(|e| AppError::Validation(format!("Invalid profile file: {e}")))?;
    // Delegate to the existing import logic
    import_profile(app, state, data).await
}
