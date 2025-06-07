#!/bin/sh

# Exit immediately if a command exits with a non-zero status
set -e

# Change to the script's directory to ensure relative paths work correctly
cd "$(dirname "$0")"

# Compile using release profile and place artifacts in a temp directory
cargo build --release \
  --target-dir=/tmp/cc-build-shell-rust \
  --manifest-path Cargo.toml

# Execute the compiled binary with any arguments passed to this script
exec /tmp/cc-build-shell-rust/release/codecrafters-shell-rust "$@"
