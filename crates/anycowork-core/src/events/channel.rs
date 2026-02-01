//! Event channel for broadcasting events to subscribers

use super::AgentEvent;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Default channel capacity
const DEFAULT_CAPACITY: usize = 256;

/// Event channel for platform adapters to subscribe to
#[derive(Clone)]
pub struct EventChannel {
    sender: Arc<broadcast::Sender<AgentEvent>>,
}

impl EventChannel {
    /// Create a new event channel with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a new event channel with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Emit an event to all subscribers
    pub fn emit(&self, event: AgentEvent) {
        // Ignore send errors (no receivers is ok)
        let _ = self.sender.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active receivers
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_channel_emit_receive() {
        let channel = EventChannel::new();
        let mut receiver = channel.subscribe();

        channel.emit(AgentEvent::Thinking {
            message: "Hello".to_string(),
        });

        let event = receiver.recv().await.unwrap();
        match event {
            AgentEvent::Thinking { message } => {
                assert_eq!(message, "Hello");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_event_channel_multiple_subscribers() {
        let channel = EventChannel::new();
        let mut receiver1 = channel.subscribe();
        let mut receiver2 = channel.subscribe();

        channel.emit(AgentEvent::Token {
            content: "test".to_string(),
        });

        let event1 = receiver1.recv().await.unwrap();
        let event2 = receiver2.recv().await.unwrap();

        match (event1, event2) {
            (AgentEvent::Token { content: c1 }, AgentEvent::Token { content: c2 }) => {
                assert_eq!(c1, "test");
                assert_eq!(c2, "test");
            }
            _ => panic!("Wrong event types"),
        }
    }

    #[test]
    fn test_receiver_count() {
        let channel = EventChannel::new();
        assert_eq!(channel.receiver_count(), 0);

        let _r1 = channel.subscribe();
        assert_eq!(channel.receiver_count(), 1);

        let _r2 = channel.subscribe();
        assert_eq!(channel.receiver_count(), 2);
    }
}
