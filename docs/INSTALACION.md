# Instaladores de PyForge y ST IDE

El mismo generador crea un `.deb` para Linux x86-64 y un instalador Windows `.exe`
que utiliza **WSL**. Ambos distribuyen LLVM 21/Inkwell como único backend. No se
ha añadido un generador de programas Windows nativos: los
programas ST/Python compilados siguen siendo ELF Linux x86-64. El IDE también se
ejecuta dentro de Linux.

## Construir ambos paquetes

Desde el repositorio, en Linux x86-64, necesitas Rust/Cargo, Python 3.11+, binutils,
LLVM 21 de desarrollo, dpkg y NSIS (`makensis`). En Ubuntu/Debian:

```bash
sudo apt install binutils llvm-21-dev libllvm21 dpkg-dev nsis
```

El empaquetador busca LLVM en `LLVM_SYS_211_PREFIX`, mediante `llvm-config-21` o
en `target/tools/llvm-21/usr/lib/llvm-21`. Esa última ruta permite usar paquetes de
desarrollo extraídos localmente, pero el `.deb` nunca guarda esa ruta: declara
`libllvm21` como dependencia de ejecución.

```bash
python3 scripts/package.py
```

Para construir únicamente Linux sin NSIS:

```bash
python3 scripts/package.py --format deb
```

Los resultados se guardan en `dist/packages/`. El nombre incluye la versión de `Cargo.toml`, la fecha UTC y una huella de las fuentes. `latest.json` identifica los artefactos de la última construcción exitosa; cada instalación también contiene `build.json` con la huella y la lista de entradas.

El generador prueba y compila una instantánea de las fuentes con LLVM y ejecuta el
catálogo en los dos modos de optimización. Incluye los binarios recién compilados,
el IDE, `src` (incluido
`KEYWORDS`), los ejemplos, el manual y la licencia. No reutiliza los binarios
antiguos de `dist/`. Excluye perfiles del navegador, cachés y directorios privados.
La caché de Cargo queda en `target/packages/`.

## Regenerar al modificar el lenguaje o el IDE

```bash
python3 scripts/package.py --watch
```

Mientras ese proceso esté abierto, guardar, añadir o eliminar fuentes Rust/ensamblador, recursos del IDE, ejemplos, documentación o scripts de empaquetado provoca una reconstrucción después de dos segundos de estabilidad. La huella incluye el contenido y los nombres, no solamente fechas de modificación. Ctrl+C detiene la vigilancia. Para Linux exclusivamente: añade `--format deb`.

Las pruebas deben pasar antes de publicar un paquete. Si una edición rompe la compilación, se informa el error y se conservan los paquetes anteriores. Una nueva edición vuelve a intentarlo. Los cambios que lleguen durante una construcción provocan otra construcción al terminar. Los paquetes históricos no se borran automáticamente.

**Un instalador ya creado o una instalación existente no se modifica sola.** El vigilante crea instaladores nuevos; debes ejecutar el nuevo instalador para actualizar otra máquina. No se instalan servicios de actualización, no se modifican permisos y no se ejecuta `sudo` al guardar un archivo. Añadir una biblioteca externa también puede requerir declarar una nueva dependencia: ningún vigilante puede deducir cualquier requisito del sistema.

## Instalar el .deb

Usa el nombre generado que aparece en `dist/packages/latest.json`:

```bash
sudo apt install ./dist/packages/pyforge_VERSION_amd64.deb
pyforge --version
pyforge new calculator
cd calculator
pyforge run
pyforge-ide .
```

Sustituye `VERSION` por la versión real del archivo. APT resuelve las dependencias
declaradas. El paquete instala `pyforge`, `pyforgec`, `pyforge-ide`,
`pyforge-engine3d`, `pyforge-editor3d` y `pyforge-game3d` en `/usr/bin`, que pertenece
al PATH habitual; no añade líneas a `.bashrc` ni sustituye PATH.

El componente 3D es opcional durante el uso. Para abrirlo:

```bash
pyforge motor3d --vulkan
pyforge motor3d --opengl
pyforge motor3d --editor
pyforge motor3d --juego ./mi-logica-st
pyforge motor3d --juego cristales ./mi-logica-st
```

Los perfiles de juego distribuidos son `portales`, `cristales`, `laberinto` y
`guardianes`. Sin perfil se conserva `portales` por compatibilidad. Los proyectos y
sus pruebas están en `examples/games/`.

El editor 3D instalado incluye el icono oficial y encuentra `pyforgec` en el mismo
directorio `/usr/bin`. Para probar una escena declarada enteramente en ST, abre
`/usr/share/pyforge/examples/editor3d/escena_profesional.st` desde la pestaña
**ST Script de escena** y pulsa **Aplicar ST**. El tutorial completo está en
`/usr/share/pyforge/examples/editor3d/README.md`.

Sus lanzadores compilan la lógica en `$XDG_CACHE_HOME/pyforge/games` o
`~/.cache/pyforge/games`. Por eso pueden ejecutarse desde la copia de solo lectura
instalada en `/usr/share/pyforge/examples` sin permisos de administrador.

El `.deb` recomienda controladores Vulkan/Mesa, `pipewire-bin` y `alsa-utils`. No los
convierte en dependencias obligatorias porque el compilador y los programas de
terminal funcionan sin GPU ni audio. `pw-cat` proporciona la salida PipeWire y
`aplay` la salida ALSA; la detección automática prefiere PipeWire.

El compilador requiere glibc al menos tan reciente como los símbolos usados por el binario. La dependencia se calcula con `readelf`, evitando prometer compatibilidad con distribuciones antiguas. Para distribuir a una versión anterior de Linux, construye el paquete en esa distribución y vuelve a probarlo.

Recursos: `/usr/share/pyforge/`. `pyforge-ide` abre por defecto el directorio actual; acepta una ruta de proyecto y `--port`. Para actualizar, instala el nuevo `.deb`. Para desinstalar, `sudo apt remove pyforge`. Guarda tus proyectos fuera de `/usr/share/pyforge`; el gestor de paquetes administra esa carpeta.

## Instalar el .exe en Windows

Requisitos: Windows x64 con WSL configurado y una distribución **predeterminada
Linux x86-64**, glibc igual o superior a la indicada por el instalador, Python
3.11+, binutils y `libllvm21`. Dentro de WSL:

```bash
sudo apt update
sudo apt install binutils libllvm21 python3 python3-tk python3-cryptography
```

El lanzador comprueba binutils y LLVM antes de iniciar el compilador y muestra la
orden correspondiente si faltan. El instalador actual construido en este equipo
requiere la versión de glibc registrada en sus metadatos; para una distribución
anterior debes reconstruir y probar el paquete en esa base.

Microsoft documenta cómo [instalar WSL](https://learn.microsoft.com/en-us/windows/wsl/install) y [seleccionar una distribución predeterminada](https://learn.microsoft.com/en-us/windows/wsl/basic-commands). Si no tienes WSL, prepara esa distribución antes de ejecutar PyForge; el instalador no habilita componentes del sistema ni descarga distribuciones sin intervención.

Ejecuta `PyForge-VERSION-windows-wsl-setup.exe`. Instala por usuario en `%LOCALAPPDATA%\Programs\PyForge` y añade su carpeta `bin` al PATH del usuario. Abre una terminal **nueva** después de instalar:

```powershell
pyforge --version
pyforge new calculator
cd calculator
pyforge run
pyforge-ide .
```

Los comandos son lanzadores `.cmd` que pasan argumentos a PowerShell y WSL. Conservan el directorio de trabajo, aceptan rutas relativas y convierten rutas absolutas Windows mediante `wslpath`. El proyecto debe estar en una ubicación accesible para WSL. Para proyectos guardados exclusivamente dentro de una distribución, también puedes trabajar directamente desde su terminal Linux.

El IDE muestra una URL local que puedes abrir en tu navegador. Las respuestas al programa se escriben en la terminal del IDE. Cambiar la distribución WSL predeterminada cambia el entorno utilizado por estos comandos.

La actualización usa la carpeta registrada de la instalación anterior. La desinstalación se ofrece en Aplicaciones y mediante `Uninstall.exe`; retira la entrada PATH solamente si la añadió PyForge, conservando las demás. No guardes proyectos dentro de las carpetas administradas `payload` o `bin`.

El `.exe` no tiene firma Authenticode; Windows puede mostrar una advertencia de editor desconocido. Su generación en Linux no sustituye una prueba de instalación, actualización y desinstalación en Windows/WSL.

## Dónde cambiar cada parte

| Archivo o función | Responsabilidad |
| --- | --- |
| `scripts/package.py`, `inputs` | Selecciona los archivos vigilados y compila también los tres binarios 3D |
| `snapshot` | Captura contenido y calcula la huella del estado del proyecto |
| `run` | Ejecuta herramientas mediante listas de argumentos, sin shell |
| `copy_tree` | Copia recursos desde la instantánea |
| `libc_requirement` | Detecta la versión mínima real de glibc |
| `llvm_environment` | Valida LLVM 21 y prepara `LLVM_SYS_211_PREFIX` sin modificar el sistema |
| `build` | Prueba, compila, crea/verifica el `.deb`, genera el `.exe` y publica los artefactos |
| `main` | Procesa opciones, espera estabilidad y reconstruye con `--watch` |
| `packaging/windows/installer.nsi` | Interfaz NSIS en español, instalación por usuario y desinstalación |
| `environment.ps1` | Añade/retira la entrada PATH sin truncar su contenido ni expandir variables existentes |
| `launch.ps1` | Localiza el puente dentro de WSL y propaga el estado de salida |
| `launch.py`, `linux_path` / `main` | Convierte rutas, comprueba binutils/LLVM y ejecuta únicamente comandos permitidos |
| `ide/server.py`, `main` | Acepta `PYFORGE_PROJECT` para abrir el proyecto actual con recursos instalados |

NSIS documenta su [sintaxis de instaladores](https://nsis.sourceforge.io/Docs/Chapter4.html). El generador permite seleccionar un compilador NSIS específico mediante la variable `MAKENSIS`; la opción normal sigue siendo `makensis` en PATH.

En este espacio de trabajo se dejó una copia local de NSIS en `target/tools/nsis/`, descargada desde los repositorios de Ubuntu. El generador la utiliza si no hay `makensis` en PATH. No modifica el sistema y no se incluye dentro de los instaladores. Si borras `target/` o copias el proyecto a otra máquina, instala NSIS para volver a generar el `.exe`.

## Iconos de archivos ST

Las versiones nuevas incluyen el rayo de `website/logo.svg` como icono de archivos
`.st` en Linux y Windows. Hay registro MIME/hicolor en Linux y ProgID/ICO por usuario
en Windows. Se conservan aplicaciones predeterminadas existentes; puedes elegir
ST IDE desde «Abrir con». El doble clic abre el editor, no ejecuta la fuente.
El icono se regenera al reconstruir el paquete. Abrir una fuente no la ejecuta; la ejecución requiere pulsar Ejecutar.

Al construir paquetes se necesita el cargador SVG de GdkPixbuf y Python GI.
El cambio del SVG queda incluido en la huella vigilada por `scripts/package.py`.


## Uso y personalizacion del IDE

### Al abrir PyForge instalado

El acceso **ST IDE** de la aplicación `.deb` abre el navegador y presenta:

- Entrar al editor: elige una carpeta en el explorador del sistema.
- Abrir proyecto: elige una carpeta existente en el explorador del sistema.
- Crear proyecto ST: elige la carpeta donde guardarlo y escribe únicamente el nombre nuevo.
- Consultar el manual local, disponible sin conexión.
- Apoyar ST: página con el logo, un mensaje y botón de donación a PayPal.

No necesitas escribir rutas. Cancelar el selector conserva la carpeta actual. El selector usa Tk en un proceso separado, incluido mediante la dependencia `python3-tk` del `.deb`; necesita una sesión gráfica y, en WSL, WSLg con Tk instalado. La creación reutiliza `pyforge new`; incluye `src/main.st` y no sobrescribe carpetas existentes. Detén el programa antes de cambiar de proyecto. Cambiar de carpeta renueva el token de sesión: otras pestañas antiguas necesitan recargarse. El logo del editor vuelve al menú; el navegador avisa si tienes cambios sin guardar.

Al abrir un archivo `.st` desde el explorador del sistema, se entra directamente en ese archivo. El comando `pyforge-ide carpeta` conserva su inicio habitual en el editor; `pyforge-ide --open-browser carpeta` muestra el menú cuando no se ha indicado una fuente.

### Personalizar sin permisos de administrador

Desde una instalación `.deb`, crea una plantilla en tu carpeta de trabajo:

```bash
python3 /usr/share/pyforge/scripts/update-ide-menu.py --plantilla mi-menu.json
```

Edita sus textos y conserva únicamente las opciones que quieras:

```json
{
  "titulo": "Bienvenido a ST",
  "mensaje": "Elige cómo comenzar tu proyecto.",
  "opciones": [
    {"accion": "editor", "texto": "Programar"},
    {"accion": "abrir", "texto": "Abrir mi proyecto"},
    {"accion": "crear", "texto": "Crear proyecto"},
    {"accion": "manual", "texto": "Aprender ST"},
    {"accion": "apoyar", "texto": "Apoyar el desarrollo"}
  ]
}
```

Aplica el cambio:

```bash
python3 /usr/share/pyforge/scripts/update-ide-menu.py mi-menu.json
```

Recarga la página de inicio. El script valida el archivo y escribe de forma atómica en `~/.config/pyforge/menu.json`, o bajo `XDG_CONFIG_HOME` si está definido. No necesita sudo, conexión a Internet ni recompilar. Una configuración inválida recupera el menú predeterminado. El archivo admite texto y acciones conocidas, nunca instrucciones de shell ni HTML.

Desde las fuentes, usa `python3 scripts/update-ide-menu.py` con los mismos argumentos. Para cambiar los valores predeterminados de futuras instalaciones, edita `ide/menu.json` y reconstruye los paquetes con `python3 scripts/package.py --format all`. Para añadir una acción nueva, amplía explícitamente `ACTIONS` en `ide/start_menu.py`, su presentación en `ide/inicio.js` y, si necesita operaciones locales, una ruta autenticada en `ide/server.py`. Añade pruebas antes de distribuirla.

### Barra, apoyo y colores

Ctrl+B muestra u oculta el explorador. El botón Archivos se retiró de la barra y el autocierre permanece activado sin botón. La terminal conserva su borde redimensionable.

El enlace Apoyar ST abre `ide/apoyar.html` en otra pestaña; desde allí el botón Donar abre el destino PayPal existente. Puedes modificar el mensaje en ese HTML y los estilos en `ide/style.css`. No se realiza ninguna transacción al abrir la página.

`ide/formats.js` reconoce claves, cadenas, números y booleanos de JSON; añade comentarios y secciones para TOML. El resaltado tolera texto incompleto y no valida ni ejecuta los documentos. Las variables y claves conservan el amarillo del IDE.

### Bases SQLite

`.db` es un contenedor binario, no un script. El IDE lo abre mediante `ide/database.py` en **solo lectura**: muestra el esquema SQL coloreado y hasta 20 filas de cada una de 30 tablas. Los valores largos se truncan a 200 caracteres; los datos binarios se describen por tamaño. Las filas se muestran como comentarios JSON junto al esquema.

La consulta tiene límites de tamaño y tiempo. No ejecuta vistas ni tablas virtuales, y la API rechaza guardar el archivo `.db`. Un archivo de otro formato con esa extensión presenta un error claro. Para cambiar datos, utiliza las funciones del módulo `db` desde un programa ST, con su configuración de permisos; el visor no sustituye un administrador completo de bases de datos.


## Privacidad y proyectos cifrados

### Elegir protección

El menú conserva los proyectos normales **sin contraseña**. En «Proyecto cifrado con contraseña» puedes crear una bóveda nueva o abrir un archivo `.stvault` existente. El selector del sistema elige el archivo y la contraseña debe tener entre 12 y 256 caracteres.

Crear una bóveda genera un proyecto ST nuevo. **No convierte ni borra una carpeta normal existente**. Si copias código desde otro lugar, esa copia original sigue existiendo: cifrar la bóveda no protege archivos, copias de seguridad o capturas que permanezcan fuera.

El botón Bloquear guarda primero el archivo editado, cifra el proyecto, revoca las sesiones y elimina la copia temporal de trabajo. Para cambiar de bóveda o volver al modo normal, cierra esa instancia del IDE y abre otra. Guarda tus cambios antes de cerrar el navegador; cerrar una pestaña no termina por sí solo el servidor local.

### Qué se cifra y cuándo

El contenedor usa AES-256-GCM con sal de 16 bytes, nonce aleatorio de 12 bytes por guardado y clave derivada mediante scrypt (`N=32768`, `r=8`, `p=1`). No se almacena la contraseña. Una contraseña equivocada o un contenedor alterado falla al autenticarse.

La copia de trabajo se crea en `/dev/shm` (tmpfs), con directorio privado. Los programas se compilan allí y se ejecutan mediante un descriptor anónimo de memoria cuando tmpfs es `noexec`. Las escrituras del editor se guardan también en el contenedor cifrado mediante reemplazo atómico. Al finalizar una ejecución se guarda el proyecto otra vez.

Límites: 32 MB y 2000 archivos, sin enlaces simbólicos. Se excluyen artefactos de compilación, cachés, `.git`, entornos y papelera. La bóveda es para fuentes y recursos de proyectos pequeños, no sustituye un respaldo consistente de una base de datos activa.

### Acceso desde el navegador

Las páginas y API del proyecto protegido requieren una cookie de sesión HttpOnly y SameSite, además del token de la API. La sesión caduca a los 15 minutos; al recargar pide la contraseña. Hay un límite de cinco intentos por minuto. Bloquear revoca todas las sesiones. El IDE sigue escuchando exclusivamente en loopback, no en la red pública.

No existe recuperación de contraseña. Conserva una copia cifrada de la bóveda y guarda la contraseña por un medio adecuado. Si la pierdes, el IDE no puede recuperar los archivos.

### Límites que debes conocer

- Mientras el proyecto está abierto, un proceso con los permisos de tu usuario, o un administrador, puede leer memoria y archivos de la sesión. El cifrado no protege un sistema ya comprometido.
- tmpfs puede utilizar swap. Para proteger un equipo perdido, usa también cifrado de disco/swap. No se promete borrado físico seguro ni eliminación de todas las copias de memoria de Python o del navegador.
- El código ST que ejecutes conserva los permisos de tu cuenta y puede escribir fuera de la bóveda. No se trata de un sandbox para código hostil.
- Otros navegadores o pestañas que ya recibieron código pueden conservarlo. Cambiar contraseñas o bloquear acceso futuro no retira copias ya realizadas.
- La caducidad bloquea nuevas solicitudes; guarda periódicamente y vuelve a autenticarte si la sesión caduca. El texto aún no guardado en el editor vive en el navegador, no en el contenedor.
- Un cierre abrupto del equipo conserva el último guardado cifrado completo; no promete recuperar cambios todavía sin guardar.

La dependencia `cryptography` viene declarada en el `.deb`. El funcionamiento en Windows requiere WSL con `/dev/shm`, Python Tk y una sesión gráfica compatible.

### Temas

El selector ofrece **Normal**, **Hacker** (verde oscuro) y **Pro oscuro** (azul/cian). Se recuerda únicamente el nombre del tema en localStorage. No cambia permisos, cifrado ni ejecución. La paleta está en `ide/style.css`, y la selección en `ide/preferences.js`.

### Dónde ampliar

- `ide/vault.py`: formato, derivación, validación, guardado atómico y copia de trabajo.
- `ide/privacy.py`: sesiones, caducidad, intento de contraseña y revocación.
- `ide/server.py`: protección de rutas, apertura, bloqueo y ejecución en memoria.
- `ide/lock.html`, `lock.js`, `preferences.js`: presentación; no guardar contraseñas en almacenamiento del navegador.
- `ide/test_privacy.py`: cifrado, alteración, API y ejecución nativa en memoria.

El diseño usa las primitivas de [cryptography](https://cryptography.io/en/latest/hazmat/primitives/key-derivation-functions.html); no implementa AES manualmente.

## Extension de VS Code

La fuente pública está en `extensions/vscode-st`; `.vscode` contiene configuración
local y no es la fuente distribuible. Para construir e instalar la extensión:

```bash
python3 scripts/build-vscode-extension.py
code --install-extension dist/extensions/pyforge-st-0.1.0.vsix --force
```

F5 ejecuta la fuente mediante argumentos separados y exige confianza en el espacio
de trabajo. No implementa depuración ni breakpoints. El vocabulario viene incluido;
`pyforge.sourceRoot` permite leer el de una copia del compilador. Cambia
`pyforge.compilerPath` si el ejecutable no está en PATH. En Windows se necesita WSL.
Las pruebas con un anfitrión simulado no sustituyen una comprobación visual en
VS Code; no se garantiza compatibilidad con todas sus versiones futuras.

## Publicar el curso y la web

```bash
python3 scripts/build-site.py
python3 -m http.server 8080 --bind 127.0.0.1 --directory dist/site
```

Abre `http://127.0.0.1:8080`. Publica el contenido de `dist/site/`; abrir el HTML
como archivo local puede impedir cargar `curso.json`. El generador actual publica
el directorio, no genera un ZIP nuevo automáticamente. No uses un ZIP antiguo como
si reflejara la construcción actual.

`docs/MANUAL_ST.md` alimenta tanto el curso como `/manual.md` del IDE; cada título
`##` del manual crea una lección. El IDE también ofrece `/manual-fuente.md`.
`scripts/update-explanation.py` actualiza la copia `explicacion.md` de la raíz.
Las cuatro guías se copian a `dist/site/docs`. `KEYWORDS` alimenta el vocabulario,
y los ejemplos públicos se colorean sin ejecutarse. Entornos virtuales, cachés,
resultados y datos quedan fuera del catálogo. El backend no se publica como estático.

Puedes usar `python3 scripts/build-site.py --watch` y, en otra terminal,
`python3 scripts/package.py --watch`. El primero actualiza la página; el segundo
prueba y genera instaladores. Ninguno actualiza instalaciones remotas. Implementar
una función nueva no redacta su lección: actualiza el manual y sus ejemplos.
La página `descargas.html` obtiene los paquetes de `dist/packages/latest.json`,
que el generador verifica por SHA-256 antes de publicarlos. Los archivos fuente
de ejemplos también se pueden descargar desde la página Ejemplos. La documentación
puede ser más reciente que el último instalador.

La web estática no puede registrar cuentas ni almacenar discusiones. El foro,
registro y login requieren `website/backend`, su base de datos y el despliegue
explicado en [su README](../website/backend/README.md). Conserva HTTPS, secretos
externos, hosts explícitos y copias privadas. Consulta [integraciones web](INTEGRACIONES.md#web-y-foro).

## Mantener estas cuatro guias

- `MANUAL_ST.md`: sintaxis, tipos, ejemplos, módulos, bibliotecas y juegos.
- `INSTALACION.md`: paquetes, IDE, privacidad, extensión y publicación del curso.
- `INTEGRACIONES.md`: permisos, API, documentos, redes, SQL, GUI y web.
- `README.md`: definición de ST, plan por etapas, motor 3D y arquitectura actual.

Edita el apartado correspondiente, sin crear otro Markdown en `docs`.
`update-integrations-api.py` actualiza únicamente la sección API de integraciones.
Los paquetes ya construidos conservan la documentación que tenían al crearlos;
para distribuir esta reorganización, genera un instalador nuevo.

## Construcción del runtime nativo experimental

`build.rs` compila `runtime/native/lib.rs` como biblioteca estática `no_std` con el
Rust utilizado por Cargo, para Linux x86-64. La biblioteca se incrusta en `pyforgec`
y se enlaza cuando el programa usa `nativo`. No hay descarga de dependencias nuevas
para ese runtime. Los instaladores futuros incluyen el cambio al reconstruirse;
los instaladores anteriores no se actualizan solos.

`--keep` conserva también `.pyforge-build/<programa>.native.a` cuando corresponde.
No es necesario distribuir ese archivo junto al ejecutable final. El auxiliar Python
continúa incluido para sus módulos originales. La independencia de Python solo se
aplica a programas que no invocan esas integraciones.

El módulo tipado `recursos` utiliza la misma biblioteca incrustada. La sintaxis
`usar` se resuelve en compilación y no añade una dependencia al ejecutable. La
extensión de VS Code y el IDE leen `CONTEXTUAL_WORDS` para colorear `usar`,
`archivo_nativo` y `buffer_nativo` sin convertirlos en palabras reservadas globales.
