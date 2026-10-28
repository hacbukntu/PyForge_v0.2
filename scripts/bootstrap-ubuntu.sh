#!/usr/bin/env bash
set -euo pipefail
sudo apt update
sudo apt install -y curl build-essential binutils llvm-21-dev libllvm21 python3 python3-venv python3-tk python3-cryptography
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi
rustc --version
cargo --version
as --version | head -n1
ld --version | head -n1
llvm-config-21 --version
echo "Bootstrap complete."
