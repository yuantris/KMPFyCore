#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
command -v cargo-ndk >/dev/null 2>&1 || { echo "Install cargo-ndk: cargo install cargo-ndk"; exit 1; }
OUT_DIR="$ROOT_DIR/bindings/android/src/main/jniLibs"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o "$OUT_DIR" build -p core-rs-android --release
echo "Generated: $OUT_DIR"
