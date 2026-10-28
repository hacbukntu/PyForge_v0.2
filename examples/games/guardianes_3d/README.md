# Guardianes del Núcleo 3D

Juego de defensa por turnos. Dos drones rojos avanzan hacia el núcleo verde durante
seis rondas. El jugador puede desplazarse y atacar un dron situado en una casilla
adyacente. El núcleo tiene 15 puntos de energía.

```bash
cargo build --release
export PATH="$PWD/target/release:$PATH"
examples/games/guardianes_3d/run.sh
```

- `WASD` o flechas: mover al guardián.
- `Espacio`, `Enter` o el botón **Acción**: atacar una casilla adyacente.
- Dron destruido: 50 puntos.
- Ronda superada: 100 puntos.
- Seis rondas: victoria. Energía cero: derrota.

Prueba las seis rondas, el protocolo y el reinicio sin abrir una ventana:

```bash
python3 examples/games/verify_visual_games.py --game guardianes
```

La lógica completa está en `src/main.st`. Para añadir enemigos se debe versionar el
protocolo en vez de reutilizar silenciosamente los dos campos existentes.
