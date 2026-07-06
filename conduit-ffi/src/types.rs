use conduit_core::PacketType;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PacketRecord {
  pub id: u64,
  pub packet_type: String,
  pub source: String,
  pub destination: String,
  pub ttl: u8,
  pub sequence: u32,
  pub timestamp_ms: u64,
  pub size_bytes: usize,
  pub direction: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
  pub timestamp_ms: u64,
  pub level: String,
  pub message: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct NetworkStats {
  pub connected_nodes: usize,
  pub routes: usize,
  pub packets_sent: u64,
  pub packets_received: u64,
  pub packets_forwarded: u64,
  pub duplicates: u64,
  pub drops: u64,
  pub voice_frames_sent: u64,
  pub voice_frames_received: u64,
  pub current_rtt_ms: u64,
  pub average_latency_ms: u64,
  pub bandwidth_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TickResponse {
  pub events: Vec<String>,
  pub stats: NetworkStats,
  pub diagnostics: conduit_sdk::NodeDiagnostics,
  pub playback_samples: Option<Vec<i16>>,
}

pub fn packet_type_name(kind: PacketType) -> &'static str {
  match kind {
    PacketType::Heartbeat => "heartbeat",
    PacketType::Discovery => "discovery",
    PacketType::Routing => "routing",
    PacketType::Voice => "voice",
    PacketType::Location => "location",
    PacketType::Telemetry => "telemetry",
    PacketType::Emergency => "emergency",
    PacketType::Messaging => "messaging",
    PacketType::Control => "control",
  }
}
