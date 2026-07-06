#!/usr/bin/env bash
# Cross-compile conduit-ffi for Android ABIs.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT/applications/android-intercom/plugins/conduit/android/src/main/jniLibs"
NDK="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"

if [[ -z "$NDK" ]]; then
  echo "Set ANDROID_NDK_HOME to your Android NDK path." >&2
  exit 1
fi

ABIS=("aarch64-linux-android" "armv7-linux-androideabi" "x86_64-linux-android")
ABI_DIRS=("arm64-v8a" "armeabi-v7a" "x86_64")

mkdir -p "$TARGET_DIR"

for i in "${!ABIS[@]}"; do
  triple="${ABIS[$i]}"
  dir="${ABI_DIRS[$i]}"
  echo "Building for $triple -> $dir"
  mkdir -p "$TARGET_DIR/$dir"
  cargo build -p conduit-ffi --release --target "$triple"
  cp "$ROOT/target/$triple/release/libconduit_ffi.so" "$TARGET_DIR/$dir/"
done

echo "Copied libconduit_ffi.so into jniLibs"
