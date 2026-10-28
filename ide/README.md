# ST IDE

Editor local independiente del compilador. Usa Python 3 y un navegador moderno; no requiere paquetes de Python, Node ni conexión a Internet.

## Iniciar

Desde la raíz del repositorio:

```bash
cargo build --release
python3 ide/server.py
```

Abre **http://127.0.0.1:8765** en el navegador. Para abrir otro proyecto:

```bash
python3 ide/server.py /ruta/a/calculator
```

Opciones: `--port 9000` cambia el puerto; `--compiler /ruta/a/pyforgec` selecciona otro compilador. Ctrl+C cierra el servidor y detiene la ejecución activa.

## Apariencia de terminal

La interfaz usa tipografía monoespaciada, barras compactas, bordes rectos y colores discretos inspirados en Neovim. Conserva el explorador, el icono ST, las sugerencias y la salida redimensionable. Es una apariencia visual: la edición sigue siendo directa, sin modos Normal/Insertar ni comandos de Vim.

Para personalizarla, cambia las variables al principio de `style.css`: `--bg`, `--panel`, `--border`, `--text`, `--muted`, `--green` y `--font`. Los bloques CSS explican cada panel. Mantén iguales la fuente, el interlineado y el relleno de `#editor` y `#highlight` para que el texto y los colores coincidan.

`index.html` documenta la estructura y sus conexiones con JavaScript. `app.js` comenta las funciones y los grupos de eventos: carga de archivos, guardado, colores, sugerencias, ejecución y consultas periódicas. Tras modificar estos archivos, recarga el navegador; no hace falta recompilar PyForge para cambiar la apariencia.

## Funciones

- Explorador de carpetas y archivos; icono ST propio para `.st`.
- Editor con números de línea, colores para palabras reservadas, tipos, funciones, comentarios, cadenas y números.
- Autocompletado de palabras ST, plantillas y funciones/variables declaradas en el archivo actual.
- `Ctrl+Espacio` abre sugerencias; flechas seleccionan, Enter/Tab insertan y Escape cierra.
- `Ctrl+S` guarda; `F5` o **Ejecutar** guarda, compila y ejecuta el `.st` o `.py` abierto mediante PyForge.
- **Comprobar** o `Ctrl+Mayús+B` guarda y analiza sintaxis/tipos sin ejecutar el programa.
- **Buscar** o `Ctrl+F` navega por coincidencias; `Ctrl+H` abre reemplazo individual o completo, con opción para distinguir mayúsculas.
- **Detener** cancela el grupo de procesos de compilación/ejecución.
- Salida de programas y diagnósticos del compilador en el panel inferior.
- Creación de archivos ST, sangría con Tab y conservación de sangría al pulsar Enter.
- Aviso antes de descartar cambios y detección de cambios externos al guardar.

Los paquetes Python como Flask no están soportados por el compilador actual. Consulta [la guía de evolución de ST](../docs/README.md).

El botón Ejecutar usa el archivo abierto, no la entrada del manifiesto. Los ejecutables temporales no se guardan encima de los del proyecto. Solo hay una ejecución por servidor. Cerrar la pestaña no detiene el servidor: usa Detener o Ctrl+C.

## Organización

| Archivo | Responsabilidad |
| --- | --- |
| `server.py` | Archivos del proyecto, servidor local y procesos de compilación |
| `index.html` | Explorador, editor, acciones y panel de salida |
| `style.css` | Interfaz oscura y colores del lenguaje |
| `language.py` | Lectura y validación automática del vocabulario Rust |
| `app.js` | Edición, autocompletado, resaltado y comunicación con la API |
| `search.js` | Búsqueda circular y reemplazo de texto, aislados para poder probarlos |
| `st.svg` | Rayo amarillo sobre negro, sin letras, para ST y el navegador |
| `python.svg` | Icono azul y amarillo para archivos Python |
| `python_language.py` | Vocabulario y reconocimiento léxico Python sin ejecutar código |

Al guardar cambios en `KEYWORDS` de `src/st/lexer.rs`, el IDE actualiza colores, autocompletado y plantillas automáticamente (aproximadamente cada 1,5 segundos). No edites otra lista de palabras en JavaScript. Recompila PyForge para que el ejecutable acepte la nueva sintaxis. Para cambiar colores modifica las reglas `#highlight` en `style.css`. Consulta [la sincronización del vocabulario](../docs/MANUAL_ST.md).

## Alcance de esta primera versión

Es un editor de un archivo activo, con sugerencias locales; no un servidor de lenguaje con análisis de tipos o referencias entre archivos. Python dispone de colores mediante `tokenize` de la versión que ejecuta el servidor, sugerencias de palabras reservadas y nombres incorporados, plantillas y símbolos locales. La ejecución sigue limitada al subconjunto admitido por PyForge. Los otros formatos solo se editan. No incluye depurador, shell de comandos, renombrado de símbolos ni instalación de iconos en el explorador del sistema operativo. El icono `.st` se muestra dentro del IDE.

El panel inferior es salida de procesos, no una consola de comandos. Las limitaciones del compilador ST siguen aplicando. El IDE ejecuta el código que selecciones con los permisos de tu usuario; no es una máquina aislada para programas desconocidos.

El servidor escucha únicamente en `127.0.0.1` y las operaciones de archivo se restringen a la carpeta del proyecto. No lo publiques en Internet. Los archivos editables se limitan a 1 MB y la salida conservada al último megabyte.

## Comprobar el servidor

Con el compilador construido, desde la raíz:

```bash
python3 -m unittest discover -s ide -v
```

Estas pruebas usan proyectos temporales, ejecutan programas reales y comprueban la cancelación, los diagnósticos, Unicode y las restricciones de rutas.

## Reconocimiento de Python

Al abrir un `.py`, el IDE colorea palabras reservadas, comentarios, cadenas con prefijos y multilínea, números, funciones y nombres incorporados. Las construcciones reconocidas corresponden al Python usado para iniciar el servidor. Las palabras contextuales como `match` se destacan de forma léxica. Las expresiones dentro de cadenas formateadas dependen de los tokens que proporcione esa versión.

`Ctrl+Espacio` ofrece palabras, nombres incorporados, símbolos locales y plantillas para `def`, `class`, `if`, `for`, `while`, `try`, `with` y `match`. Enter añade sangría después de `:`. No incluye análisis de tipos ni resolución de métodos o paquetes externos como un servidor de lenguaje completo.

Para cambiar las plantillas edita `pythonSnippets` en `app.js`; para cambiar categorías, `highlight` en `python_language.py`; los colores están en `style.css`. El texto se analiza tras una breve pausa de escritura, sin ejecutarlo. Una respuesta antigua se descarta si cambias de archivo o contenido. Los fallos de conexión conservan el texto plano.

El icono de cada `.py` se carga desde `python.svg`; los `.st`, la cabecera y el favicon usan `st.svg`, ahora un rayo amarillo sobre negro sin letras. Ambos son SVG comentados y editables.

Reinicia el servidor y recarga el navegador después de instalar esta actualización.

## Sugerencias junto al cursor

El menú aparece desde la primera letra, debajo del cursor, para palabras reservadas y nombres locales reconocidos. Si falta espacio inferior se coloca encima, y cerca del borde derecho se desplaza para permanecer dentro del editor. Sigue el desplazamiento del texto y los cambios de tamaño del panel. Ctrl+Espacio también permite abrirlo manualmente; flechas seleccionan, Enter/Tab insertan y Escape cierra.

`caretCoordinates` en `app.js` mide la línea actual con la misma fuente y tabulaciones del editor. `positionCompletion` convierte esas medidas en coordenadas del panel y controla los límites. El reconocimiento de símbolos sigue siendo local y léxico; no resuelve todos los ámbitos ni los atributos de paquetes externos.

## Crear dentro de src y eliminar archivos

**Nuevo archivo** crea siempre dentro de `src` del proyecto abierto. `programa.st` se guarda en `src/programa.st`; `utilidades/math.st`, en `src/utilidades/math.st`. También puedes escribir el prefijo `src/` sin que se duplique. Se crean las subcarpetas necesarias y se mantiene el formato `.st` para los archivos nuevos. No se sobrescriben archivos existentes ni se admiten rutas absolutas, ascensos con `..` o enlaces simbólicos.

Para eliminar, abre el archivo y pulsa **Eliminar** junto al nombre. La confirmación muestra su ruta y avisa si hay cambios sin guardar. El servidor comprueba que el contenido guardado no haya cambiado y exige detener cualquier ejecución activa. Solo mueve archivos editables; no borra carpetas.

La eliminación mueve el archivo a `.st-ide-trash/archivo-…/nombre` dentro del proyecto. La ubicación aparece en el aviso; puedes restaurarlo manualmente desde esa carpeta a su ruta anterior. La papelera queda oculta en el explorador del IDE y no se vacía automáticamente. Solo se conserva la versión guardada en disco, no cambios sin guardar en el editor.

`Workspace.source_path` normaliza las rutas nuevas. `Workspace.remove` valida y mueve a la papelera. En `app.js`, `create` abre la ruta normalizada devuelta por la API y `deleteFile` confirma la acción, actualiza el explorador y cierra el archivo retirado. Estas funciones están comentadas en español.

El tema conserva la tipografía de terminal, simplifica las superficies y usa amarillo tenue para acciones y selección. Las variables al principio de `style.css` permiten ajustar la paleta.

## Entrada interactiva en la terminal

Ejecuta el archivo y responde a `entrada()` desde la línea de la terminal inferior. Enter o **Enviar** entrega una respuesta; **Fin de entrada** o Ctrl+D con la línea vacía envía EOF. **Detener** cancela la ejecución aunque esté esperando una respuesta. Se admiten 4095 bytes UTF-8 por envío. La línea se habilita mientras el programa acepta entrada. Reinicia el servidor y recarga el navegador para activar esta interfaz.

Sin indicar proyecto, el IDE abre la carpeta desde la que se inició; un argumento de carpeta o `PYFORGE_PROJECT` permite elegir otra raíz explícitamente. Los programas se ejecutan desde esa carpeta del proyecto. El explorador restringe las rutas a esa carpeta; la terminal envía texto al programa, sin ejecutar comandos de shell. El directorio de trabajo no impide que un programa nativo acceda a otros archivos permitidos al usuario. La API conserva la opción de entrada preparada para clientes anteriores.

## Comentarios multilínea ST

Escribe `/*` o `"""` y acepta la plantilla con Enter o Tab: el IDE inserta el cierre, conserva la sangría y coloca el cursor en la línea interior. Ambos formatos reciben color de comentario. Las sugerencias de código se ocultan dentro del comentario; se puede escribir el texto libremente. Los bloques no se anidan y las comillas triples de ST no son cadenas. El editor Python conserva sus propias reglas.

La prueba `node ide/test_comments.js` comprueba las funciones JavaScript reales de coloreado, sugerencias y posición del cursor. Node se necesita solo para ejecutar esa prueba, no para usar el IDE.

## Biblioteca estándar ST

Tras `importar "math";`, escribe `math.` para ver sus funciones públicas; también se reconocen alias como `importar "math" como m;`. Las firmas proceden de `src/stdlib/*.st` y se actualizan al consultar el vocabulario. Los auxiliares `aux_` no se sugieren. Recompila el compilador para ejecutar funciones nuevas; las sugerencias locales no sustituyen la comprobación semántica.

## Plegar bloques de código

Pulsa **▾** junto al número de línea para ocultar el interior de un bloque y **▸** para desplegarlo. **Desplegar todo** muestra el documento completo. ST reconoce bloques entre llaves y comentarios multilínea, ignorando delimitadores dentro de cadenas y comentarios. Python usa una detección conservadora por indentación; no sustituye a su analizador completo.

Puedes plegar funciones, clases, condiciones y bucles, incluidos bloques anidados. Los números siguen correspondiendo al archivo original. Guardar y Ejecutar siempre usan el documento completo. El plegado solo vive en la sesión del archivo abierto; no modifica su contenido ni se conserva al volver a abrirlo. Ctrl+Z deshace y Ctrl+Y o Ctrl+Mayús+Z rehace hasta 50 ediciones; alternar flechas no cuenta como edición.

`folding.js` separa el texto real de la proyección visible y traduce posiciones para editar sin perder interiores ocultos. `app.js` dibuja los controles, reconcilia la entrada y guarda el texto completo. Las pruebas `test_folding.js` cubren edición cerca de los límites, anidación, historial y guardado/ejecución de bloques ocultos.

## Explorador y tamaño de terminal

**Ctrl+B** muestra/oculta la barra lateral conservando la edición. Arrastra el borde superior de la terminal para cambiar su altura; también puedes enfocar ese borde y usar flechas, Home o End. El ajuste respeta el espacio mínimo del editor. Se ofrecen plantillas de `tipo(valor)`, `entero(valor)`, `flotante(valor)` y `booleano(valor)` sin reservar el identificador `tipo`.


## Inicio, apoyo y formatos

Al abrir la aplicación instalada sin seleccionar una fuente aparece un menú en el navegador. Abrir directamente una fuente conserva la entrada al editor. El logo de la barra permite volver al menú; los cambios sin guardar mantienen la confirmación del navegador.

La barra principal ya no contiene Archivos ni Autocierre: Ctrl+B alterna el explorador y el autocierre siempre está activado, incluso si una versión anterior guardó otra preferencia. Apoyar ST abre una página local con logo, mensaje y botón de donación a PayPal.

JSON y TOML se colorean mediante `formats.js`, que conserva y escapa el texto. Los archivos `.db` SQLite aparecen en el explorador y muestran esquema SQL y una muestra de filas en solo lectura. No se editan bytes binarios ni se permite guardar esa vista. Las filas se muestran como comentarios JSON; tablas virtuales y vistas no se ejecutan.

Para personalizar el menú consulta [INSTALACION.md](../docs/INSTALACION.md). `start_menu.py` valida opciones, `inicio.js` las presenta, `server.py` procesa las acciones autenticadas y `scripts/update-ide-menu.py` actualiza la configuración del usuario. No se modifica el compilador.

## Manual y acceso desde la terminal

`/manual.md` muestra el manual estilizado y coloreado; `/manual-fuente.md` permite descargar Markdown. El enlace Inicio de la salida y Ctrl+Alt+H vuelven al menú aun con foco en la terminal. No detienen el programa. Consulta [INTEGRACIONES.md](../docs/INTEGRACIONES.md) para ampliar las ventanas Qt de ST.

El menú abre un selector nativo para Entrar al editor y Abrir proyecto. Crear proyecto selecciona primero la carpeta padre y pide solo un nombre. Cancelar no cambia el proyecto. El `.deb` incluye la dependencia `python3-tk`; desde las fuentes necesitas Tk y una sesión gráfica.

## Proyectos cifrados y temas

Inicio permite crear/abrir una bóveda `.stvault`; el modo normal sigue sin contraseña. Bloquear guarda el archivo actual y exige autenticación para volver. Consulta [INSTALACION.md](../docs/INSTALACION.md) para cifrado, memoria, sesiones y límites. El selector de temas ofrece Normal, Hacker y Pro oscuro.
