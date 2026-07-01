#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$PROJECT_DIR/src-tauri/binaries"

echo "=> Compiling sidecar binary with bun..."
bun build "$PROJECT_DIR/sidecar/server.js" \
  --compile \
  --outfile "$PROJECT_DIR/src-tauri/binaries/sidecar-server-aarch64-apple-darwin"

echo "=> Sidecar binary built successfully!"
ls -lh "$PROJECT_DIR/src-tauri/binaries/sidecar-server-aarch64-apple-darwin"
