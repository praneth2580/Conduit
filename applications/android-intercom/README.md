# Conduit Android Intercom

Reference Android application for validating the Conduit framework (see [EXAMPLE.md](../../EXAMPLE.md)).

## Architecture

```
UI (Capacitor / TypeScript)
        ↓
Conduit Capacitor Plugin (Kotlin)
        ↓
conduit-ffi (Rust cdylib)
        ↓
conduit-sdk → Conduit Framework
```

The UI contains **no communication logic** — only diagnostics and controls.

## Prerequisites

- Node.js 20+
- Rust toolchain
- Android Studio + Android NDK
- `ANDROID_NDK_HOME` set

## Web development (mock backend)

```bash
cd applications/android-intercom
npm install
npm run dev
```

Open http://localhost:5173 — uses the web mock plugin with simulated peers.

## Android build

```bash
# 1. Build Rust FFI for Android
./scripts/build-android-ffi.sh

# 2. Build web assets and sync Capacitor
cd applications/android-intercom
npm install
npm run cap:sync

# 3. Open in Android Studio
npm run cap:android
```

## Screens

| Screen | Purpose |
|--------|---------|
| Dashboard | Initialize / join / leave, test mode selection |
| Discovery | Nearby nodes, signal strength, link quality |
| Mesh | ASCII topology visualization |
| Voice | PTT, continuous, VAD modes + audio stats |
| Stats | Packets, routes, RTT, bandwidth |
| Packets | Packet inspector for routing debug |
| Logs | Exportable event log |

## Test modes

Single device, two/three device, mesh, mobility, and battery test modes are selectable on the dashboard for field testing workflows described in EXAMPLE.md.
