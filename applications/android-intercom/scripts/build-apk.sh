#!/usr/bin/env bash
# Headless build: web → native → Gradle APK (no Android Studio).
set -euo pipefail

APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$APP_DIR/../.." && pwd)"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-$HOME/Android/Sdk}}"
export ANDROID_HOME="$ANDROID_SDK_ROOT"

echo "==> Sync Capacitor web assets"
cd "$APP_DIR"
npm run build
node scripts/ensure-android.mjs
npx cap sync android

echo "==> Build Rust native library"
bash "$ROOT/scripts/build-android-ffi.sh"

echo "==> Write local.properties"
cat > "$APP_DIR/android/local.properties" <<EOF
sdk.dir=$ANDROID_SDK_ROOT
EOF

echo "==> Gradle assembleDebug"
cd "$APP_DIR/android"
chmod +x gradlew
./gradlew assembleDebug --no-daemon

APK="$APP_DIR/android/app/build/outputs/apk/debug/app-debug.apk"
echo ""
echo "APK built: $APK"
