#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/llvm-env.sh"
cargo build --release
./target/release/pyforgec emit-ir examples/functions.py
./target/release/pyforgec emit-asm examples/functions.py -o /tmp/functions.s
./target/release/pyforgec run examples/functions.py
