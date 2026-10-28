# Arena de Cristales 3D

Juego de recolección y evasión. Debes recoger cinco cristales amarillos mientras dos
drones rojos persiguen tu posición. Cuatro rocas bloquean el movimiento. El estado,
la persecución, las colisiones, las tres vidas y la victoria se calculan en ST.

```bash
cargo build --release
export PATH="$PWD/target/release:$PATH"
examples/games/cristales_3d/run.sh
```

- `WASD` o flechas: mover una casilla.
- Cada cristal: 75 puntos.
- Contactar un dron: pierde una vida y reinicia las posiciones de peligro.
- Cinco cristales: victoria. Cero vidas: derrota.
- `R`: reiniciar. `Q`/`Escape`: salir.

Prueba automatizada:

```bash
python3 examples/games/verify_visual_games.py --game cristales
```

Las rocas visibles de `GameProfile::decorations` deben coincidir con `bloqueado` en
`src/main.st`. Si añades otra, actualiza ambos lugares y el verificador.
