# Escenas 3D controladas desde ST

`escena_profesional.st` es un proyecto mínimo y ejecutable para
`pyforge-editor3d`. Toda la escena se declara con sintaxis ST; el editor no evalúa
JSON escrito dentro del programa ni ejecuta Python.

## Abrir el ejemplo

Desde una instalación de PyForge:

```bash
pyforge motor3d --editor
```

En la pestaña **ST Script de escena**:

1. Escribe `examples/editor3d/escena_profesional.st` en **Archivo .st**.
2. Comprueba que **Compilador** contiene `pyforgec` o su ruta instalada.
3. Pulsa **Abrir** y después **Aplicar ST**.
4. Selecciona una entidad en la jerarquía y muévela con el gizmo `W`.
5. Pulsa **Exportar visual** para guardar la posición nueva en el `.st`.

Durante el desarrollo del repositorio también puede iniciarse así:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --features editor --bin pyforge-editor3d
```

La integración puede verificarse sin ventana:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --example validar_escena_st -- \
  target/debug/pyforgec examples/editor3d/escena_profesional.st
```

## API del módulo `editor3d`

Una escena comienza con su nombre y el número exacto de entidades:

```st
importar "editor3d";

editor3d.iniciar("Mi nivel"; 1);
editor3d.entidad("Jugador"; "cubo"; "jugador");
editor3d.posicion(0.0; 0.5; 0.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(1.0; 1.0; 1.0);
editor3d.terminar();
```

Cada entidad necesita las cuatro llamadas, en ese orden:

| Función ST | Contenido |
| --- | --- |
| `entidad(nombre; malla; material)` | Identidad y recursos visuales |
| `posicion(x; y; z)` | Traslación mundial |
| `rotacion(x; y; z; w)` | Cuaternión XYZW |
| `escala(x; y; z)` | Tamaño; ningún componente puede ser cero |

El diseño dividido respeta el ABI actual de ST, que admite hasta seis parámetros
por función. Una cadena vacía en malla o material representa un recurso ausente.

## Qué valida el editor

El botón **Aplicar ST** guarda el texto, ejecuta `pyforgec build` sin shell y
lanza el binario temporal. El puente impone 90 segundos para compilar, 15 segundos
para ejecutar, 4 MiB de salida y 10 000 entidades. También comprueba la cabecera,
el final, cada campo, números finitos, escala no nula, cuaternión válido y nombres
acotados. Una salida parcial no modifica la escena abierta.

La recarga automática está desactivada inicialmente. Al activarla, cualquier
cambio guardado fuera del editor vuelve a compilar el archivo; por eso debe usarse
solo con código ST de confianza. Los scripts conservan las capacidades normales
que el usuario les otorgue mediante módulos de archivos, procesos o red.

## Controles

La pestaña **Ayuda** se muestra inicialmente y describe cada panel. **Jerarquía**
es la lista izquierda, **Viewport 3D** el lienzo central, **Inspector** las
propiedades derechas y **Consola / Script ST / Ayuda** el panel inferior. Los
botones superiores incluyen su nombre junto al símbolo para que no dependan del
icono ni de una fuente especial.

| Acción | Control |
| --- | --- |
| Seleccionar | Clic en el objeto o en la jerarquía |
| Enfocar selección | `F` o doble clic en la jerarquía |
| Mover | `W` y arrastrar un eje |
| Rotar | `E` y arrastrar un eje |
| Escalar | `R` y arrastrar un eje |
| Orbitar cámara | Botón derecho |
| Desplazar cámara | Botón central |
| Zoom | Rueda |
| Duplicar | `Ctrl+D` |
| Eliminar | `Supr` |
| Deshacer/Rehacer | `Ctrl+Z` / `Ctrl+Y` |

El viewport actual representa entidades como volúmenes sólidos y utiliza el nombre
de material para su paleta. `luz` y `camara` son marcadores editables de escena;
todavía no conectan iluminación o cámaras del renderer GPU. El pipeline PBR, las
mallas glTF y sus animaciones permanecen separados del viewport de `egui`.
