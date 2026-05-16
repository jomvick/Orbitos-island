use std::collections::HashMap;
use std::sync::Arc;

use super::traits::{AgentPlugin, PluginResult};
use crate::state::{AgentKind, UniversalEvent};

#[derive(Default)]
pub struct AgentRegistry {
    plugins: HashMap<String, Arc<dyn AgentPlugin>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Arc<dyn AgentPlugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn AgentPlugin>> {
        self.plugins.get(name)
    }

    pub fn process(&self, agent: &AgentKind, payload: &str) -> PluginResult {
        let name = agent.to_string();
        match self.plugins.get(&name) {
            Some(plugin) => plugin.parse(payload),
            None => {
                let event: UniversalEvent = serde_json::from_str(payload)
                    .map_err(|e| super::traits::PluginError::ParseError(e.to_string()))?;
                Ok(Some(event))
            }
        }
    }

    pub fn registered_agents(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}
