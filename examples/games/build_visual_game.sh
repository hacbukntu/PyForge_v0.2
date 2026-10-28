#!/usr/bin/env bash
set -euo pipefail

# Compila la lógica ST y abre el ejecutor gráfico común con el perfil solicitado.
if [ "$#" -ne 2 ]; then
    echo "USO: build_visual_game.sh <carpeta-juego> <perfil>" >&2
    exit 2
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
GAME_DIR="$1"
PROFILE="$2"

# Los ejemplos del paquete Debian viven en /usr/share, que es de solo lectura para
# usuarios normales. Los binarios generados se guardan en la caché del usuario para
# que estos mismos lanzadores funcionen desde el repositorio y desde una instalación.
if [ -n "${XDG_CACHE_HOME:-}" ]; then
    CACHE_BASE="$XDG_CACHE_HOME"
elif [ -n "${HOME:-}" ]; then
    CACHE_BASE="$HOME/.cache"
else
    CACHE_BASE="${TMPDIR:-/tmp}"
fi
BUILD_DIR="$CACHE_BASE/pyforge/games/$PROFILE"
LOGIC_BIN="$BUILD_DIR/game_logic"

command -v pyforgec >/dev/null 2>&1 || {
    echo "ERROR: pyforgec no está en PATH." >&2
    exit 1
}

mkdir -p "$BUILD_DIR"
pyforgec check "$GAME_DIR/src/main.st"
pyforgec build "$GAME_DIR/src/main.st" -o "$LOGIC_BIN"
chmod +x "$LOGIC_BIN"

if command -v pyforge-game3d >/dev/null 2>&1; then
    exec pyforge-game3d --perfil "$PROFILE" "$LOGIC_BIN"
fi

command -v cargo >/dev/null 2>&1 || {
    echo "ERROR: instala el paquete PyForge actualizado o Cargo/Rust." >&2
    exit 1
}
cargo build --manifest-path "$ROOT/crates/pyforge_engine3d/Cargo.toml" \
    --features editor --bin pyforge-game3d --release --locked
exec "$ROOT/crates/pyforge_engine3d/target/release/pyforge-game3d" \
    --perfil "$PROFILE" "$LOGIC_BIN"
