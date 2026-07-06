import { Conduit } from "conduit-plugin";
import type {
  LogEntry,
  NodeDiagnostics,
  PacketRecord,
  TickResponse,
  VoiceMode,
} from "../types";

function unwrapArray<T>(value: T[] | { value: T[] }): T[] {
  if (Array.isArray(value)) return value;
  if (value && typeof value === "object" && "value" in value) {
    return (value as { value: T[] }).value;
  }
  return [];
}

function unwrapString(value: string | { value: string }): string {
  if (typeof value === "string") return value;
  if (value && typeof value === "object" && "value" in value) {
    return (value as { value: string }).value;
  }
  return "";
}

export const conduitBridge = {
  async initialize(name: string): Promise<void> {
    await Conduit.initialize({ name });
  },

  async join(): Promise<void> {
    await Conduit.joinNetwork();
  },

  async leave(): Promise<void> {
    await Conduit.stopAudioCapture();
    await Conduit.leaveNetwork();
  },

  async tick(): Promise<TickResponse> {
    return Conduit.tick();
  },

  async setVoiceMode(mode: VoiceMode): Promise<void> {
    await Conduit.setVoiceMode({ mode });
  },

  async setPtt(active: boolean): Promise<void> {
    await Conduit.setPushToTalk({ active });
  },

  async startAudio(): Promise<void> {
    await Conduit.startAudioCapture();
  },

  async stopAudio(): Promise<void> {
    await Conduit.stopAudioCapture();
  },

  async diagnostics(): Promise<NodeDiagnostics> {
    return Conduit.getDiagnostics();
  },

  async packets(): Promise<PacketRecord[]> {
    const raw = await Conduit.getPacketLog();
    return unwrapArray(raw as PacketRecord[] | { value: PacketRecord[] });
  },

  async logs(): Promise<LogEntry[]> {
    const raw = await Conduit.getEventLog();
    return unwrapArray(raw as LogEntry[] | { value: LogEntry[] });
  },

  async exportLogs(): Promise<string> {
    const raw = await Conduit.exportLogs();
    return unwrapString(raw as string | { value: string });
  },

  async version(): Promise<string> {
    const v = await Conduit.getVersion();
    return `${v.crate} (protocol ${v.protocol})`;
  },
};
