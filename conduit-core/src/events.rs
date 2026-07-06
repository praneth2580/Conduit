use crate::ids::NodeId;
use crate::packets::PacketType;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Categories of internal Conduit events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
  NodeStarted { node_id: NodeId },
  NodeStopped { node_id: NodeId },
  ConfigLoaded,
  PacketCreated { packet_type: PacketType, sequence: u32 },
  PacketDropped { packet_type: PacketType, reason: String },
  LogLevelChanged,
}

/// A timestamped event emitted within a Conduit node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
  pub kind: EventKind,
  pub timestamp_ms: u64,
}

impl Event {
  pub fn new(kind: EventKind) -> Self {
    Self {
      kind,
      timestamp_ms: crate::utils::unix_timestamp_ms(),
    }
  }
}

pub type EventHandler = Arc<dyn Fn(&Event) + Send + Sync>;

/// In-process pub/sub bus for core-level events.
///
/// Higher layers (mesh, routing, SDK) will publish through this without coupling
/// to specific subscribers.
#[derive(Default)]
pub struct EventBus {
  handlers: Mutex<Vec<EventHandler>>,
}

impl EventBus {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn subscribe<F>(&self, handler: F)
  where
    F: Fn(&Event) + Send + Sync + 'static,
  {
    self.handlers.lock().unwrap().push(Arc::new(handler));
  }

  pub fn publish(&self, event: Event) {
    let handlers = self.handlers.lock().unwrap();
    for handler in handlers.iter() {
      handler(&event);
    }
  }

  pub fn subscriber_count(&self) -> usize {
    self.handlers.lock().unwrap().len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};

  #[test]
  fn publish_notifies_subscribers() {
    let bus = EventBus::new();
    let count = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&count);

    bus.subscribe(move |_| {
      counter.fetch_add(1, Ordering::SeqCst);
    });

    bus.publish(Event::new(EventKind::ConfigLoaded));
    assert_eq!(count.load(Ordering::SeqCst), 1);
  }

  #[test]
  fn multiple_subscribers_receive_events() {
    let bus = EventBus::new();
    let a = Arc::new(AtomicUsize::new(0));
    let b = Arc::new(AtomicUsize::new(0));

    let a_clone = Arc::clone(&a);
    bus.subscribe(move |_| {
      a_clone.fetch_add(1, Ordering::SeqCst);
    });

    let b_clone = Arc::clone(&b);
    bus.subscribe(move |_| {
      b_clone.fetch_add(1, Ordering::SeqCst);
    });

    bus.publish(Event::new(EventKind::ConfigLoaded));
    assert_eq!(a.load(Ordering::SeqCst), 1);
    assert_eq!(b.load(Ordering::SeqCst), 1);
  }
}
