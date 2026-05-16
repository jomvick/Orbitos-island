use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub binary: Option<String>,
    pub installed: bool,
    pub hooks_supported: bool,
    pub hooks_installed: bool,
    pub config_path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverResult {
    pub agents: Vec<AgentInfo>,
    pub daemon_socket: Option<String>,
    pub total_agents: usize,
    pub installed_count: usize,
    pub hooks_installed_count: usize,
}

static AGENTS: &[AgentConfig] = &[
    AgentConfig {
        name: "claude",
        binaries: &["claude"],
        hooks_supported: true,
        config_install: Some(install_claude_hooks),
    },
    AgentConfig {
        name: "opencode",
        binaries: &["opencode"],
        hooks_supported: true,
        config_install: Some(install_opencode_hooks),
    },
    AgentConfig {
        name: "codex",
        binaries: &["codex"],
        hooks_supported: false,
        config_install: None,
    },
    AgentConfig {
        name: "aider",
        binaries: &["aider"],
        hooks_supported: false,
        config_install: None,
    },
    AgentConfig {
        name: "gemini",
        binaries: &["gemini"],
        hooks_supported: false,
        config_install: None,
    },
    AgentConfig {
        name: "cursor",
        binaries: &["cursor"],
        hooks_supported: false,
        config_install: None,
    },
    AgentConfig {
        name: "copilot",
        binaries: &["github-copilot-cli"],
        hooks_supported: false,
        config_install: None,
    },
    AgentConfig {
        name: "deepseek",
        binaries: &["deepseek"],
        hooks_supported: false,
        config_install: None,
    },
    AgentConfig {
        name: "antigravity",
        binaries: &["antigravity"],
        hooks_supported: false,
        config_install: None,
    },
];

type ConfigInstallFn = fn(&AgentConfig) -> Result<bool, String>;

struct AgentConfig {
    name: &'static str,
    binaries: &'static [&'static str],
    hooks_supported: bool,
    config_install: Option<ConfigInstallFn>,
}

fn find_binary(name: &str) -> Option<String> {
    std::env::var_os("PATH")
        .as_ref()
        .and_then(|paths| {
            std::env::split_paths(paths).find_map(|dir| {
                let full_path = dir.join(name);
                if full_path.is_file() {
                    Some(full_path.to_string_lossy().to_string())
                } else {
                    None
                }
            })
        })
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn read_json_file(path: &PathBuf) -> Result<serde_json::Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    serde_json::from_str(&content).map_err(|e| format!("invalid JSON in {}: {}", path.display(), e))
}

fn write_json_file(path: &PathBuf, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {}", parent.display(), e))?;
    }
    let content = serde_json::to_string_pretty(value)
        .map_err(|e| format!("serialization error: {}", e))?;
    std::fs::write(path, &content).map_err(|e| format!("cannot write {}: {}", path.display(), e))
}

fn install_claude_hooks(_cfg: &AgentConfig) -> Result<bool, String> {
    let settings_path = expand_home("~/.claude/settings.json");

    let mut settings: serde_json::Value = if settings_path.exists() {
        read_json_file(&settings_path)?
    } else {
        serde_json::json!({})
    };

    let hook_binary = find_binary("agentos-hook")
        .unwrap_or_else(|| "agentos-hook".to_string());

    let hooks = serde_json::json!({
        "UserPromptSubmit": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "SessionStart": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "SessionEnd": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "Stop": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "StopFailure": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "PreToolUse": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 86400 }] }],
        "PostToolUse": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "PostToolUseFailure": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "PermissionRequest": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 86400 }] }],
        "PermissionDenied": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "Notification": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "SubagentStart": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "SubagentStop": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }],
        "PreCompact": [{ "matcher": "*", "hooks": [{ "type": "command", "command": format!("{} --source claude", hook_binary), "timeout": 5 }] }]
    });

    settings["hooks"] = hooks;
    write_json_file(&settings_path, &settings)?;
    info!(path = %settings_path.display(), "claude hooks installed");
    Ok(true)
}

fn install_opencode_hooks(_cfg: &AgentConfig) -> Result<bool, String> {
    let config_path = expand_home("~/.config/opencode/opencode.json");
    let plugin_src = find_plugin_js_path();

    let mut config: serde_json::Value = if config_path.exists() {
        read_json_file(&config_path)?
    } else {
        serde_json::json!({})
    };

    let hook_binary = find_binary("agentos-hook")
        .unwrap_or_else(|| "agentos-hook".to_string());

    let hooks = serde_json::json!({
        "preTask": format!("{} --source opencode --event", hook_binary),
        "postTask": format!("{} --source opencode --event", hook_binary),
        "onPermission": format!("{} --source opencode --event", hook_binary),
        "onError": format!("{} --source opencode --event", hook_binary)
    });

    config["hooks"] = hooks;
    write_json_file(&config_path, &config)?;
    info!(path = %config_path.display(), "opencode hooks installed");

    if let Some(src) = plugin_src {
        let plugin_dir = expand_home("~/.config/opencode/plugins");
        let plugin_dest = plugin_dir.join("agentos.js");
        std::fs::create_dir_all(&plugin_dir)
            .map_err(|e| format!("cannot create plugin dir: {}", e))?;
        std::fs::copy(&src, &plugin_dest)
            .map_err(|e| format!("cannot copy plugin: {}", e))?;
        info!(from = %src.display(), to = %plugin_dest.display(), "opencode JS plugin installed");

        let plugin_uri = format!("file://{}", plugin_dest.display());
        let plugins = config["plugin"]
            .as_array_mut()
            .map(|arr| {
                if !arr.iter().any(|p| p.as_str() == Some(&plugin_uri)) {
                    arr.push(serde_json::json!(plugin_uri));
                }
            });

        if plugins.is_none() {
            config["plugin"] = serde_json::json!([plugin_uri]);
            write_json_file(&config_path, &config)?;
        }
    }

    Ok(true)
}

fn find_plugin_js_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("plugins/opencode/js/agentos-opencode.js"),
        PathBuf::from("../plugins/opencode/js/agentos-opencode.js"),
        expand_home("~/.config/opencode/plugins/agentos.js"),
    ];
    let exe_dir = std::env::current_exe().ok()?;
    let exe_parent = exe_dir.parent()?;

    let relative = [
        exe_parent.join("plugins/opencode/js/agentos-opencode.js"),
        exe_parent.join("../plugins/opencode/js/agentos-opencode.js"),
    ];

    for path in candidates.iter().chain(relative.iter()) {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

pub fn discover_agents() -> DiscoverResult {
    let mut agents = Vec::new();
    let mut installed_count = 0;
    let mut hooks_installed_count = 0;

    for agent in AGENTS {
        let binary = agent.binaries.iter().find_map(|b| find_binary(b));
        let installed = binary.is_some();

        let hooks_installed = if installed && agent.hooks_supported {
            match agent.config_install {
                Some(install_fn) => match install_fn(agent) {
                    Ok(true) => {
                        hooks_installed_count += 1;
                        true
                    }
                    Ok(false) => false,
                    Err(e) => {
                        warn!(agent = %agent.name, error = %e, "hook installation failed");
                        false
                    }
                },
                None => false,
            }
        } else {
            false
        };

        let config_path = agent_config_path(agent.name);

        let message = if !installed {
            format!("{} not found in PATH", agent.name)
        } else if agent.hooks_supported && hooks_installed {
            "hooks installed and active".to_string()
        } else if agent.hooks_supported {
            "binary found but hooks may not be configured".to_string()
        } else {
            "binary found — use agentos-hook manually to send events".to_string()
        };

        if installed {
            installed_count += 1;
        }

        agents.push(AgentInfo {
            name: agent.name.to_string(),
            binary,
            installed,
            hooks_supported: agent.hooks_supported,
            hooks_installed,
            config_path,
            message,
        });
    }

    let daemon_socket = find_binary("agentos-hook").map(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{}/.agentosd.sock", home)
    });

    DiscoverResult {
        agents,
        daemon_socket,
        total_agents: AGENTS.len(),
        installed_count,
        hooks_installed_count,
    }
}

fn agent_config_path(name: &str) -> Option<String> {
    match name {
        "claude" => Some(
            expand_home("~/.claude/settings.json")
                .to_string_lossy()
                .to_string(),
        ),
        "opencode" => Some(
            expand_home("~/.config/opencode/opencode.json")
                .to_string_lossy()
                .to_string(),
        ),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn find_all_binaries() -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    for agent in AGENTS {
        let found: Vec<String> = agent
            .binaries
            .iter()
            .filter_map(|b| find_binary(b))
            .collect();
        if !found.is_empty() {
            result.insert(agent.name.to_string(), found);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_returns_all_agents() {
        let result = discover_agents();
        assert_eq!(result.total_agents, 9);
        assert_eq!(result.agents.len(), 9);
    }

    #[test]
    fn test_discover_structure() {
        let result = discover_agents();
        for agent in &result.agents {
            assert!(!agent.name.is_empty());
            assert!(!agent.message.is_empty());
        }
    }

    #[test]
    fn test_find_binary_self() {
        let result = find_binary("sh");
        assert!(result.is_some());
    }

    #[test]
    fn test_find_binary_nonexistent() {
        let result = find_binary("thiscannotexist_xyz_999");
        assert!(result.is_none());
    }

    #[test]
    fn test_expand_home() {
        let home = std::env::var("HOME").unwrap();
        let expanded = expand_home("~/.claude/settings.json");
        assert!(expanded.to_string_lossy().contains(&home));
        assert!(expanded.to_string_lossy().ends_with(".claude/settings.json"));
    }

    #[test]
    fn test_find_all_binaries() {
        let _bins = find_all_binaries();
    }
}
