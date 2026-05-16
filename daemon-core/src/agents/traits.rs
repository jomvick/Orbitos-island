use crate::state::UniversalEvent;

pub type PluginResult = Result<Option<UniversalEvent>, PluginError>;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("unsupported event: {0}")]
    UnsupportedEvent(String),
}

pub trait AgentPlugin: Send + Sync {
    fn name(&self) -> &'static str;

    fn agent_kind(&self) -> crate::state::AgentKind;

    fn parse(&self, payload: &str) -> PluginResult;

    fn validate(&self, payload: &str) -> Result<(), PluginError> {
        serde_json::from_str::<serde_json::Value>(payload)
            .map(|_| ())
            .map_err(|e| PluginError::ParseError(e.to_string()))
    }
}
