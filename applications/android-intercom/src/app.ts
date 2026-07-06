import { conduitBridge } from "./conduit/bridge";
import type { AppState, TestMode, VoiceMode } from "./types";

const SCREENS = [
  "dashboard",
  "discovery",
  "mesh",
  "voice",
  "stats",
  "packets",
  "logs",
] as const;

type Screen = (typeof SCREENS)[number];

export class IntercomApp {
  private root: HTMLElement;
  private screen: Screen = "dashboard";
  private tickTimer: number | null = null;
  private state: AppState = {
    initialized: false,
    joined: false,
    nodeName: "android-node",
    voiceMode: "ptt",
    testMode: "single",
    diagnostics: null,
    stats: null,
    packets: [],
    logs: [],
    pttActive: false,
    codec: "linear",
    packetRate: 0,
    lastTickMs: 0,
  };

  constructor(root: HTMLElement) {
    this.root = root;
    this.render();
  }

  private render(): void {
    this.root.innerHTML = `
      <header>
        <h1>Conduit Android Intercom</h1>
        <p id="subtitle">Framework validation reference app</p>
      </header>
      <nav id="nav"></nav>
      <main id="main"></main>
      <footer id="footer">Not connected</footer>
    `;
    this.renderNav();
    this.renderScreen();
  }

  private renderNav(): void {
    const nav = this.root.querySelector("#nav")!;
    nav.innerHTML = SCREENS.map(
      (s) =>
        `<button data-screen="${s}" class="${s === this.screen ? "active" : ""}">${label(s)}</button>`,
    ).join("");
    nav.querySelectorAll("button").forEach((btn) => {
      btn.addEventListener("click", () => {
        this.screen = btn.getAttribute("data-screen") as Screen;
        this.renderNav();
        this.renderScreen();
      });
    });
  }

  private renderScreen(): void {
    const main = this.root.querySelector("#main")!;
    switch (this.screen) {
      case "dashboard":
        main.innerHTML = this.dashboardHtml();
        this.bindDashboard();
        break;
      case "discovery":
        main.innerHTML = this.discoveryHtml();
        break;
      case "mesh":
        main.innerHTML = this.meshHtml();
        break;
      case "voice":
        main.innerHTML = this.voiceHtml();
        this.bindVoice();
        break;
      case "stats":
        main.innerHTML = this.statsHtml();
        break;
      case "packets":
        main.innerHTML = this.packetsHtml();
        break;
      case "logs":
        main.innerHTML = this.logsHtml();
        this.bindLogs();
        break;
    }
    this.updateFooter();
  }

  private dashboardHtml(): string {
    return `
      <section class="panel">
        <h2>Session</h2>
        <div class="row">
          <input id="node-name" value="${this.state.nodeName}" placeholder="Device name" />
          <select id="test-mode">
            ${testModeOptions(this.state.testMode)}
          </select>
        </div>
        <div class="row">
          <button class="primary" id="btn-init">Initialize</button>
          <button class="primary" id="btn-join" ${this.state.initialized ? "" : "disabled"}>Join Network</button>
          <button class="danger" id="btn-leave" ${this.state.joined ? "" : "disabled"}>Leave</button>
        </div>
        <p style="color:var(--muted);font-size:0.85rem;margin:0">
          Test modes: single device, two/three device, mesh, mobility, battery.
          No communication logic lives in this UI — all networking is in Conduit.
        </p>
      </section>
      <section class="panel">
        <h2>Overview</h2>
        <div class="grid">
          ${stat("Node", this.state.diagnostics?.node_name ?? "—")}
          ${stat("Neighbors", String(this.state.stats?.connected_nodes ?? 0))}
          ${stat("Routes", String(this.state.stats?.routes ?? 0))}
          ${stat("RTT", `${this.state.stats?.current_rtt_ms ?? 0} ms`)}
        </div>
      </section>
    `;
  }

  private discoveryHtml(): string {
    const neighbors = this.state.diagnostics?.neighbors ?? [];
    if (neighbors.length === 0) {
      return `<section class="panel"><h2>Nearby Devices</h2><p>No peers discovered yet.</p></section>`;
    }
    const rows = neighbors
      .map(
        (n) => `<tr>
          <td>${n.name}</td>
          <td>${shortId(n.node_id)}</td>
          <td>${n.signal_strength ?? "—"} dBm</td>
          <td>${Math.round(n.link_quality * 100)}%</td>
          <td>${n.state}</td>
          <td>${timeAgo(n.last_seen_ms)}</td>
        </tr>`,
      )
      .join("");
    return `
      <section class="panel">
        <h2>Nearby Devices</h2>
        <table>
          <thead><tr><th>Name</th><th>Node ID</th><th>Signal</th><th>Link</th><th>State</th><th>Last Seen</th></tr></thead>
          <tbody>${rows}</tbody>
        </table>
      </section>`;
  }

  private meshHtml(): string {
    const local = this.state.diagnostics?.node_name ?? "This device";
    const neighbors = this.state.diagnostics?.neighbors ?? [];
    let tree = `${local}\n│`;
    if (neighbors.length === 0) {
      tree += "\n└── (no neighbors)";
    } else {
      neighbors.forEach((n, i) => {
        const branch = i === neighbors.length - 1 ? "└" : "├";
        tree += `\n${branch}──── ${n.name}`;
      });
    }
    return `<section class="panel"><h2>Mesh Topology (debug)</h2><div class="mesh-tree">${tree}</div></section>`;
  }

  private voiceHtml(): string {
    return `
      <section class="panel">
        <h2>Voice Testing</h2>
        <div class="row">
          <select id="voice-mode">
            <option value="ptt" ${this.state.voiceMode === "ptt" ? "selected" : ""}>Push-To-Talk</option>
            <option value="continuous" ${this.state.voiceMode === "continuous" ? "selected" : ""}>Continuous</option>
            <option value="vad" ${this.state.voiceMode === "vad" ? "selected" : ""}>Voice Activity Detection</option>
          </select>
          <button class="primary" id="btn-audio-start">Start Audio</button>
          <button id="btn-audio-stop">Stop Audio</button>
        </div>
        <button class="ptt ${this.state.pttActive ? "active" : ""}" id="btn-ptt">
          ${this.state.pttActive ? "Transmitting…" : "Hold to Talk"}
        </button>
        <div class="grid" style="margin-top:1rem">
          ${stat("Codec", this.state.codec)}
          ${stat("Packet rate", `${this.state.packetRate}/s`)}
          ${stat("Latency", `${this.state.stats?.average_latency_ms ?? 0} ms`)}
          ${stat("Voice RX", String(this.state.stats?.voice_frames_received ?? 0))}
        </div>
      </section>`;
  }

  private statsHtml(): string {
    const s = this.state.stats;
    return `
      <section class="panel">
        <h2>Network Statistics</h2>
        <div class="grid">
          ${stat("Connected nodes", String(s?.connected_nodes ?? 0))}
          ${stat("Routes", String(s?.routes ?? 0))}
          ${stat("Packets sent", String(s?.packets_sent ?? 0))}
          ${stat("Packets received", String(s?.packets_received ?? 0))}
          ${stat("Forwarded", String(s?.packets_forwarded ?? 0))}
          ${stat("Duplicates", String(s?.duplicates ?? 0))}
          ${stat("Drops", String(s?.drops ?? 0))}
          ${stat("Current RTT", `${s?.current_rtt_ms ?? 0} ms`)}
          ${stat("Avg latency", `${s?.average_latency_ms ?? 0} ms`)}
          ${stat("Bandwidth", `${formatBytes(s?.bandwidth_bytes ?? 0)}`)}
        </div>
      </section>`;
  }

  private packetsHtml(): string {
    const rows = this.state.packets
      .slice()
      .reverse()
      .slice(0, 50)
      .map(
        (p) => `<tr>
          <td>${p.id}</td><td>${p.packet_type}</td><td>${p.direction}</td>
          <td>${shortId(p.source)}</td><td>${shortId(p.destination)}</td>
          <td>${p.ttl}</td><td>${p.sequence}</td><td>${p.size_bytes} B</td>
        </tr>`,
      )
      .join("");
    return `
      <section class="panel">
        <h2>Packet Inspector</h2>
        <table>
          <thead><tr><th>ID</th><th>Type</th><th>Dir</th><th>Source</th><th>Dest</th><th>TTL</th><th>Seq</th><th>Size</th></tr></thead>
          <tbody>${rows || "<tr><td colspan=8>No packets yet</td></tr>"}</tbody>
        </table>
      </section>`;
  }

  private logsHtml(): string {
    const items = this.state.logs
      .slice()
      .reverse()
      .slice(0, 100)
      .map(
        (l) =>
          `<div class="log-entry ${l.level}">[${new Date(l.timestamp_ms).toLocaleTimeString()}] ${l.level.toUpperCase()} — ${escapeHtml(l.message)}</div>`,
      )
      .join("");
    return `
      <section class="panel">
        <h2>Event Log</h2>
        <button class="primary" id="btn-export">Export Logs</button>
        <div class="log-list" style="margin-top:0.75rem">${items || "<p>No events yet.</p>"}</div>
      </section>`;
  }

  private bindDashboard(): void {
    const nameInput = this.root.querySelector<HTMLInputElement>("#node-name")!;
    const testMode = this.root.querySelector<HTMLSelectElement>("#test-mode")!;
    this.root.querySelector("#btn-init")!.addEventListener("click", async () => {
      this.state.nodeName = nameInput.value.trim() || "android-node";
      this.state.testMode = testMode.value as TestMode;
      await conduitBridge.initialize(this.state.nodeName);
      this.state.initialized = true;
      this.renderScreen();
    });
    this.root.querySelector("#btn-join")!.addEventListener("click", async () => {
      await conduitBridge.join();
      this.state.joined = true;
      this.startTickLoop();
      this.renderScreen();
    });
    this.root.querySelector("#btn-leave")!.addEventListener("click", async () => {
      await this.stopTickLoop();
      await conduitBridge.leave();
      this.state.joined = false;
      this.renderScreen();
    });
  }

  private bindVoice(): void {
    const mode = this.root.querySelector<HTMLSelectElement>("#voice-mode")!;
    mode.addEventListener("change", async () => {
      this.state.voiceMode = mode.value as VoiceMode;
      await conduitBridge.setVoiceMode(this.state.voiceMode);
    });
    const ptt = this.root.querySelector<HTMLButtonElement>("#btn-ptt")!;
    const setPtt = async (active: boolean) => {
      this.state.pttActive = active;
      await conduitBridge.setPtt(active);
      ptt.classList.toggle("active", active);
      ptt.textContent = active ? "Transmitting…" : "Hold to Talk";
    };
    ptt.addEventListener("mousedown", () => void setPtt(true));
    ptt.addEventListener("mouseup", () => void setPtt(false));
    ptt.addEventListener("mouseleave", () => void setPtt(false));
    ptt.addEventListener("touchstart", (e) => {
      e.preventDefault();
      void setPtt(true);
    });
    ptt.addEventListener("touchend", () => void setPtt(false));
    this.root.querySelector("#btn-audio-start")!.addEventListener("click", () => {
      void conduitBridge.startAudio();
    });
    this.root.querySelector("#btn-audio-stop")!.addEventListener("click", () => {
      void conduitBridge.stopAudio();
    });
  }

  private bindLogs(): void {
    this.root.querySelector("#btn-export")!.addEventListener("click", async () => {
      const json = await conduitBridge.exportLogs();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `conduit-logs-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    });
  }

  private startTickLoop(): void {
    if (this.tickTimer !== null) return;
    let eventsLastSecond = 0;
    let lastRateCheck = Date.now();
    this.tickTimer = window.setInterval(async () => {
      try {
        const tick = await conduitBridge.tick();
        this.state.diagnostics = tick.diagnostics;
        this.state.stats = tick.stats;
        this.state.lastTickMs = tick.stats.current_rtt_ms;
        eventsLastSecond += tick.events.length;
        const now = Date.now();
        if (now - lastRateCheck >= 1000) {
          this.state.packetRate = eventsLastSecond;
          eventsLastSecond = 0;
          lastRateCheck = now;
        }
        this.state.packets = await conduitBridge.packets();
        this.state.logs = await conduitBridge.logs();
        if (this.screen !== "voice") {
          this.renderScreen();
        } else {
          this.updateFooter();
        }
      } catch (err) {
        console.error(err);
      }
    }, 1000);
  }

  private async stopTickLoop(): Promise<void> {
    if (this.tickTimer !== null) {
      window.clearInterval(this.tickTimer);
      this.tickTimer = null;
    }
  }

  private updateFooter(): void {
    const footer = this.root.querySelector("#footer")!;
    const joined = this.state.joined ? "joined" : "offline";
    const neighbors = this.state.stats?.connected_nodes ?? 0;
    footer.textContent = `${joined} · ${neighbors} neighbors · tick ${this.state.lastTickMs}ms`;
  }
}

function label(screen: Screen): string {
  const map: Record<Screen, string> = {
    dashboard: "Dashboard",
    discovery: "Discovery",
    mesh: "Mesh",
    voice: "Voice",
    stats: "Stats",
    packets: "Packets",
    logs: "Logs",
  };
  return map[screen];
}

function stat(label: string, value: string): string {
  return `<div class="stat"><div class="label">${label}</div><div class="value">${value}</div></div>`;
}

function testModeOptions(current: TestMode): string {
  const modes: { id: TestMode; label: string }[] = [
    { id: "single", label: "Single device" },
    { id: "two-device", label: "Two devices" },
    { id: "three-device", label: "Three devices" },
    { id: "mesh", label: "Mesh test" },
    { id: "mobility", label: "Mobility test" },
    { id: "battery", label: "Battery test" },
  ];
  return modes
    .map((m) => `<option value="${m.id}" ${m.id === current ? "selected" : ""}>${m.label}</option>`)
    .join("");
}

function shortId(id: string): string {
  return id.length > 12 ? `${id.slice(0, 8)}…` : id;
}

function timeAgo(ms: number): string {
  const delta = Date.now() - ms;
  if (delta < 5000) return "just now";
  return `${Math.round(delta / 1000)}s ago`;
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  return `${(n / 1024).toFixed(1)} KB`;
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
