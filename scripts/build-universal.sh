#!/bin/bash
set -euo pipefail

# Build universal binary for macOS (aarch64 + x86_64)
# Requires both targets installed:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin

echo "Building for aarch64-apple-darwin..."
cargo build --release --target aarch64-apple-darwin

echo "Building for x86_64-apple-darwin..."
cargo build --release --target x86_64-apple-darwin

echo "Creating universal binary..."
mkdir -p target/universal-apple-darwin/release

lipo -create \
    target/aarch64-apple-darwin/release/skyscraper \
    target/x86_64-apple-darwin/release/skyscraper \
    -output target/universal-apple-darwin/release/skyscraper

echo "Universal binary created at target/universal-apple-darwin/release/skyscraper"

# Create tarball for Homebrew
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
TARBALL="skyscraper-${VERSION}-universal-apple-darwin.tar.gz"

cd target/universal-apple-darwin/release
tar czf "../../../${TARBALL}" skyscraper
cd ../../..

echo "Tarball created: ${TARBALL}"
echo "SHA256: $(shasum -a 256 "${TARBALL}" | cut -d' ' -f1)"
