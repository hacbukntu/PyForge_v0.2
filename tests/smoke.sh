#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../scripts/llvm-env.sh"
cargo build --release
BIN=target/release/pyforgec
$BIN check examples/hello.py
$BIN build examples/hello.py -o /tmp/pyforge_hello
OUT=$(/tmp/pyforge_hello)
EXPECTED=$'Hello from native PyForge\n42'
[[ "$OUT" == "$EXPECTED" ]] || { echo "unexpected output:"; printf '%s\n' "$OUT"; exit 1; }
$BIN build examples/functions.py -o /tmp/pyforge_functions
[[ "$(/tmp/pyforge_functions)" == $'42\n144' ]]
$BIN build examples/control_flow.py -o /tmp/pyforge_control
[[ "$(/tmp/pyforge_control)" == $'55\n5\n4\n3\n2\n1\nTrue' ]]
echo "All smoke tests passed."
