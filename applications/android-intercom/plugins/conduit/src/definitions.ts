export interface NeighborInfo {
  node_id: string;
  name: string;
  signal_strength: number | null;
  link_quality: number;
  signal_quality: number;
  state: string;
  last_seen_ms: number;
}

export interface NodeDiagnostics {
  node_id: string;
  node_name: string;
  joined: boolean;
  neighbor_count: number;
  neighbors: NeighborInfo[];
  route_count: number;
  discovery_peer_count: number;
}

export interface NetworkStats {
  connected_nodes: number;
  routes: number;
  packets_sent: number;
  packets_received: number;
  packets_forwarded: number;
  duplicates: number;
  drops: number;
  voice_frames_sent: number;
  voice_frames_received: number;
  current_rtt_ms: number;
  average_latency_ms: number;
  bandwidth_bytes: number;
}

export interface PacketRecord {
  id: number;
  packet_type: string;
  source: string;
  destination: string;
  ttl: number;
  sequence: number;
  timestamp_ms: number;
  size_bytes: number;
  direction: string;
}

export interface LogEntry {
  timestamp_ms: number;
  level: string;
  message: string;
}

export interface TickResponse {
  events: string[];
  stats: NetworkStats;
  diagnostics: NodeDiagnostics;
  playback_samples?: number[] | null;
}

export interface ConduitPlugin {
  initialize(options: { name: string }): Promise<void>;
  joinNetwork(): Promise<void>;
  leaveNetwork(): Promise<void>;
  tick(): Promise<TickResponse>;
  setPushToTalk(options: { active: boolean }): Promise<void>;
  setVoiceMode(options: { mode: "ptt" | "continuous" | "vad" }): Promise<void>;
  sendVoice(options: { samples: number[] }): Promise<void>;
  getDiagnostics(): Promise<NodeDiagnostics>;
  getPacketLog(): Promise<PacketRecord[]>;
  getEventLog(): Promise<LogEntry[]>;
  exportLogs(): Promise<string>;
  getVersion(): Promise<{ crate: string; protocol: string }>;
  startAudioCapture(): Promise<void>;
  stopAudioCapture(): Promise<void>;
}
