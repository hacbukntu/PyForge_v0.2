# Laberinto Neón 3D

Juego de navegación con muros sólidos, colisiones y puntuación por eficiencia. Se
inicia en `(-6, -6)` y la salida está en `(6, 6)`. Hay 120 turnos para encontrarla.

```bash
cargo build --release
export PATH="$PWD/target/release:$PATH"
examples/games/laberinto_3d/run.sh
```

Muévete con `WASD` o flechas. Un intento contra un muro consume un turno, conserva
la posición y muestra un diagnóstico. La puntuación comienza en 1000, descuenta cinco
puntos por turno y nunca baja de 100 al ganar.

El verificador construye una ruta con BFS independiente y completa el laberinto:

```bash
python3 examples/games/verify_visual_games.py --game laberinto
```

Al modificar el mapa, conserva sincronizadas `bloqueado` en `src/main.st`,
`maze_walls` en `game_main.rs` y `maze_path` en el verificador.
