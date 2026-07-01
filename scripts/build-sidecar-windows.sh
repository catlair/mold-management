#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$PROJECT_DIR/src-tauri/binaries"

echo "=> Compiling sidecar binary for Windows with bun..."
bun build "$PROJECT_DIR/sidecar/server.js" \
  --compile \
  --compile-executable-path "$PROJECT_DIR/bun-windows-x64/bun.exe" \
  --outfile "$PROJECT_DIR/src-tauri/binaries/sidecar-server-x86_64-pc-windows-msvc.exe"

echo "=> Sidecar binary built successfully!"
ls -lh "$PROJECT_DIR/src-tauri/binaries/sidecar-server-x86_64-pc-windows-msvc.exe"