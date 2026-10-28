#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
COMPILER="${PYFORGEC:-$ROOT/target/release/pyforgec}"
exec python3 "$HERE/../verify_visual_games.py" --compiler "$COMPILER" --game portales
