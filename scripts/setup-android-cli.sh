#!/usr/bin/env bash
# Bootstrap Android SDK/NDK + Rust targets for headless CLI builds.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-$HOME/Android/Sdk}}"
export ANDROID_HOME="$ANDROID_SDK_ROOT"

echo "Android SDK: $ANDROID_SDK_ROOT"

if [[ ! -d "$ANDROID_SDK_ROOT" ]]; then
  echo "Create SDK dir or set ANDROID_SDK_ROOT." >&2
  exit 1
fi

SDKMANAGER=""
if [[ -x "$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager" ]]; then
  SDKMANAGER="$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager"
elif [[ -x "$ANDROID_SDK_ROOT/cmdline-tools/bin/sdkmanager" ]]; then
  SDKMANAGER="$ANDROID_SDK_ROOT/cmdline-tools/bin/sdkmanager"
fi

if [[ -z "$SDKMANAGER" ]]; then
  echo "Installing Android command-line tools..."
  TMP="$(mktemp -d)"
  curl -fsSL "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip" -o "$TMP/cmdtools.zip"
  unzip -q "$TMP/cmdtools.zip" -d "$TMP"
  mkdir -p "$ANDROID_SDK_ROOT/cmdline-tools/latest"
  mv "$TMP/cmdline-tools"/* "$ANDROID_SDK_ROOT/cmdline-tools/latest/"
  rm -rf "$TMP"
  SDKMANAGER="$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager"
fi

yes | "$SDKMANAGER" --licenses >/dev/null 2>&1 || true
"$SDKMANAGER" \
  "platform-tools" \
  "platforms;android-36" \
  "build-tools;34.0.0" \
  "ndk;26.1.10909125" \
  || true

export ANDROID_NDK_HOME="$ANDROID_SDK_ROOT/ndk/26.1.10909125"
if [[ ! -d "$ANDROID_NDK_HOME" ]]; then
  # fallback: pick any installed ndk
  ANDROID_NDK_HOME="$(find "$ANDROID_SDK_ROOT/ndk" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | head -1)"
fi
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"

echo "Android NDK: $ANDROID_NDK_HOME"

rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "Installing cargo-ndk..."
  cargo install cargo-ndk --locked
fi

echo "Android CLI toolchain ready."
