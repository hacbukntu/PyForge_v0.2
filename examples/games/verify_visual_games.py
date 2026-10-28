#!/usr/bin/env python3
"""Compila y completa las reglas principales de los cuatro juegos 3D sin abrir GUI."""

import argparse
from collections import deque
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
GAMES = {
    "portales": ROOT / "examples/games/portal_runner_visual/src/main.st",
    "cristales": ROOT / "examples/games/cristales_3d/src/main.st",
    "laberinto": ROOT / "examples/games/laberinto_3d/src/main.st",
    "guardianes": ROOT / "examples/games/guardianes_3d/src/main.st",
}
FIELDS = (
    "player_x", "player_z", "target_x", "target_z",
    "enemy_1_x", "enemy_1_z", "enemy_2_x", "enemy_2_z",
    "level", "score", "lives", "turns", "ended", "won",
)


def transition(binary: Path, command: str, state=None):
    """Ejecuta una transición y valida el protocolo común de 16 líneas."""
    values = [command]
    if state is not None:
        values.extend(str(state[field]) for field in FIELDS)
    result = subprocess.run(
        [str(binary)], input="\n".join(values) + "\n", text=True,
        capture_output=True, timeout=10,
    )
    if result.returncode:
        raise RuntimeError(result.stderr or result.stdout)
    lines = result.stdout.splitlines()
    if len(lines) < 16 or lines[0] != "OK":
        raise AssertionError(f"protocolo inválido: {lines!r}")
    parsed = {name: int(lines[index + 1]) for index, name in enumerate(FIELDS)}
    if parsed["ended"] not in (0, 1) or parsed["won"] not in (0, 1):
        raise AssertionError("las banderas deben usar 0 o 1")
    parsed["message"] = "\n".join(lines[15:])
    return parsed


def compile_game(compiler: Path, source: Path, output: Path):
    """Compila desde la fuente actual; nunca reutiliza artefactos del ejemplo."""
    subprocess.run([str(compiler), "check", str(source)], check=True, capture_output=True)
    subprocess.run(
        [str(compiler), "build", str(source), "-o", str(output)],
        check=True, capture_output=True,
    )


def maze_path():
    """Calcula una solución independiente para probar colisiones y victoria."""
    blocked = set()
    blocked.update((x, -4) for x in range(-4, 3))
    blocked.update((2, z) for z in range(-3, 1))
    blocked.update((x, -1) for x in range(-4, -1))
    blocked.update((-1, z) for z in range(0, 3))
    blocked.update((x, 2) for x in range(0, 5))
    blocked.update((4, z) for z in range(3, 6))
    blocked.update((x, 4) for x in range(-5, 2))
    start, goal = (-6, -6), (6, 6)
    queue = deque([(start, [])])
    visited = {start}
    for_current = ((0, 1, "w"), (0, -1, "s"), (-1, 0, "a"), (1, 0, "d"))
    while queue:
        position, path = queue.popleft()
        if position == goal:
            return path
        for dx, dz, command in for_current:
            following = (position[0] + dx, position[1] + dz)
            if -7 <= following[0] <= 7 and -7 <= following[1] <= 7 and following not in blocked and following not in visited:
                visited.add(following)
                queue.append((following, path + [command]))
    raise AssertionError("el laberinto no tiene solución")


def verify(name: str, binary: Path):
    """Completa el objetivo principal de un juego y comprueba reinicio."""
    state = transition(binary, "init")
    assert not state["ended"] and not state["won"]
    if name in {"portales", "cristales"}:
        rocks = {(-3, 2), (2, -3), (4, 3), (-5, -2)} if name == "cristales" else set()
        # Sigue siempre una ruta Manhattan válida. En Cristales la estrategia
        # puede perder vidas, pero debe poder recoger los cinco antes de agotarlas.
        for _ in range(300):
            if state["ended"]:
                break
            options = []
            for command, dx, dz in (("d", 1, 0), ("a", -1, 0), ("w", 0, 1), ("s", 0, -1)):
                following = (state["player_x"] + dx, state["player_z"] + dz)
                if following in rocks or not (-10 <= following[0] <= 10 and -10 <= following[1] <= 10):
                    continue
                distance = abs(following[0] - state["target_x"]) + abs(following[1] - state["target_z"])
                options.append((distance, command))
            state = transition(binary, min(options)[1], state)
    else:
        commands = maze_path() if name == "laberinto" else ["f"] * 48
        for command in commands:
            state = transition(binary, command, state)
    assert state["ended"] == 1 and state["won"] == 1, state
    restarted = transition(binary, "r", state)
    assert restarted["ended"] == 0 and restarted["won"] == 0
    print(f"{name}: partida completa, victoria y reinicio correctos")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, default=ROOT / "target/release/pyforgec")
    parser.add_argument("--game", choices=sorted(GAMES))
    args = parser.parse_args()
    compiler = args.compiler.resolve()
    selected = [args.game] if args.game else list(GAMES)
    with tempfile.TemporaryDirectory(prefix="pyforge-juegos3d-") as temporary:
        root = Path(temporary)
        for name in selected:
            binary = root / name
            compile_game(compiler, GAMES[name], binary)
            verify(name, binary)
    print(f"Juegos 3D verificados: {len(selected)}/{len(selected)}")


if __name__ == "__main__":
    main()
