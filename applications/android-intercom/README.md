# Conduit Android Intercom

Reference Android application for validating the Conduit framework (see [EXAMPLE.md](../../EXAMPLE.md)).

## Architecture

```
UI (Capacitor / TypeScript)
        ↓
Conduit Capacitor Plugin (Kotlin)
        ↓
conduit-ffi (Rust cdylib + JNI)
        ↓
conduit-sdk → Conduit Framework
```

The UI contains **no communication logic** — only diagnostics and controls.

## Prerequisites

- Node.js 20+
- Rust toolchain (`rustup`)
- Java 17+ (`java`)
- `adb` (Android platform-tools)
- Android SDK at `~/Android/Sdk` or set `ANDROID_SDK_ROOT`

**No Android Studio required.**

## One-time setup

```bash
cd applications/android-intercom
npm install
npm run android:setup    # installs NDK, Rust Android targets, cargo-ndk
```

## Build & run on device (CLI only)

Connect a phone with USB debugging, then:

```bash
cd applications/android-intercom
npm run android:run    # build APK + install + launch
```

Or step by step:

```bash
npm run android:build    # web + Rust native + Gradle APK
npm run android:install  # adb install + start app
```

APK output: `android/app/build/outputs/apk/debug/app-debug.apk`

## Web development (browser mock)

```bash
npm run dev
```

Open http://localhost:5173 — uses the web mock plugin with simulated peers.

## Environment

| Variable | Default |
|----------|---------|
| `ANDROID_SDK_ROOT` | `~/Android/Sdk` |
| `ANDROID_NDK_HOME` | `$ANDROID_SDK_ROOT/ndk/26.1.10909125` |
| `CONDUIT_ANDROID_ABIS` | `arm64-v8a` (set to `all` for emulator ABIs) |

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
