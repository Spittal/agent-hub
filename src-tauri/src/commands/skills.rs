use std::collections::HashSet;
use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, State};
use tracing::{info, warn};

use crate::commands::skills_config;
use crate::error::AppError;
use crate::persistence;
use crate::state::skill::InstalledSkill;
use crate::state::skills_registry::{
    MarketplaceSkillDetail, SkillsMarketplaceCache, SkillsSearchResult,
};
use crate::state::SharedState;

// ---------------------------------------------------------------------------
// YAML frontmatter parser (reused from old implementation)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, Default)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
}

fn parse_frontmatter(content: &str) -> (SkillFrontmatter, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (SkillFrontmatter::default(), content.to_string());
    }

    let after_first = &trimmed[3..];
    if let Some(end_idx) = after_first.find("\n---") {
        let yaml_str = &after_first[..end_idx];
        let body_start = end_idx + 4;
        let body = after_first[body_start..]
            .trim_start_matches('\n')
            .to_string();

        match serde_yaml::from_str::<SkillFrontmatter>(yaml_str) {
            Ok(fm) => (fm, body),
            Err(e) => {
                warn!("Failed to parse SKILL.md frontmatter: {e}");
                (SkillFrontmatter::default(), content.to_string())
            }
        }
    } else {
        (SkillFrontmatter::default(), content.to_string())
    }
}

// ---------------------------------------------------------------------------
// Shared directory scanner
// ---------------------------------------------------------------------------

/// Info about a skill found on disk in a tool's skills directory.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingSkillInfo {
    pub skill_id: String,
    pub name: String,
    pub description: String,
}

/// Scan a skills directory for subdirectories containing SKILL.md (and standalone .md files).
/// Returns info for each skill found, excluding any whose skill_id is in `exclude_ids`.
fn scan_skills_in_dir(dir: &Path, exclude_ids: &HashSet<String>) -> Vec<ExistingSkillInfo> {
    if !dir.exists() {
        return vec![];
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut results = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        // Case 1: subdirectory with SKILL.md
        if path.is_dir() {
            let skill_id = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            if exclude_ids.contains(&skill_id) {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            if let Some(info) = read_skill_info(&skill_md, &skill_id) {
                results.push(info);
            }
        }
        // Case 2: standalone .md file
        else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let skill_id = match path.file_stem().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            if exclude_ids.contains(&skill_id) {
                continue;
            }
            if let Some(info) = read_skill_info(&path, &skill_id) {
                results.push(info);
            }
        }
    }

    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    results
}

/// Read a SKILL.md file and extract name/description from frontmatter.
fn read_skill_info(file_path: &Path, skill_id: &str) -> Option<ExistingSkillInfo> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| warn!("Failed to read {}: {e}", file_path.display()))
        .ok()?;
    let (fm, _body) = parse_frontmatter(&content);
    Some(ExistingSkillInfo {
        skill_id: skill_id.to_string(),
        name: fm.name.unwrap_or_else(|| skill_id.to_string()),
        description: fm.description.unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------------
// Marketplace commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn search_skills_marketplace(
    cache: State<'_, SkillsMarketplaceCache>,
    state: State<'_, SharedState>,
    search: String,
    limit: Option<u32>,
) -> Result<SkillsSearchResult, AppError> {
    let (installed_ids, installed_skill_ids): (Vec<String>, Vec<String>) = {
        let s = state.lock().unwrap();
        let ids = s.installed_skills.iter().map(|sk| sk.id.clone()).collect();
        let skill_ids = s.installed_skills.iter().map(|sk| sk.skill_id.clone()).collect();
        (ids, skill_ids)
    };

    let result = cache
        .search(&search, limit.unwrap_or(30), &installed_ids, &installed_skill_ids)
        .await;
    Ok(result)
}

#[tauri::command]
pub async fn get_skills_marketplace_detail(
    cache: State<'_, SkillsMarketplaceCache>,
    source: String,
    skill_id: String,
    name: String,
    installs: u64,
) -> Result<MarketplaceSkillDetail, AppError> {
    let content = cache
        .fetch_skill_content(&source, &skill_id)
        .await
        .ok_or_else(|| {
            AppError::Protocol(format!(
                "Could not fetch SKILL.md for {source}/{skill_id}"
            ))
        })?;

    let (fm, _body) = parse_frontmatter(&content);

    Ok(MarketplaceSkillDetail {
        id: format!("{source}/{skill_id}"),
        name: fm.name.unwrap_or(name),
        source: source.clone(),
        skill_id,
        installs,
        description: fm.description.unwrap_or_default(),
        content,
    })
}

// ---------------------------------------------------------------------------
// Shared helpers for managed skills (used by memory.rs, discovery.rs, etc.)
// ---------------------------------------------------------------------------

/// Install a managed skill into state, persist it, and write to enabled tool directories.
/// Returns the created `InstalledSkill`. Skips if a skill with the same `skill_id` already exists.
pub fn install_managed_skill(
    app: &AppHandle,
    state: &SharedState,
    skill_id: &str,
    name: &str,
    description: &str,
    content: &str,
    managed_by: &str,
) {
    let integrations = {
        let mut s = state.lock().unwrap();
        if s.installed_skills.iter().any(|sk| sk.skill_id == skill_id) {
            return;
        }
        let skill = InstalledSkill {
            id: format!("mcp-manager/{skill_id}"),
            name: name.to_string(),
            skill_id: skill_id.to_string(),
            source: "mcp-manager".to_string(),
            description: description.to_string(),
            content: content.to_string(),
            enabled: true,
            installs: None,
            managed: None,
            managed_by: Some(managed_by.to_string()),
        };
        s.installed_skills.push(skill);
        persistence::save_installed_skills(app, &s.installed_skills);
        s.enabled_skill_integrations.clone()
    };

    if let Err(e) = skills_config::write_skill(skill_id, content, &integrations) {
        warn!("Failed to write managed skill {skill_id}: {e}");
    }
    info!("Installed managed skill: {skill_id} (managed_by={managed_by})");
}

/// Uninstall a managed skill from state, persist, and remove from tool directories.
pub fn uninstall_managed_skill(
    app: &AppHandle,
    state: &SharedState,
    skill_id: &str,
    managed_by: &str,
) {
    let integrations = {
        let mut s = state.lock().unwrap();
        let idx = match s.installed_skills.iter().position(|sk| {
            sk.skill_id == skill_id && sk.managed_by.as_deref() == Some(managed_by)
        }) {
            Some(i) => i,
            None => return,
        };
        s.installed_skills.remove(idx);
        persistence::save_installed_skills(app, &s.installed_skills);
        s.enabled_skill_integrations.clone()
    };

    if let Err(e) = skills_config::remove_skill(skill_id, &integrations) {
        warn!("Failed to remove managed skill {skill_id}: {e}");
    }
    info!("Uninstalled managed skill: {skill_id} (managed_by={managed_by})");
}

// ---------------------------------------------------------------------------
// Startup reconciliation — ensure managed skills exist for enabled features
// ---------------------------------------------------------------------------

/// Reconcile managed skills on startup: if a feature (memory, discovery) is
/// enabled in state but the corresponding managed skill entry is missing,
/// install it. This handles users who enabled features before managed skills
/// were introduced.
pub fn reconcile_managed_skills(app: &AppHandle, state: &SharedState) {
    use crate::commands::discovery::{DISCOVERY_SKILL_CONTENT, DISCOVERY_SKILL_ID};
    use crate::commands::memory::{MEMORY_MANAGED_SKILL_CONTENT, MEMORY_SKILL_ID};

    let (memory_enabled, discovery_enabled) = {
        let s = state.lock().unwrap();
        let mem = s
            .servers
            .iter()
            .any(|srv| srv.managed_by.as_deref() == Some("memory"));
        let disc = s.tool_discovery_enabled;
        (mem, disc)
    };

    if memory_enabled {
        // Ensure the Claude Code SKILL.md file exists on disk
        crate::commands::memory::install_memory_skill();

        // Ensure the managed skill entry exists in state
        install_managed_skill(
            app,
            state,
            MEMORY_SKILL_ID,
            "using-memory-mcp",
            "Search and store persistent memories using the agent-memory MCP server",
            MEMORY_MANAGED_SKILL_CONTENT,
            "memory",
        );
        info!("Reconciled memory managed skill");
    }

    if discovery_enabled {
        install_managed_skill(
            app,
            state,
            DISCOVERY_SKILL_ID,
            "using-discovery",
            "Find and use MCP tools through the discovery endpoint",
            DISCOVERY_SKILL_CONTENT,
            "discovery",
        );
        info!("Reconciled discovery managed skill");
    }
}

// ---------------------------------------------------------------------------
// Management commands
// ---------------------------------------------------------------------------

/// Frontend-facing installed skill (without full content to keep payloads small).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkillInfo {
    pub id: String,
    pub name: String,
    pub skill_id: String,
    pub source: String,
    pub description: String,
    pub enabled: bool,
    pub installs: Option<u64>,
    pub managed: bool,
    pub managed_by: Option<String>,
}

impl From<&InstalledSkill> for InstalledSkillInfo {
    fn from(s: &InstalledSkill) -> Self {
        let is_managed = s.managed_by.is_some() || s.managed == Some(true);
        Self {
            id: s.id.clone(),
            name: s.name.clone(),
            skill_id: s.skill_id.clone(),
            source: s.source.clone(),
            description: s.description.clone(),
            enabled: s.enabled,
            installs: s.installs,
            managed: is_managed,
            managed_by: s.managed_by.clone(),
        }
    }
}

#[tauri::command]
pub async fn list_installed_skills(
    state: State<'_, SharedState>,
) -> Result<Vec<InstalledSkillInfo>, AppError> {
    let s = state.lock().unwrap();
    Ok(s.installed_skills.iter().map(InstalledSkillInfo::from).collect())
}

#[tauri::command]
pub async fn install_skill(
    app: AppHandle,
    state: State<'_, SharedState>,
    cache: State<'_, SkillsMarketplaceCache>,
    id: String,
    name: String,
    source: String,
    skill_id: String,
    installs: Option<u64>,
) -> Result<InstalledSkillInfo, AppError> {
    // Check if already installed
    {
        let s = state.lock().unwrap();
        if s.installed_skills.iter().any(|sk| sk.id == id) {
            return Err(AppError::Validation(format!("Skill already installed: {id}")));
        }
    }

    // Fetch SKILL.md content
    let content = cache
        .fetch_skill_content(&source, &skill_id)
        .await
        .ok_or_else(|| {
            AppError::Protocol(format!(
                "Could not fetch SKILL.md for {source}/{skill_id}"
            ))
        })?;

    let (fm, _body) = parse_frontmatter(&content);

    let skill = InstalledSkill {
        id: id.clone(),
        name: fm.name.unwrap_or(name),
        skill_id: skill_id.clone(),
        source,
        description: fm.description.unwrap_or_default(),
        content: content.clone(),
        enabled: true,
        installs,
        managed: None,
        managed_by: None,
    };

    let enabled_integrations: Vec<String>;
    {
        let mut s = state.lock().unwrap();
        s.installed_skills.push(skill.clone());
        enabled_integrations = s.enabled_skill_integrations.clone();
        persistence::save_installed_skills(&app, &s.installed_skills);
    }

    // Write SKILL.md to all enabled tool directories
    if let Err(e) = skills_config::write_skill(&skill_id, &content, &enabled_integrations) {
        warn!("Failed to write skill files: {e}");
    }

    info!("Installed skill: {id}");
    Ok(InstalledSkillInfo::from(&skill))
}

#[tauri::command]
pub async fn uninstall_skill(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: String,
) -> Result<(), AppError> {
    // Check if managed — managed skills cannot be uninstalled directly
    {
        let s = state.lock().unwrap();
        let skill = s.installed_skills.iter().find(|sk| sk.id == id)
            .ok_or_else(|| AppError::Validation(format!("Skill not found: {id}")))?;
        if skill.managed_by.is_some() || skill.managed == Some(true) {
            return Err(AppError::Validation("Cannot uninstall a managed skill. Disable the parent feature instead.".into()));
        }
    }

    let (skill_id, enabled_integrations) = {
        let mut s = state.lock().unwrap();
        let idx = s
            .installed_skills
            .iter()
            .position(|sk| sk.id == id)
            .ok_or_else(|| AppError::Validation(format!("Skill not found: {id}")))?;

        let skill = s.installed_skills.remove(idx);
        let integrations = s.enabled_skill_integrations.clone();
        persistence::save_installed_skills(&app, &s.installed_skills);
        (skill.skill_id, integrations)
    };

    // Remove SKILL.md from all enabled tool directories
    if let Err(e) = skills_config::remove_skill(&skill_id, &enabled_integrations) {
        warn!("Failed to remove skill files: {e}");
    }

    info!("Uninstalled skill: {id}");
    Ok(())
}

#[tauri::command]
pub async fn toggle_skill(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: String,
    enabled: bool,
) -> Result<InstalledSkillInfo, AppError> {
    let (skill_id, content, enabled_integrations) = {
        let mut s = state.lock().unwrap();
        let skill = s
            .installed_skills
            .iter_mut()
            .find(|sk| sk.id == id)
            .ok_or_else(|| AppError::Validation(format!("Skill not found: {id}")))?;

        skill.enabled = enabled;
        let skill_id = skill.skill_id.clone();
        let content = skill.content.clone();
        let integrations = s.enabled_skill_integrations.clone();
        persistence::save_installed_skills(&app, &s.installed_skills);
        (skill_id, content, integrations)
    };

    if enabled {
        if let Err(e) = skills_config::write_skill(&skill_id, &content, &enabled_integrations) {
            warn!("Failed to write skill files on enable: {e}");
        }
    } else {
        if let Err(e) = skills_config::remove_skill(&skill_id, &enabled_integrations) {
            warn!("Failed to remove skill files on disable: {e}");
        }
    }

    let s = state.lock().unwrap();
    let skill = s.installed_skills.iter().find(|sk| sk.id == id).unwrap();
    Ok(InstalledSkillInfo::from(skill))
}

#[tauri::command]
pub async fn get_skill_content(
    state: State<'_, SharedState>,
    id: String,
) -> Result<SkillContentResponse, AppError> {
    let s = state.lock().unwrap();
    let skill = s
        .installed_skills
        .iter()
        .find(|sk| sk.id == id)
        .ok_or_else(|| AppError::Validation(format!("Skill not found: {id}")))?;

    let (_fm, body) = parse_frontmatter(&skill.content);

    Ok(SkillContentResponse {
        id: skill.id.clone(),
        name: skill.name.clone(),
        content: body,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContentResponse {
    pub id: String,
    pub name: String,
    pub content: String,
}

// ---------------------------------------------------------------------------
// Skill integration commands (Settings > Skills)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillToolInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub enabled: bool,
    pub skills_path: String,
    pub existing_skills: Vec<ExistingSkillInfo>,
}

/// Detect which tools support skills, whether they're installed, and whether
/// skill management is enabled for each.
#[tauri::command]
pub async fn detect_skill_integrations(
    state: State<'_, SharedState>,
) -> Result<Vec<SkillToolInfo>, AppError> {
    let tools = skills_config::get_skill_tool_definitions()?;
    let (enabled_ids, installed_skill_ids) = {
        let s = state.lock().unwrap();
        let enabled = s.enabled_skill_integrations.clone();
        let ids: HashSet<String> = s.installed_skills.iter().map(|sk| sk.skill_id.clone()).collect();
        (enabled, ids)
    };

    let results = tools
        .into_iter()
        .map(|tool| {
            // A tool is "installed" if its parent directory exists
            let parent = tool.skills_dir.parent();
            let installed = parent.map(|p| p.exists()).unwrap_or(false);

            let existing_skills = scan_skills_in_dir(&tool.skills_dir, &installed_skill_ids);

            SkillToolInfo {
                id: tool.id.to_string(),
                name: tool.name.to_string(),
                installed,
                enabled: enabled_ids.contains(&tool.id.to_string()),
                skills_path: tool.skills_dir.display().to_string(),
                existing_skills,
            }
        })
        .collect();

    Ok(results)
}

/// Read full content of a SKILL.md file (for import). Returns `(skill_id, name, description, content)`.
fn read_skill_for_import(file_path: &Path, skill_id: &str) -> Option<(String, String, String, String)> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| warn!("Failed to read {}: {e}", file_path.display()))
        .ok()?;
    let (fm, _body) = parse_frontmatter(&content);
    let name = fm.name.unwrap_or_else(|| skill_id.to_string());
    let description = fm.description.unwrap_or_default();
    Some((skill_id.to_string(), name, description, content))
}

/// Scan a tool's skills directory and return importable skills (full content) not already installed.
fn find_importable_skills(skills_dir: &Path, installed_ids: &HashSet<String>) -> Vec<(String, String, String, String)> {
    if !skills_dir.exists() {
        return vec![];
    }
    let entries = match std::fs::read_dir(skills_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_id = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            if installed_ids.contains(&skill_id) {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                if let Some(info) = read_skill_for_import(&skill_md, &skill_id) {
                    results.push(info);
                }
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let skill_id = match path.file_stem().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            if installed_ids.contains(&skill_id) {
                continue;
            }
            if let Some(info) = read_skill_for_import(&path, &skill_id) {
                results.push(info);
            }
        }
    }
    results
}

/// Enable skill file management for a tool — imports existing skills from disk, then writes all enabled skills.
#[tauri::command]
pub async fn enable_skill_integration(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: String,
) -> Result<SkillToolInfo, AppError> {
    if !skills_config::supports_skills(&id) {
        return Err(AppError::Validation(format!(
            "Tool {id} does not support skills"
        )));
    }

    let tools = skills_config::get_skill_tool_definitions()?;
    let tool = tools.iter().find(|t| t.id == id).ok_or_else(|| {
        AppError::Validation(format!("Unknown skill tool: {id}"))
    })?;

    // Import existing skills from this tool's directory before enabling
    let importable = {
        let s = state.lock().unwrap();
        let installed_ids: HashSet<String> = s.installed_skills.iter().map(|sk| sk.skill_id.clone()).collect();
        find_importable_skills(&tool.skills_dir, &installed_ids)
    };

    if !importable.is_empty() {
        let mut s = state.lock().unwrap();
        for (skill_id, name, description, content) in &importable {
            // Double-check not already added (another tool may share the same skill_id)
            if s.installed_skills.iter().any(|sk| sk.skill_id == *skill_id) {
                continue;
            }
            let skill = InstalledSkill {
                id: format!("local:{id}/{skill_id}"),
                name: name.clone(),
                skill_id: skill_id.clone(),
                source: "local".to_string(),
                description: description.clone(),
                content: content.clone(),
                enabled: true,
                installs: None,
                managed: None,
                managed_by: None,
            };
            info!("Imported existing skill from {}: {skill_id}", tool.name);
            s.installed_skills.push(skill);
        }
        persistence::save_installed_skills(&app, &s.installed_skills);
    }

    let installed_skills = {
        let mut s = state.lock().unwrap();
        if !s.enabled_skill_integrations.contains(&id) {
            s.enabled_skill_integrations.push(id.clone());
            persistence::save_enabled_skill_integrations(&app, &s.enabled_skill_integrations);
        }
        s.installed_skills.clone()
    };

    // Sync all enabled skills to this tool
    if let Err(e) = skills_config::sync_skills_for_tool(&id, &installed_skills) {
        warn!("Failed to sync skills for {id}: {e}");
    }

    let tool = tools.iter().find(|t| t.id == id).unwrap();
    let parent = tool.skills_dir.parent();
    let installed = parent.map(|p| p.exists()).unwrap_or(false);

    // Re-read installed skill IDs (may have changed from import above)
    let installed_skill_ids: HashSet<String> = {
        let s = state.lock().unwrap();
        s.installed_skills.iter().map(|sk| sk.skill_id.clone()).collect()
    };

    info!("Enabled skill integration for {}", tool.name);

    Ok(SkillToolInfo {
        id: tool.id.to_string(),
        name: tool.name.to_string(),
        installed,
        enabled: true,
        skills_path: tool.skills_dir.display().to_string(),
        existing_skills: scan_skills_in_dir(&tool.skills_dir, &installed_skill_ids),
    })
}

/// Disable skill file management for a tool — removes all managed skill files.
#[tauri::command]
pub async fn disable_skill_integration(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: String,
) -> Result<SkillToolInfo, AppError> {
    let (installed_skills, tools) = {
        let mut s = state.lock().unwrap();
        s.enabled_skill_integrations.retain(|i| i != &id);
        persistence::save_enabled_skill_integrations(&app, &s.enabled_skill_integrations);
        (s.installed_skills.clone(), skills_config::get_skill_tool_definitions()?)
    };

    // Remove all managed skill files from this tool
    if let Err(e) = skills_config::remove_all_skills_for_tool(&id, &installed_skills) {
        warn!("Failed to remove skills for {id}: {e}");
    }

    let tool = tools.iter().find(|t| t.id == id).ok_or_else(|| {
        AppError::Validation(format!("Unknown skill tool: {id}"))
    })?;
    let parent = tool.skills_dir.parent();
    let installed = parent.map(|p| p.exists()).unwrap_or(false);

    let installed_skill_ids: HashSet<String> = {
        let s = state.lock().unwrap();
        s.installed_skills.iter().map(|sk| sk.skill_id.clone()).collect()
    };

    info!("Disabled skill integration for {}", tool.name);

    Ok(SkillToolInfo {
        id: tool.id.to_string(),
        name: tool.name.to_string(),
        installed,
        enabled: false,
        skills_path: tool.skills_dir.display().to_string(),
        existing_skills: scan_skills_in_dir(&tool.skills_dir, &installed_skill_ids),
    })
}
