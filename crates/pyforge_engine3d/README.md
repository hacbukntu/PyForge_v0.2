# PyForge Engine 3D

Backend gráfico opcional de ST. El crate vive fuera del workspace principal para
que compilar programas de consola no requiera una GPU ni bibliotecas gráficas.

## Componentes comprobables

- `renderer.rs`: ventana `winit`, selección Vulkan/OpenGL mediante `wgpu`, superficie
  sRGB, profundidad, textura RGBA, shader WGSL e instanciación de tres geometrías.
- `assets.rs`: importación acotada de OBJ, glTF 2.0, PNG, JPEG y WGSL en CPU.
- `animation.rs`: interpolación de poses y matrices de skinning en CPU.
- `audio.rs`: carga WAV, mezcla estéreo espacial y salida PCM física opcional mediante
  `pw-cat` (PipeWire) o `aplay` (ALSA).
- `scene.rs`: escena JSON versionada, validada y guardada por reemplazo atómico.
- `st_scene.rs`: compilación acotada de escenas `.st`, protocolo validado y
  exportación del documento visual de vuelta a código ST.
- `editor_viewport.rs`: cámara orbital, rejilla, caras sólidas sombreadas, selección
  y gizmos manipulables para mover, rotar y escalar cajas dentro de `egui`.
- `editor.rs`: interfaz con menús e iconos, jerarquía buscable, inspector completo,
  historial, consola, editor ST, recarga opcional y paneles redimensionables.
- `game_main.rs`: ejecutor común de cuatro juegos ST con perfiles visuales, HUD,
  límites de protocolo y estados de victoria/derrota.

## Compilar y probar

```bash
cargo test --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --all-features --all-targets --offline
```

Abrir la demostración GPU:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml -- --vulkan
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml -- --opengl
```

Abrir el editor:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --features editor --bin pyforge-editor3d
```

En el viewport, el botón derecho orbita, el central desplaza y la rueda cambia la
distancia. `W`, `E` y `R` eligen los gizmos de traslación, rotación y escala; `F`
centra la selección. `Ctrl+D`, `Supr`, `Ctrl+Z` y `Ctrl+Y` duplican, eliminan y
recorren el historial.

Los paneles y botones tienen etiquetas en español y la pestaña **Ayuda** explica
su función desde el inicio. `eframe` usa Glow para que el texto se renderice de
forma legible en equipos Mesa donde el atlas WGPU/Vulkan aparecía como bloques.
El renderer 3D de `renderer.rs` sigue usando WGPU con Vulkan/OpenGL; son rutas
separadas.

## Escena dirigida por ST

El módulo estándar `editor3d` permite definir la escena sin escribir Rust ni JSON:

```st
importar "editor3d";

editor3d.iniciar("Nivel"; 1);
editor3d.entidad("Jugador"; "cubo"; "jugador");
editor3d.posicion(0.0; 0.5; 0.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(1.0; 1.0; 1.0);
editor3d.terminar();
```

La pestaña **ST Script de escena** abre, guarda, compila y aplica el archivo sin
bloquear la interfaz. **Exportar visual** convierte los cambios del inspector y
los gizmos en esas mismas llamadas ST. La ejecución usa `Command` directamente,
sin shell, y limita tiempo, salida, tamaño de fuente y número de entidades. La guía
completa y el ejemplo están en `examples/editor3d/README.md`.

La recarga automática es opcional y permanece desactivada al iniciar. Un script ST
es código ejecutable y conserva las capacidades de los módulos que importe; solo
deben aplicarse fuentes de confianza.

## Ejemplos del crate

Inspeccionar un recurso sin abrir una ventana:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --example inspeccionar_recurso -- modelo.gltf
```

Reproducir un WAV espacial. `auto` prefiere PipeWire y usa ALSA como alternativa:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --example audio_espacial -- sonido.wav auto
```

La salida física depende de que `pw-cat` o `aplay` estén instalados y de que exista
un servidor/dispositivo de audio accesible. El código no abre una shell ni concatena
argumentos proporcionados por el archivo.

Los cuatro juegos ST reproducibles están documentados en
`examples/games/JUEGOS_3D.md`.

Validar una escena ST sin abrir la interfaz:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --example validar_escena_st -- \
  target/debug/pyforgec examples/editor3d/escena_profesional.st
```

## Frontera actual

El renderer GPU y el viewport del editor son dos rutas distintas: el viewport actual
dibuja cajas sólidas con sombreado simple mediante `egui`, no incrusta la textura
final del pipeline PBR.
El PBR del shader es una aproximación local con textura base, metalicidad y rugosidad
constantes. Todavía no hay mapas de sombras, iluminación basada en imágenes, materiales
glTF conectados, varias mallas importadas en el grafo visible ni skinning enviado a GPU.

Tampoco existe streaming asíncrono de mundos. Los importadores leen el recurso completo
dentro de límites configurados. La salida de audio es real, pero usa los clientes del
sistema en procesos separados; aún no hay mezclador continuo de voces, recuperación de
dispositivo ni API de audio expuesta directamente a `.st`.
