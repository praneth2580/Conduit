mod udp;

use conduit_core::error::Result;
use conduit_core::NodeId;
use conduit_discovery::PeerEndpoint;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub use udp::UdpNetwork;

/// A single inbound transport frame from a remote peer.
#[derive(Debug, Clone)]
pub struct InboundFrame {
  pub from: NodeId,
  pub data: Vec<u8>,
}

/// Delivers encoded transport frames between nodes.
pub trait NetworkBackend: Send {
  fn send_frames(&mut self, to: NodeId, frames: &[Vec<u8>]) -> Result<()>;
  fn drain_inbound(&mut self) -> Vec<InboundFrame>;
  fn register_peer(&mut self, _node_id: NodeId, _endpoint: &PeerEndpoint) {}
}

/// Shared in-memory bus for simulations and unit tests.
#[derive(Debug, Default)]
pub struct SimBus {
  inboxes: HashMap<NodeId, VecDeque<InboundFrame>>,
}

impl SimBus {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn register(&mut self, node_id: NodeId) {
    self.inboxes.entry(node_id).or_default();
  }

  pub fn deliver(&mut self, from: NodeId, to: NodeId, data: Vec<u8>) {
    if let Some(inbox) = self.inboxes.get_mut(&to) {
      inbox.push_back(InboundFrame { from, data });
    }
  }

  pub fn drain(&mut self, node_id: NodeId) -> Vec<InboundFrame> {
    self
      .inboxes
      .get_mut(&node_id)
      .map(|q| q.drain(..).collect())
      .unwrap_or_default()
  }
}

/// Handle to a shared simulation bus.
#[derive(Debug, Clone)]
pub struct SimBusHandle {
  inner: Arc<Mutex<SimBus>>,
}

impl SimBusHandle {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(SimBus::new())),
    }
  }

  pub fn register(&self, node_id: NodeId) {
    self.inner.lock().expect("sim bus lock").register(node_id);
  }

  fn deliver(&self, from: NodeId, to: NodeId, data: Vec<u8>) {
    self
      .inner
      .lock()
      .expect("sim bus lock")
      .deliver(from, to, data);
  }

  fn drain(&self, node_id: NodeId) -> Vec<InboundFrame> {
    self
      .inner
      .lock()
      .expect("sim bus lock")
      .drain(node_id)
  }
}

/// In-memory network backend backed by a [`SimBusHandle`].
#[derive(Debug, Clone)]
pub struct SimNetwork {
  local_id: NodeId,
  bus: SimBusHandle,
}

impl SimNetwork {
  pub fn new(local_id: NodeId, bus: SimBusHandle) -> Self {
    bus.register(local_id);
    Self { local_id, bus }
  }
}

impl NetworkBackend for SimNetwork {
  fn send_frames(&mut self, to: NodeId, frames: &[Vec<u8>]) -> Result<()> {
    for frame in frames {
      self.bus.deliver(self.local_id, to, frame.clone());
    }
    Ok(())
  }

  fn drain_inbound(&mut self) -> Vec<InboundFrame> {
    self.bus.drain(self.local_id)
  }
}

/// No-op backend for initialize-only scenarios.
#[derive(Debug, Default)]
pub struct NullNetwork;

impl NetworkBackend for NullNetwork {
  fn send_frames(&mut self, _to: NodeId, _frames: &[Vec<u8>]) -> Result<()> {
    Ok(())
  }

  fn drain_inbound(&mut self) -> Vec<InboundFrame> {
    Vec::new()
  }
}
