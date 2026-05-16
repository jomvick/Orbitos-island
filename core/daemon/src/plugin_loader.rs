use std::sync::Arc;

use daemon_core::agents::{AgentPlugin, AgentRegistry};
use tracing::info;

pub fn load_default_plugins() -> AgentRegistry {
    let mut registry = AgentRegistry::new();

    let plugins: Vec<(&str, Arc<dyn AgentPlugin>)> = vec![
        ("opencode", Arc::new(plugin_opencode::OpenCodePlugin)),
        ("claude", Arc::new(plugin_claude::ClaudePlugin)),
        ("codex", Arc::new(plugin_codex::CodexPlugin)),
        (
            "antigravity",
            Arc::new(plugin_antigravity::AntigravityPlugin),
        ),
        ("aider", Arc::new(plugin_aider::AiderPlugin)),
        ("gemini", Arc::new(plugin_gemini::GeminiPlugin)),
        ("cursor", Arc::new(plugin_cursor::CursorPlugin)),
        ("copilot", Arc::new(plugin_copilot::CopilotPlugin)),
        ("deepseek", Arc::new(plugin_deepseek::DeepSeekPlugin)),
    ];

    for (name, plugin) in &plugins {
        registry.register(Arc::clone(plugin));
        info!(plugin = %name, "registered plugin");
    }

    info!(count = %plugins.len(), "plugins loaded");
    registry
}
