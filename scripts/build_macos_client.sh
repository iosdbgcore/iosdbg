#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Build and package a macOS app bundle for rust-lldb-visual-debugger.

Usage:
  scripts/build_macos_client.sh [options]

Options:
  --target <triple>       Build a single target (x86_64-apple-darwin | aarch64-apple-darwin)
  --universal             Build both macOS targets and merge with lipo
  --profile <name>        Cargo profile to use (default: release)
  --features <list>       Cargo feature list, e.g. "real-lldb"
  --bundle-id <id>        macOS bundle identifier (default: com.iosdbg.visualdebugger)
  --app-name <name>       App display name (default: Rust LLDB Visual Debugger)
  -h, --help              Show this help message

Examples:
  scripts/build_macos_client.sh --target aarch64-apple-darwin
  scripts/build_macos_client.sh --universal --features real-lldb
EOF
}

PACKAGE_NAME="$(awk -F'"' '/^\[package\]/{in_package=1; next} in_package && /^name =/{print $2; exit}' Cargo.toml)"
VERSION="$(awk -F'"' '/^\[package\]/{in_package=1; next} in_package && /^version =/{print $2; exit}' Cargo.toml)"

if [[ -z "${PACKAGE_NAME}" || -z "${VERSION}" ]]; then
  echo "Unable to parse package name/version from Cargo.toml" >&2
  exit 1
fi

APP_NAME="Rust LLDB Visual Debugger"
BUNDLE_ID="com.iosdbg.visualdebugger"
PROFILE="release"
FEATURES=""
TARGET=""
UNIVERSAL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET="${2:-}"
      shift 2
      ;;
    --universal)
      UNIVERSAL=1
      shift
      ;;
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --features)
      FEATURES="${2:-}"
      shift 2
      ;;
    --bundle-id)
      BUNDLE_ID="${2:-}"
      shift 2
      ;;
    --app-name)
      APP_NAME="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script must run on macOS (Darwin)." >&2
  exit 1
fi

if [[ "${UNIVERSAL}" -eq 1 && -n "${TARGET}" ]]; then
  echo "Use either --target or --universal, not both." >&2
  exit 1
fi

if [[ "${UNIVERSAL}" -eq 0 && -z "${TARGET}" ]]; then
  TARGET="$(rustc -vV | awk '/host:/{print $2}')"
fi

case "${TARGET:-}" in
  x86_64-apple-darwin|aarch64-apple-darwin|"")
    ;;
  *)
    echo "Unsupported target: ${TARGET}" >&2
    exit 1
    ;;
esac

build_target() {
  local target="$1"
  rustup target add "${target}" >/dev/null

  local cmd=(cargo build --target "${target}" --profile "${PROFILE}")
  if [[ -n "${FEATURES}" ]]; then
    cmd+=(--features "${FEATURES}")
  fi

  echo "Building ${target} ..."
  "${cmd[@]}"
}

mkdir -p dist/macos
APP_DIR="dist/macos/${APP_NAME}.app"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

if [[ "${UNIVERSAL}" -eq 1 ]]; then
  build_target "x86_64-apple-darwin"
  build_target "aarch64-apple-darwin"

  lipo -create \
    "target/x86_64-apple-darwin/${PROFILE}/${PACKAGE_NAME}" \
    "target/aarch64-apple-darwin/${PROFILE}/${PACKAGE_NAME}" \
    -output "${APP_DIR}/Contents/MacOS/${PACKAGE_NAME}"
  ARTIFACT_TARGET="universal2"
else
  build_target "${TARGET}"
  cp "target/${TARGET}/${PROFILE}/${PACKAGE_NAME}" "${APP_DIR}/Contents/MacOS/${PACKAGE_NAME}"
  ARTIFACT_TARGET="${TARGET}"
fi

chmod +x "${APP_DIR}/Contents/MacOS/${PACKAGE_NAME}"

cat > "${APP_DIR}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>${PACKAGE_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>${BUNDLE_ID}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

ZIP_PATH="dist/macos/${PACKAGE_NAME}-${VERSION}-${ARTIFACT_TARGET}.zip"
rm -f "${ZIP_PATH}"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "${APP_DIR}" "${ZIP_PATH}"

echo "App bundle: ${APP_DIR}"
echo "Zip artifact: ${ZIP_PATH}"
