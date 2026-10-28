# Cuatro juegos 3D con lógica ST

Los cuatro proyectos usan el mismo recorrido:

```text
teclado/ventana Rust → comando validado → ejecutable ST → estado validado
                                         ↓
                             escena con cubos sólidos y HUD
```

| Juego | Perfil | Mecánica completa |
| --- | --- | --- |
| [Portal Runner](portal_runner_visual/README.md) | `portales` | Cinco objetivos, puntuación y victoria |
| [Arena de Cristales](cristales_3d/README.md) | `cristales` | Recolección, persecución, obstáculos y vidas |
| [Laberinto Neón](laberinto_3d/README.md) | `laberinto` | Muros, límite de turnos y ruta de salida |
| [Guardianes del Núcleo](guardianes_3d/README.md) | `guardianes` | Combate, energía, rondas y victoria/derrota |

## Ejecutar todos los juegos desde una instalación

Entra en el directorio de uno, compila la lógica y selecciona su perfil:

```bash
pyforgec build src/main.st -o juego
pyforge motor3d --juego cristales ./juego
```

Los perfiles válidos son `portales`, `cristales`, `laberinto` y `guardianes`. Los
scripts `run.sh` o `build_and_run.sh` automatizan el mismo proceso desde las fuentes.
Guardan el ejecutable temporal en `$XDG_CACHE_HOME/pyforge/games` o, si esa variable
no existe, en `~/.cache/pyforge/games`; así también funcionan desde la copia de solo
lectura instalada en `/usr/share/pyforge/examples`.

## Protocolo común

El ejecutable ST recibe una línea de comando. Después de `init`, recibe catorce
enteros: jugador, objetivo, dos enemigos, nivel, puntos, vidas, turnos y las banderas
`terminado`/`victoria`. Devuelve `OK`, esos catorce campos y un mensaje. Las banderas
solo admiten `0` o `1`; coordenadas, contadores y mensajes tienen límites.

Cada acción termina el proceso ST. Esto evita el crecimiento histórico del runtime
durante un bucle persistente, aunque crear un proceso por turno no sirve todavía para
un juego de acción en tiempo real. El siguiente paso del motor es una ABI persistente
con propiedad y liberación comprobadas.

## Verificación

```bash
cargo build --release
python3 examples/games/verify_visual_games.py --compiler target/release/pyforgec
cargo test --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --all-features --all-targets --offline
```

El verificador recompila siempre las fuentes en un temporal. No reutiliza `build/`,
no abre ventanas y completa una condición significativa de cada juego.

## Alcance gráfico

El viewport proyecta cubos con caras sólidas, sombreado por cara, contornos y colores
por material. Usa `egui/wgpu`, cámara orbital y una rejilla 3D. Es una presentación
gráfica completa para estos cuatro juegos de cuadrícula. No utiliza todavía modelos
glTF animados, mapas de sombras, IBL ni el pipeline PBR de la demostración Vulkan;
esas capacidades continúan como etapas distintas del motor.
