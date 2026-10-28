# Portal Runner 3D

Juego completo de exploración por turnos. La lógica `.st` controla movimiento,
cinco portales, puntuación, límites, reinicio y victoria. El ejecutor nativo muestra
el jugador cian, el portal púrpura, torres sólidas, cámara orbital y HUD.

```bash
cargo build --release
export PATH="$PWD/target/release:$PATH"
examples/games/portal_runner_visual/build_and_run.sh
```

Usa `WASD` o flechas para moverte. Cada portal suma 100 puntos y coloca el siguiente
en otra zona. La partida termina al alcanzar el quinto portal. `R` reinicia y
`Q`/`Escape` cierra.

Prueba lógica sin ventana:

```bash
examples/games/portal_runner_visual/test_logic.sh
```

Edita posiciones y puntuación en `src/main.st`. Conserva las 16 líneas del protocolo
común descrito en [`JUEGOS_3D.md`](../JUEGOS_3D.md).
