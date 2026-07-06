import { WebPlugin } from "@capacitor/core";
import type {
  ConduitPlugin,
  LogEntry,
  NodeDiagnostics,
  PacketRecord,
  TickResponse,
} from "./definitions";

const emptyStats = () => ({
  connected_nodes: 0,
  routes: 0,
  packets_sent: 0,
  packets_received: 0,
  packets_forwarded: 0,
  duplicates: 0,
  drops: 0,
  voice_frames_sent: 0,
  voice_frames_received: 0,
  current_rtt_ms: 0,
  average_latency_ms: 0,
  bandwidth_bytes: 0,
});

export class ConduitWeb extends WebPlugin implements ConduitPlugin {
  private name = "web-node";
  private joined = false;
  private voiceMode: "ptt" | "continuous" | "vad" = "ptt";
  private ptt = false;
  private logs: LogEntry[] = [];
  private packets: PacketRecord[] = [];
  private mockNeighbors: NodeDiagnostics["neighbors"] = [];
  private tickCount = 0;
  private audioTimer: number | null = null;

  async initialize(options: { name: string }): Promise<void> {
    this.name = options.name;
    this.log("info", `initialized (web mock) as ${options.name}`);
  }

  async joinNetwork(): Promise<void> {
    this.joined = true;
    this.mockNeighbors = [
      {
        node_id: "00000000-0000-0000-0000-000000000002",
        name: "mock-peer-b",
        signal_strength: -55,
        link_quality: 0.82,
        signal_quality: 72,
        state: "active",
        last_seen_ms: Date.now(),
      },
    ];
    this.log("info", "joined network (mock)");
  }

  async leaveNetwork(): Promise<void> {
    this.joined = false;
    this.mockNeighbors = [];
    this.log("info", "left network");
  }

  async tick(): Promise<TickResponse> {
    this.tickCount += 1;
    const diagnostics: NodeDiagnostics = {
      node_id: "00000000-0000-0000-0000-000000000001",
      node_name: this.name,
      joined: this.joined,
      neighbor_count: this.mockNeighbors.length,
      neighbors: this.mockNeighbors,
      route_count: this.joined ? 1 : 0,
      discovery_peer_count: this.mockNeighbors.length,
    };
    const events: string[] = [];
    if (this.tickCount % 5 === 0 && this.joined) {
      events.push("heartbeat");
    }
    return {
      events,
      stats: {
        ...emptyStats(),
        connected_nodes: diagnostics.neighbor_count,
        routes: diagnostics.route_count,
        current_rtt_ms: 3 + (this.tickCount % 4),
        average_latency_ms: 5,
      },
      diagnostics,
    };
  }

  async setPushToTalk(options: { active: boolean }): Promise<void> {
    this.ptt = options.active;
    this.log("info", options.active ? "PTT pressed" : "PTT released");
  }

  async setVoiceMode(options: { mode: "ptt" | "continuous" | "vad" }): Promise<void> {
    this.voiceMode = options.mode;
    this.log("info", `voice mode: ${options.mode}`);
  }

  async sendVoice(options: { samples: number[] }): Promise<void> {
    this.packets.push({
      id: this.packets.length + 1,
      packet_type: "voice",
      source: this.name,
      destination: "broadcast",
      ttl: 8,
      sequence: this.packets.length,
      timestamp_ms: Date.now(),
      size_bytes: options.samples.length * 2,
      direction: "outbound",
    });
  }

  async getDiagnostics(): Promise<NodeDiagnostics> {
    return (await this.tick()).diagnostics;
  }

  async getPacketLog(): Promise<PacketRecord[]> {
    return [...this.packets];
  }

  async getEventLog(): Promise<LogEntry[]> {
    return [...this.logs];
  }

  async exportLogs(): Promise<string> {
    return JSON.stringify(
      {
        exported_at_ms: Date.now(),
        logs: this.logs,
        packets: this.packets,
      },
      null,
      2,
    );
  }

  async getVersion(): Promise<{ crate: string; protocol: string }> {
    return { crate: "0.1.0-web", protocol: "0.1" };
  }

  async startAudioCapture(): Promise<void> {
    if (this.audioTimer !== null) return;
    this.log("info", "audio capture started (mock)");
    this.audioTimer = window.setInterval(() => {
      if (this.voiceMode === "ptt" && !this.ptt) return;
      const samples = Array.from({ length: 960 }, () => Math.floor(Math.random() * 200 - 100));
      void this.sendVoice({ samples });
    }, 20);
  }

  async stopAudioCapture(): Promise<void> {
    if (this.audioTimer !== null) {
      window.clearInterval(this.audioTimer);
      this.audioTimer = null;
    }
    this.log("info", "audio capture stopped");
  }

  private log(level: string, message: string): void {
    this.logs.push({ timestamp_ms: Date.now(), level, message });
    if (this.logs.length > 500) this.logs.shift();
  }
}
