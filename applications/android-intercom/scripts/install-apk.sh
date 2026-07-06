#!/usr/bin/env bash
# Install debug APK and launch on a connected device/emulator.
set -euo pipefail

APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APK="$APP_DIR/android/app/build/outputs/apk/debug/app-debug.apk"
PKG="com.conduit.intercom"

if [[ ! -f "$APK" ]]; then
  echo "APK not found. Run: npm run android:build" >&2
  exit 1
fi

if ! command -v adb >/dev/null 2>&1; then
  echo "adb not found. Install platform-tools." >&2
  exit 1
fi

DEVICE="$(adb devices | awk 'NR>1 && $2=="device" {print $1; exit}')"
if [[ -z "$DEVICE" ]]; then
  echo "No Android device/emulator connected." >&2
  adb devices
  exit 1
fi

echo "Installing on $DEVICE..."
adb install -r "$APK"
adb shell am start -n "$PKG/.MainActivity"
echo "Launched Conduit Intercom."
