use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct HookInput {
    pub source: Option<String>,
    pub event: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub session_id: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub metadata: Option<serde_json::Value>,
}

pub fn parse_input(input: &str) -> Result<HookInput, String> {
    // Try parsing as typed hook payload first
    if let Ok(hook) = serde_json::from_str::<HookInput>(input) {
        return Ok(HookInput {
            timestamp: hook.timestamp.or_else(|| Some(Utc::now())),
            ..hook
        });
    }

    // Try parsing as raw event (no source wrapper)
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(input) {
        return Ok(HookInput {
            source: None,
            event: Some(val),
            session_id: None,
            timestamp: Some(Utc::now()),
            metadata: None,
        });
    }

    Err("invalid JSON: expected an object or array".to_string())
}

pub fn resolve_source(
    cli_source: Option<String>,
    parsed_source: Option<String>,
) -> Result<String, String> {
    match (cli_source, parsed_source) {
        (Some(s), _) if !s.is_empty() => Ok(s),
        (_, Some(s)) if !s.is_empty() => Ok(s),
        _ => {
            Err("missing source: provide --source or include 'source' field in payload".to_string())
        }
    }
}
