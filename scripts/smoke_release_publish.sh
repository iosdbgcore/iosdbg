#!/usr/bin/env bash

set -euo pipefail

verify_assets() {
  local root="$1"
  local x64_file
  local arm64_file

  x64_file=$(ls "${root}/x86_64-apple-darwin"/rust-lldb-visual-debugger-*-x86_64-apple-darwin.zip 2>/dev/null | head -n 1 || true)
  arm64_file=$(ls "${root}/aarch64-apple-darwin"/rust-lldb-visual-debugger-*-aarch64-apple-darwin.zip 2>/dev/null | head -n 1 || true)

  [[ -n "${x64_file}" && -n "${arm64_file}" ]]
}

workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT

mkdir -p "${workdir}/x86_64-apple-darwin" "${workdir}/aarch64-apple-darwin"
touch "${workdir}/x86_64-apple-darwin/rust-lldb-visual-debugger-0.1.0-x86_64-apple-darwin.zip"

if verify_assets "$workdir"; then
  echo "Smoke validation failed: expected missing arm64 asset to fail" >&2
  exit 1
fi

touch "${workdir}/aarch64-apple-darwin/rust-lldb-visual-debugger-0.1.0-aarch64-apple-darwin.zip"
if ! verify_assets "$workdir"; then
  echo "Smoke validation failed: expected both assets present to pass" >&2
  exit 1
fi

echo "Release publish smoke checks passed."
