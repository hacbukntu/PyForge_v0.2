# PyForge v0.2

**80 proyectos ST completos:** [abre el catálogo](examples/proyectos/README.md). Juegos de consola, negocio, organización, texto, archivos, algoritmos, clases e informes, con instrucciones y pruebas reproducibles.

**Tipos ST:** variables declaradas e inicializadas, inferencia, `tipo(valor)`, conversiones explícitas y listas ordenadas y mutables con elementos de distintos tipos mediante `lista[mixto]`. **Módulos ST:** funciones y variables `publica`/`privada`, nombres aislados y exportaciones de solo lectura. Consulta la sección 25 de [explicacion.md](explicacion.md), [el ejemplo de listas mixtas](examples/listas_mixtas.st), [el ejemplo de conversiones](examples/inferencia_y_conversiones.st) y [el ejemplo de visibilidad](examples/modules_visibility/main.st).

**Biblioteca estándar nativa:** `importar "time";`, `"os"`, `"math"`, `"io"` y `"json"`. Funciones en español, autocompletado y ejemplos en la sección 24 de [explicacion.md](explicacion.md).

Ahora ST incluye clases con campos tipados y métodos, excepciones explícitas con `finalmente` y `entrada("Ingresa tu nombre: ")`. Consulta las secciones 20–22 del manual para conocer su sintaxis y límites.

**Explicación del lenguaje:** [explicacion.md](explicacion.md), con sintaxis, ejemplos, implementación y límites actuales.

**Visión y plan de desarrollo:** [ST nativo, web y motor 3D](docs/README.md).

**Documentación unificada:** [Manual de ST y PyForge](docs/MANUAL_ST.md), con ejemplos comprobados, arquitectura y capacidades pendientes.

**PyForge es el compilador AOT del lenguaje propio ST, con sintaxis en español y
LLVM 21 como único backend.** El analizador del subconjunto Python se conserva como
compatibilidad beta y experimental; no define la gramática de ST.

Esta versión **no es un transpilador de Python a Rust, C o C++**. PyForge analiza
el código `.st` o el subconjunto `.py`, lo comprueba, lo convierte a su
representación intermedia (IR), lo optimiza y lo baja obligatoriamente a LLVM 21
mediante Inkwell. Después, GNU `as` y `ld` producen un ejecutable ELF nativo para
Linux x86-64.

```text
programa.st
   |
   v
Lexer y parser propios de ST
   |
   v
AST
   |
   v
Análisis semántico y de tipos
   |
   v
IR de PyForge
   |
   v
Optimizador
   |
   v
LLVM 21 + Inkwell
   |
   v
GNU as + GNU ld
   |
   v
Ejecutable ELF nativo
```

Rust es el **lenguaje de implementación del compilador**. Los programas del usuario no se traducen a Rust.

## Tu propio lenguaje `.st`

El compilador también admite una sintaxis ST independiente, con palabras en español, bloques `{ ... }` y sentencias terminadas en `;`:

```text
variable respuesta: entero = 40 + 2;
mostrar(respuesta);
```

```bash
cargo build --release
./target/release/pyforgec run examples/hello.st
```

Los archivos `.py` conservan su analizador original. Consulta [la guía del lenguaje ST](docs/MANUAL_ST.md) para cambiar palabras reservadas, sintaxis y operadores, crear proyectos y entender cada función nueva.

## Módulos y colecciones ST

```text
importar "math.st";
variable numeros = [10, 20];
agregar(numeros, 30);
variable ficha = ["Ada"; 36; verdadero]; // lista[mixto], ordenada y mutable
ficha[1] = 37;
variable datos = {"total": longitud(numeros)};
mostrar(datos["total"]);
```

Consulta [la guía de módulos, colecciones y vocabulario automático del IDE](docs/MANUAL_ST.md) para crear `math.st`, conocer los tipos admitidos y personalizar las palabras reservadas.

Consulta [la guía de evolución de ST](docs/README.md) para conocer las capacidades pendientes, dónde extender el compilador y el camino hacia un framework web.

## IDE local para ST

El editor está en una carpeta independiente, con explorador, colores, autocompletado, iconos `.st` y botones para compilar, ejecutar y detener archivos `.st` y `.py` del subconjunto admitido:

```bash
python3 ide/server.py
```

Abre `http://127.0.0.1:8765`. Consulta [las instrucciones del IDE](ide/README.md) para abrir otro proyecto y conocer los atajos y límites de esta primera versión.

## Ejecutar juegos y demos

### Editor y demostración 3D opcionales

```bash
pyforge motor3d --vulkan
pyforge motor3d --editor
```

El editor incluye viewport sólido, cámara orbital y gizmos para mover, rotar y
escalar entidades. La demostración GPU es una ruta separada con textura, profundidad
y sombreado PBR básico. Consulta las secciones 35–37 del
[manual](docs/MANUAL_ST.md) para conocer las diferencias y los límites actuales.

Cuatro juegos mantienen sus reglas en ST y muestran el estado en una ventana nativa:

```bash
export PATH="$PWD/target/release:$PATH"
examples/games/portal_runner_visual/test_logic.sh
examples/games/portal_runner_visual/build_and_run.sh
python3 examples/games/verify_visual_games.py
```

La [guía de los cuatro juegos](examples/games/JUEGOS_3D.md) explica el protocolo,
controles, perfiles y puntos de ampliación.

### Juego de consola desde el catálogo

El proyecto más simple para probar una partida real es `examples/proyectos/01_adivina_numero`:

```bash
cargo build --release
printf '50\n37\n' | ./target/release/pyforgec run examples/proyectos/01_adivina_numero/src/main.st
```

Salida esperada:

```text
Adivina del 1 al 100: cinco intentos.
Menor
Ganaste
```

También puedes entrar en la carpeta del proyecto y usar el comando corto:

```bash
cd examples/proyectos/01_adivina_numero
pyforge run
```

### Proyecto web con Django y demo de la culebrita

```bash
cargo build --release
mkdir -p /tmp/pyforge-web && cd /tmp/pyforge-web
/home/pwd_hack/Descargas/PyForge_v0.2/target/release/pyforge new demo --web
cd demo
python3 -m venv .venv
. .venv/bin/activate
pip install -r web/requirements.txt
export PYFORGE_WEB_PYTHON="$PWD/.venv/bin/python"
pyforge web
```

Abre `http://127.0.0.1:8000/` y entra con la cuenta local creada por el proyecto. La plantilla incluye una vista y un estilo profesionales para la demo de la culebrita, sin reemplazar la base segura del scaffold.

## Estado

Versión: **0.2.0 experimental**

Destino: **Linux x86-64**

PyForge v0.2 es un prototipo de compilador, no un sustituto directo de CPython. Admite un subconjunto reducido de Python compilable estáticamente para que el código generado use enteros nativos y operaciones de máquina en lugar de objetos Python.

## Novedades respecto a v0.1

- Etapas propias de compilación: código fuente → AST → análisis semántico → IR → optimizador → LLVM 21.
- Sin generación de código fuente Rust a partir de Python.
- `pyforgec`, similar a `rustc`/`gcc`, para archivos fuente individuales.
- `pyforge`, similar a Cargo, para crear, compilar, ejecutar, comprobar y limpiar proyectos.
- Ejecutables ELF nativos para Linux.
- Rutinas autónomas en ensamblador que usan llamadas al sistema Linux para imprimir enteros, booleanos y literales de cadena.
- Llamadas a funciones basadas en la convención System V AMD64 (hasta 6 parámetros).
- Comprobación de tipos estables.
- Evaluación de constantes y optimizaciones locales sencillas.
- Comandos para inspeccionar la IR y el ensamblador.

## Requisitos

Ubuntu/Debian:

```bash
sudo apt update
sudo apt install -y curl build-essential binutils
```

Instala Rust si es necesario:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

También puedes ejecutar:

```bash
./scripts/bootstrap-ubuntu.sh
```

Comprueba las herramientas:

```bash
rustc --version
cargo --version
as --version
ld --version
```

## Compilar PyForge

```bash
sudo apt install -y llvm-21-dev libllvm21
LLVM_SYS_211_PREFIX=/usr/lib/llvm-21 \
  cargo build --release
```

`LLVM_SYS_211_PREFIX` debe señalar al prefijo que contiene `bin/llvm-config`.
LLVM se necesita al construir esa edición y `libLLVM.so.21.1` al ejecutarla. El
`.deb` distribuido declara `libllvm21` para que APT la resuelva.

Las dos herramientas son:

```text
target/release/pyforgec
target/release/pyforge
```

Opcionalmente:

```bash
./scripts/build-release.sh
```

Este script las copia a `dist/`.

## Inicio rápido: compilar un archivo Python

```bash
./target/release/pyforgec build examples/hello.py
```

Salida:

```text
dist/hello
```

Ejecútalo:

```bash
./dist/hello
```

Salida esperada (se conserva el texto literal del ejemplo):

```text
Hello from native PyForge
42
```

El binario generado se ejecuta directamente, sin usar `python examples/hello.py`.

## `pyforgec`: interfaz del compilador

Los comandos siguientes suponen que los binarios están en `PATH`. También puedes usar sus rutas `./target/release/pyforgec` y `./target/release/pyforge` desde la raíz de este repositorio.

### Comprobar el código fuente

```bash
pyforgec check app.py
```

Realiza el análisis sintáctico y semántico, la conversión a IR y la validación de la generación de código, sin enlazar un ejecutable.

### Inspeccionar la IR de PyForge

```bash
pyforgec emit-ir app.py
```

O bien:

```bash
pyforgec emit-ir app.py -o app.ir
```

### Generar ensamblador x86-64

```bash
pyforgec emit-asm app.py -o app.s
```

### Inspeccionar el backend LLVM 21

```bash
pyforgec check programa.st
pyforgec emit-llvm programa.st -o programa.ll
pyforgec emit-asm programa.st -o programa-llvm.s
pyforgec build programa.st -o programa
pyforgec run programa.st
```

LLVM valida el módulo, ejecuta `default<O2>` cuando la optimización está activa y
emite el ensamblador que usa el runtime ST existente. `--no-opt` permite comparar
la generación sin O2. No existe selector de backend: toda compilación usa LLVM.

Las excepciones actuales dependen de restaurar exactamente `rsp` y `rbp`. Mientras
esa ABI se migra a un mecanismo de excepciones propio de LLVM, los programas que
contienen `intentar`, `capturar`, `finalmente` o `lanzar` encapsulan el ensamblador
probado como ensamblador inline privado del módulo. Esa parte todavía no se
transforma a SSA ni se optimiza por LLVM. El runtime final sigue limitado a Linux
x86-64.

### Generar un ejecutable nativo

```bash
pyforgec build app.py
```

O elige la ruta de salida:

```bash
pyforgec build app.py -o my_program
```

Conserva los archivos intermedios `.s` y `.o`:

```bash
pyforgec build app.py --keep
```

Desactiva el optimizador actual:

```bash
pyforgec build app.py --no-opt
```

### Compilar y ejecutar

```bash
pyforgec run app.py
```

## `pyforge`: gestor de proyectos

Crea un proyecto:

```bash
pyforge new calculator
cd calculator
```

Los proyectos nuevos usan ST de forma predeterminada, con `entry = "src/main.st"` en `PyForge.toml`.

Estructura generada:

```text
calculator/
├── PyForge.toml
├── .gitignore
└── src/
    └── main.st
```

Después:

```bash
pyforge check
pyforge build
pyforge run
pyforge clean
```

Todos esos comandos usan LLVM. La opción antigua `--backend` se rechaza para evitar
que una orden parezca seleccionar un generador que ya no existe.

La salida optimizada se escribe en:

```text
target/release/<nombre-del-proyecto>
```

Con `--no-opt`:

```text
target/debug/<nombre-del-proyecto>
```

## Subconjunto de Python admitido

### Valores nativos

- `int` → entero con signo de 64 bits (almacenamiento como `i64`; aritmética con operaciones nativas de 64 bits).
- `bool` → valor nativo 0/1.
- **Literales** de cadena directamente dentro de `print()`.

Ejemplos:

```python
x: int = 10
enabled: bool = True
print(x)
print(enabled)
print("hello")
```

### Variables

```python
x = 10
y: int = 20
x += 5
x -= 2
x *= 4
x /= 2
x %= 7
```

Una variable debe mantener el mismo tipo durante todo el programa compilable.

### Aritmética entera

En esta versión, `/` realiza división entera; no produce el resultado de punto flotante de Python.

```text
+  -  *  /  %
```

### Comparaciones

```text
==  !=  <  <=  >  >=
```

### Expresiones booleanas

```text
and  or  not
```

`and` y `or` se convierten en instrucciones con evaluación en cortocircuito.

### Condiciones

```python
if x > 10:
    print(1)
elif x == 10:
    print(2)
else:
    print(3)
```

### Bucle while

```python
while x > 0:
    print(x)
    x -= 1
```

### Bucles for y range

```python
for i in range(10):
    print(i)
```

Y también:

```python
for i in range(10, 20):
    print(i)
```

`range(start, stop, step)` todavía no está implementado; v0.2 incrementa de uno en uno.

### Control de bucles

```text
break
continue
pass
```

### Funciones

Las funciones requieren anotaciones de tipo en sus parámetros:

```python
def add(a: int, b: int) -> int:
    return a + b

print(add(20, 22))
```

La ABI actual admite hasta seis argumentos de palabra; el frontend comprueba sus
tipos antes de construir la llamada LLVM.

## Ejemplo: bucle de cálculo en CPU

```python
def sum_to(n: int) -> int:
    total: int = 0
    for i in range(n):
        total += i
    return total

print(sum_to(100000000))
```

Compila:

```bash
pyforgec build examples/benchmark.py -o benchmark_native
```

Ejecuta:

```bash
time ./benchmark_native
```

En este tipo de bucle con enteros de tipo estático, el programa generado opera con enteros nativos de la CPU en lugar de crear un objeto entero de CPython para cada operación.

## Ensamblador generado

Para una función conceptualmente como esta:

```python
def add(a: int, b: int) -> int:
    return a + b
```

LLVM produce operaciones x86-64 con una forma general como esta:

```asm
movq %rsi, %rbx
movq %rdi, %r14
callq pf_llvm_stack_check@PLT
movq %r14, %rdi
movq %rbx, %rsi
callq pf_llvm_add@PLT
retq
```

Puedes inspeccionar la salida exacta con:

```bash
pyforgec emit-asm examples/functions.py
```

## Modelo de ejecución

Los ejecutables generados contienen pequeñas rutinas de ejecución escritas en ensamblador x86-64. Proporcionan:

- Punto de entrada del proceso `_start`
- Impresión de enteros con signo
- Impresión de booleanos
- Impresión de literales de cadena
- Llamadas al sistema Linux `write` y `exit`

Para los ejemplos admitidos, el ejecutable **no necesita CPython ni libc durante su ejecución**.

## Por qué se siguen usando `as` y `ld`

PyForge realiza la compilación del lenguaje. GNU `as` convierte el ensamblador x86-64 generado en secciones de código de máquina; GNU `ld` las enlaza en un ejecutable ELF.

Esto equivale a tener etapas separadas de ensamblado y enlazado. No convierte a PyForge en un transpilador.

Una versión futura puede sustituir estas herramientas externas por:

1. Emisión de objetos directamente desde LLVM.
2. Enlace mediante LLD con una configuración reproducible.

Las etapas quedarían así:

```text
.st → AST → IR ST → LLVM IR → objeto → enlazador → ejecutable
```

## Optimizador actual

PyForge aplica primero optimizaciones semánticas sencillas a su IR:

- Evaluación de constantes
- Comparaciones entre constantes
- Eliminación de una constante seguida de `Pop`

Ejemplo:

```python
x = 10 + 20 * 2
```

Las partes formadas exclusivamente por constantes pueden simplificarse antes de
generar LLVM IR. Después, LLVM ejecuta `default<O2>` salvo que se use `--no-opt`.

Las optimizaciones futuras incluyen:

- Más información de alias y rangos para la IR LLVM
- Propagación de constantes
- Eliminación de código muerto
- Eliminación local de subexpresiones comunes
- Expansión de funciones en el punto de llamada
- Movimiento de operaciones invariantes fuera de bucles
- Sustitución de operaciones costosas por otras equivalentes más simples
- Perfiles para guiar optimizaciones LLVM
- Eliminación de comprobaciones de límites para futuros contenedores
- SIMD y vectorización

## Limitaciones actuales importantes

PyForge v0.2 todavía **no** admite toda la semántica de Python. Estas características no están disponibles o están incompletas:

- Números de punto flotante
- Enteros Python de precisión arbitraria
- Listas, diccionarios, conjuntos y tuplas
- Variables de cadena y operaciones generales con cadenas
- Clases y objetos
- Excepciones
- Generadores y programación asíncrona
- Decoradores
- Importaciones y `pip`
- ABI de extensiones C de Python
- `eval`/`exec`
- Reflexión y modificación dinámica de objetos
- Argumentos por nombre, predeterminados o de cantidad variable
- Funciones anidadas y cierres léxicos
- Generadores de código para Windows PE64 y ARM64

No se espera que paquetes arbitrarios de Python puedan compilarse en esta versión.

## Organización del código del compilador

```text
src/
├── ast.rs             Definiciones del AST
├── lexer.rs           Analizador léxico de expresiones
├── parser.rs          Analizador sintáctico basado en sangría
├── semantic.rs        Tipos, símbolos y comprobación de funciones
├── ir.rs              IR de pila de PyForge
├── lower.rs           Conversión del AST a IR
├── optimizer.rs       Optimizaciones de la IR
├── codegen/
│   ├── mod.rs
│   ├── llvm.rs        Lowering obligatorio a LLVM 21 mediante Inkwell
│   └── llvm_compat.rs Runtime Linux y transición privada de excepciones
├── driver.rs          Coordinación de la compilación
├── project.rs         Gestión de proyectos PyForge.toml
└── bin/
    ├── pyforgec.rs    Comando del compilador
    └── pyforge.rs     Comando de proyectos similar a Cargo
```

Consulta también [la arquitectura](docs/README.md).

## Pruebas

Después de compilar:

```bash
./tests/smoke.sh
```

Las pruebas compilan y ejecutan varios ejemplos `.py` como archivos ELF nativos.
La cobertura del backend LLVM se ejecuta así:

```bash
LLVM_SYS_211_PREFIX=/usr/lib/llvm-21 cargo test --all-targets
LLVM_SYS_211_PREFIX=/usr/lib/llvm-21 cargo build
python3 scripts/verify-projects.py --compiler target/debug/pyforgec
```

## Hoja de ruta

### v0.3 — corrección del lenguaje y mejoras de la IR

- Bloques básicos explícitos
- Valores similares a SSA
- Diagnósticos semánticos y de tipos más precisos
- Ubicaciones del código fuente en todos los errores
- Diagnósticos LLVM vinculados a posiciones de ST
- ABI de errores compatible con optimización LLVM
- Comprobaciones de división por cero
- Configuración del comportamiento ante desbordamientos

### v0.4 — objetos y enlace LLVM

- Emisión de objetos con `TargetMachine`
- Integración reproducible con LLD
- Reubicaciones y símbolos por plataforma
- Información de depuración para ST

### v0.5 — plataformas nativas adicionales

- Runtime y ABI AArch64
- Ejecutables Windows nativos
- Matrices de pruebas e instaladores por destino

### v0.6 — más valores de Python

- Números de punto flotante
- Cadenas nativas
- Listas y tuplas
- Asignador de memoria dinámica durante la ejecución

### v0.7+

- Módulos e importaciones
- Gestor de paquetes y archivo de bloqueo de versiones
- Compilación incremental y caché
- Generador PE para Windows x86-64
- Generador de código ARM64
- Entorno opcional de compatibilidad para características dinámicas de Python

## Filosofía

El objetivo es construir ST como lenguaje AOT propio, con gramática en español,
tipos comprobables y generación LLVM. El frontend Python se mantiene únicamente
como compatibilidad beta; no se afirma que paquetes Python arbitrarios sean ST ni
que alcancen automáticamente el rendimiento de Rust.

Este enfoque mantiene las expectativas técnicas claras y permite avanzar hacia una arquitectura de compilación sólida.

## Licencia

MIT. Consulta [LICENSE](LICENSE).

Consulta [cadenas y flotantes nativos](docs/MANUAL_ST.md) y el ejemplo `examples/cadenas.st` para declarar e imprimir los cuatro tipos escalares.

Consulta [las comprobaciones de seguridad y sus límites](docs/README.md): aritmética comprobada, verificación de flujo y presupuestos de memoria/pila.

## Instaladores y actualizaciones

Consulta [Instalación y empaquetado](docs/INSTALACION.md). `python3 scripts/package.py` genera el `.deb` y el instalador `.exe` para Windows con WSL. `python3 scripts/package.py --watch` los regenera al modificar las fuentes o recursos. Los archivos ya instalados se actualizan ejecutando el nuevo paquete.

## Web de descargas y curso

`python3 scripts/build-site.py` genera `dist/site/` con las descargas actuales, el curso basado en el manual y los ejemplos. Añade `--watch` para regenerar al guardar cambios. Consulta [la guía de la página web](docs/INSTALACION.md) para vista previa, sincronización y despliegue.

### Integraciones opcionales experimentales

Se añadieron módulos de archivos, SQLite, documentos, redes, cifrado, procesos y GUI,
con permisos propios denegados por defecto. Utilizan Python 3.11 como auxiliar;
las bibliotecas externas son opcionales. VPN solo incluye detección e inspección.
No se afirma cobertura completa ni aislamiento equivalente a Rust.

Consulta la [guía y límites](docs/INTEGRACIONES.md), la [API comentada](docs/INTEGRACIONES.md),
los [12 proyectos](examples/integraciones/README.md) y el [informe de validación](docs/README.md).
El IDE incorpora autocierre configurable, consulta de permisos y enlace de apoyo.

Los instaladores también registran el [logo de la web como icono de archivos `.st`](docs/INSTALACION.md).
Se conserva cualquier editor predeterminado existente. Abrir una fuente en ST IDE
la selecciona para editar sin ejecutarla automáticamente.

## 20 aplicaciones y juegos adicionales

Consulta el [catálogo de proyectos avanzados](examples/proyectos_avanzados/README.md) y la [guía de ejecución](docs/MANUAL_ST.md). Incluyen seis juegos, caja, inventario, informes, cifrado, redes y un panel gráfico, con comentarios en español y pruebas reproducibles.

## Sitios web, foro y proyectos cifrados

`pyforge new mi_web --web` genera fuentes ST, vistas HTML, CSS/JS y un backend Django con vista ST, capturas por sesión, culebrita y SQLite. `pyforge web` lo compila y ejecuta localmente. Consulta [INTEGRACIONES.md](docs/INTEGRACIONES.md) para dependencias, arquitectura y despliegue HTTPS.

El menú del IDE permite crear o abrir proyectos cifrados `.stvault`, además del modo normal sin contraseña. Los temas disponibles son Normal, Hacker y Pro oscuro. Consulta [INSTALACION.md](docs/INSTALACION.md) para conocer el alcance del cifrado y sus límites.

El foro oficial está en [website/backend](website/backend/README.md). La base Rust independiente se documenta en [INTEGRACIONES.md](docs/INTEGRACIONES.md).

## Motor 2D y revisión

Consulta [la guía de correcciones, juego ejecutable, extensión y paquetes](docs/README.md).
