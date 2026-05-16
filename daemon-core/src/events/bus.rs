use std::sync::Arc;

use tokio::sync::broadcast;

use crate::state::UniversalEvent;

const CHANNEL_CAPACITY: usize = 1024;

#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<Arc<UniversalEvent>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    pub fn publish(&self, event: Arc<UniversalEvent>) -> Result<usize, EventBusError> {
        self.tx
            .send(event)
            .map_err(|e| EventBusError::ChannelFull(e.0))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<UniversalEvent>> {
        self.tx.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("event bus channel full, dropped event")]
    ChannelFull(Arc<UniversalEvent>),
    #[error("no subscribers")]
    NoSubscribers,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AgentKind, EventKind, UniversalEvent};
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_event() -> Arc<UniversalEvent> {
        Arc::new(UniversalEvent {
            id: Uuid::new_v4(),
            agent: AgentKind::Opencode,
            event: EventKind::SessionStarted,
            session_id: "test-session".to_string(),
            cwd: None,
            branch: None,
            model: None,
            tokens_input: None,
            tokens_output: None,
            duration_ms: None,
            terminal: None,
            pane: None,
            permission: None,
            question: None,
            jump_target: None,
            error: None,
            metadata: None,
            timestamp: Utc::now(),
        })
    }

    #[test]
    fn test_publish_and_receive() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        let event = sample_event();

        bus.publish(event.clone()).unwrap();
        let received = rx.try_recv().unwrap();
        assert_eq!(received.session_id, "test-session");
        assert_eq!(received.agent, AgentKind::Opencode);
    }

    #[test]
    fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        let event = sample_event();

        bus.publish(event.clone()).unwrap();
        let r1 = rx1.try_recv().unwrap();
        let r2 = rx2.try_recv().unwrap();
        assert_eq!(r1.session_id, r2.session_id);
    }

    #[test]
    fn test_no_message_without_publish() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        let result = rx.try_recv();
        assert!(result.is_err());
    }

    #[test]
    fn test_subscriber_count() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);
        let _rx1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);
        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }
}

