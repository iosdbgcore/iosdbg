#!/usr/bin/env bash

set -euo pipefail

WORKFLOW_FILE=".github/workflows/build-macos-client.yml"

if [[ ! -f "$WORKFLOW_FILE" ]]; then
  echo "Workflow file missing: $WORKFLOW_FILE" >&2
  exit 1
fi

require_pattern() {
  local pattern="$1"
  local message="$2"
  if ! grep -Eq "$pattern" "$WORKFLOW_FILE"; then
    echo "Validation failed: ${message}" >&2
    exit 1
  fi
}

require_pattern '^[[:space:]]*release:' "release trigger must exist"
require_pattern '^[[:space:]]*pull_request:' "pull_request trigger must exist"
require_pattern 'x86_64-apple-darwin' "matrix must include x86_64-apple-darwin"
require_pattern 'aarch64-apple-darwin' "matrix must include aarch64-apple-darwin"
require_pattern 'rust-lldb-visual-debugger-\*-x86_64-apple-darwin\.zip' "x64 asset naming pattern missing"
require_pattern 'rust-lldb-visual-debugger-\*-aarch64-apple-darwin\.zip' "arm64 asset naming pattern missing"
require_pattern 'contents:[[:space:]]*write' "publish job must declare contents: write"
require_pattern 'Verify release assets exist' "publish job must verify both assets before upload"

echo "Workflow validation passed: triggers, matrix targets, naming, and release safeguards are present."
