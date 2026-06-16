#!/bin/bash

set -euo pipefail

echo "🚀 Starting Windows build process..."

if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Error: cargo is not installed."
    exit 1
fi

OS_TYPE=$(uname -s)
TARGET="x86_64-pc-windows-gnu"
OUTPUT_DIR="./target/release_windows"
EXE_NAME="BlinkReminder.exe"
IS_CROSS_COMPILE=false

if [[ "$OS_TYPE" == "Darwin"* ]] || [[ "$OS_TYPE" == "Linux"* ]]; then
    IS_CROSS_COMPILE=true
    echo "ℹ️ Detected $OS_TYPE. Setting up cross-compilation for Windows..."

    if ! rustup target list | grep -q "$TARGET (installed)"; then
        echo "📦 Installing $TARGET target..."
        rustup target add "$TARGET"
    fi

    if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        if [[ "$OS_TYPE" == "Darwin"* ]]; then
            echo "❌ Error: mingw-w64 is not installed. Please install it using: brew install mingw-w64"
        else
            echo "❌ Error: mingw-w64 is not installed. Please install it using your package manager (e.g. apt install mingw-w64)"
        fi
        exit 1
    fi
fi

echo "🔨 Building release version..."
if [[ "$IS_CROSS_COMPILE" == true ]]; then
    CARGO_TARGET_DIR=target CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
    cargo build --release --target "$TARGET"
    EXE_PATH="target/$TARGET/release/blink-reminder-native.exe"
else
    CARGO_TARGET_DIR=target cargo build --release
    EXE_PATH="target/release/blink-reminder-native.exe"
fi

if [[ ! -f "$EXE_PATH" ]]; then
    echo "❌ Error: Build failed, executable not found at $EXE_PATH"
    exit 1
fi

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo "📦 Packaging files..."
cp "$EXE_PATH" "$OUTPUT_DIR/$EXE_NAME"

echo "✅ Build completed successfully!"
echo "📁 Output directory: $OUTPUT_DIR"
echo "   - $OUTPUT_DIR/$EXE_NAME"
