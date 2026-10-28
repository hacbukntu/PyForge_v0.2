#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/llvm-env.sh"
cargo build --release
mkdir -p dist
cp target/release/pyforge target/release/pyforgec dist/
echo "Binaries copied to dist/."
