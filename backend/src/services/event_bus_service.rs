use tokio::sync::broadcast;

use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEvent {
    pub id: Uuid,
    pub topic: String,
    pub payload: JsonValue,
    pub occurred_at: DateTime<Utc>,
}

impl AppEvent {
    pub fn new(topic: impl Into<String>, payload: JsonValue) -> Self {
        Self {
            id: Uuid::new_v4(),
            topic: topic.into(),
            payload,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Clone)]
pub struct EventBusService {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBusService {
    const DEFAULT_CAPACITY: usize = 256;

    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(1));
        Self { sender }
    }

    pub fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: AppEvent) -> usize {
        match self.sender.send(event) {
            Ok(receiver_count) => receiver_count,
            Err(_) => 0,
        }
    }

    pub fn emit(&self, topic: impl Into<String>, payload: JsonValue) -> usize {
        self.publish(AppEvent::new(topic, payload))
    }

    pub fn emit_empty(&self, topic: impl Into<String>) -> usize {
        self.emit(topic, JsonValue::Null)
    }
}
