#!/usr/bin/env bash
# Construye el VSIX con vocabulario actual; instala únicamente cuando se ejecuta este script.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if ! command -v code >/dev/null 2>&1; then
  echo 'No se encontró VS Code en PATH.' >&2
  exit 1
fi
python3 "$ROOT/scripts/build-vscode-extension.py"
code --install-extension "$ROOT/dist/extensions/pyforge-st-0.1.0.vsix" --force
