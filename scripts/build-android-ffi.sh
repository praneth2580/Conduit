#!/usr/bin/env bash
# Cross-compile conduit-ffi (Rust + JNI) into the Capacitor plugin jniLibs.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT/applications/android-intercom/plugins/conduit/android/src/main/jniLibs"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-$HOME/Android/Sdk}}"
export ANDROID_HOME="$ANDROID_SDK_ROOT"

if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
  if [[ -d "$ANDROID_SDK_ROOT/ndk/26.1.10909125" ]]; then
    export ANDROID_NDK_HOME="$ANDROID_SDK_ROOT/ndk/26.1.10909125"
  else
    export ANDROID_NDK_HOME="$(find "$ANDROID_SDK_ROOT/ndk" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | head -1)"
  fi
fi

if [[ -z "${ANDROID_NDK_HOME:-}" || ! -d "$ANDROID_NDK_HOME" ]]; then
  echo "NDK not found. Run: ./scripts/setup-android-cli.sh" >&2
  exit 1
fi

export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"

# Default: arm64 only (physical devices). Set CONDUIT_ANDROID_ABIS=all for emulators.
ABIS="${CONDUIT_ANDROID_ABIS:-arm64-v8a}"

mkdir -p "$TARGET_DIR"

if command -v cargo-ndk >/dev/null 2>&1; then
  echo "Building native libs with cargo-ndk ($ABIS)..."
  cargo ndk -t "$ABIS" -o "$TARGET_DIR" build -p conduit-ffi --release
else
  echo "cargo-ndk not found; using manual cargo targets..."
  case "$ABIS" in
    all)
      TRIPLES=(aarch64-linux-android armv7-linux-androideabi x86_64-linux-android)
      DIRS=(arm64-v8a armeabi-v7a x86_64)
      ;;
    *)
      TRIPLES=(aarch64-linux-android)
      DIRS=(arm64-v8a)
      ;;
  esac
  for i in "${!TRIPLES[@]}"; do
    triple="${TRIPLES[$i]}"
    dir="${DIRS[$i]}"
    echo "Building $triple -> $dir"
    mkdir -p "$TARGET_DIR/$dir"
    cargo build -p conduit-ffi --release --target "$triple"
    cp "$ROOT/target/$triple/release/libconduit_ffi.so" "$TARGET_DIR/$dir/"
  done
fi

echo "Native libraries installed in jniLibs"
