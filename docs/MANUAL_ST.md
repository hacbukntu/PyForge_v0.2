# Manual unificado de ST y PyForge

Documentación consolidada de la versión 0.2.0. Versión del proyecto: 0.2.0 experimental.

Las otras tres guías son [instalación e IDE](INSTALACION.md), [integraciones y web](INTEGRACIONES.md) y [arquitectura y seguridad](README.md).

Este archivo reúne cómo escribir programas, compilar, usar el IDE, entender el compilador y ampliar el lenguaje. Los ejemplos marcados como ST son programas ejecutables independientes, salvo los dos archivos del ejemplo de módulos. Las limitaciones se indican explícitamente.

**ST todavía no es un lenguaje completo equivalente a Python.** Tiene un núcleo nativo funcional, con clases nominales, excepciones explícitas, listas mixtas y colecciones tipadas anidadas; el registro público y la resolución de versiones siguen pendientes. Hay entrada por líneas y lectura/escritura básica de archivos. Este manual documenta capacidades reales; no implementa las pendientes por describirlas.

## 1. Preparar el compilador

Esta lección empieza después de instalar PyForge con el paquete `.deb` en Linux o el instalador `.exe` para Windows con WSL. Aprenderás a comprobar la instalación, crear un proyecto y ejecutar tu primer archivo ST.

### Comprobar la instalación

Abre una terminal nueva para que reconozca los comandos añadidos a PATH. En Linux usa tu terminal habitual; en Windows puedes usar PowerShell o la terminal de Windows, con WSL ya configurado.

```bash
pyforge --version
pyforgec --version
```

Ambos comandos deben mostrar una versión. Si aparece «comando no encontrado», cierra la terminal y abre otra. Si continúa, revisa la instalación y el PATH siguiendo la guía de instalación. En Windows, comprueba también que tu distribución WSL predeterminada tenga las dependencias indicadas por el instalador.

### Crear tu primer proyecto

Elige una carpeta donde guardar tus programas y ejecuta:

```bash
pyforge new mi_proyecto
cd mi_proyecto
pyforge run
```

`pyforge new` crea el proyecto con un archivo de entrada `src/main.st` y su configuración `PyForge.toml`. `pyforge run` compila y ejecuta ese archivo. El paquete instalado contiene las herramientas necesarias de PyForge; no necesitas descargar su repositorio ni compilar el compilador con Rust/Cargo para seguir el curso.

### Editar y ejecutar desde el IDE

Desde la carpeta del proyecto:

```bash
pyforge-ide .
```

Abre en el navegador la dirección local que muestra la terminal. En el explorador selecciona `src/main.st`, escribe el siguiente programa y guárdalo:

```st
variable nombre: cadena = "Ana";
mostrar("Hola, " + nombre);
```

Pulsa **Ejecutar** o F5. El panel de salida mostrará `Hola, Ana`. Puedes modificar el nombre y ejecutarlo de nuevo. Conserva abierta la terminal que inició el IDE; Ctrl+C cierra el servidor local.

### Comandos que utilizarás en el curso

| Comando | Para qué sirve |
| --- | --- |
| `pyforge check` | Comprueba el proyecto actual |
| `pyforge build` | Compila el proyecto actual |
| `pyforge run` | Compila y ejecuta su archivo de entrada |
| `pyforge-ide .` | Abre el IDE para el proyecto actual |
| `pyforgec check src/main.st` | Comprueba un archivo concreto |
| `pyforgec run src/main.st` | Compila y ejecuta ese archivo |
| `pyforgec build src/main.st -o programa` | Guarda el ejecutable con el nombre indicado |

Los ejecutables generados son Linux x86-64. En Windows, los comandos del instalador los ejecutan mediante WSL. El soporte de archivos `.py` es beta, experimental e incompleto; este curso utiliza principalmente `.st`.

Para instalar, actualizar o resolver problemas del entorno, consulta la [guía de instalación](INSTALACION.md).

## 2. Sintaxis fundamental

- Los archivos ST terminan en `.st`; distinguen mayúsculas y minúsculas.
- Los bloques usan `{ ... }`. La sangría mejora la lectura, pero no determina bloques.
- Las sentencias simples terminan con `;`.
- `//` y `#` inician comentarios hasta el fin de línea. `/* ... */` y `""" ... """` delimitan comentarios de una o varias líneas.
- Los identificadores usan letras ASCII, números y `_`, sin empezar por un número. Las cadenas sí admiten Unicode.
- Las palabras reservadas no pueden usarse como nombres. También se restringen nombres internos como `print` y los que empiezan por `__` en ST.
- `variable` declara e inicializa un nombre nuevo; una asignación sin `variable` solo puede modificar un nombre que ya exista y conserve su tipo.

```st
// Primer programa: declaración, cálculo e impresión.
variable respuesta: entero = 40 + 2;
mostrar(respuesta); // 42
# También se admite este estilo de comentario.
```

### Comentarios de varias líneas

Un comentario sirve para explicar código o dejar notas; el compilador ignora su contenido. Se admiten estos dos formatos:

```st
/*
   Este comentario ocupa varias líneas.
   Puedes explicar aquí para qué sirve el programa.
*/
variable nombre: cadena = "Ana";

"""
Este es otro comentario de varias líneas.
mostrar("Esto no se ejecuta");
"""
mostrar(nombre);
```

También se pueden insertar entre tokens: `mostrar(10 /* divisor */ / 2);` imprime `5`. No unen fragmentos de palabras: `var/* nota */iable` no equivale a `variable`.

- `/*` termina en el primer `*/`; `"""` termina en las siguientes `"""`.
- No hay anidamiento. Un delimitador del otro formato dentro del comentario es texto sin significado especial.
- Dentro del comentario no se interpretan comillas simples, dobles ni escapes.
- Dentro de una cadena normal, los delimitadores son texto: `mostrar("/* texto */");` los imprime. Para imprimir tres comillas dobles puedes usar una cadena con comillas simples: `mostrar('"""');`.
- Si falta el cierre, el diagnóstico indica la línea y columna de apertura. Los saltos de línea del comentario se contabilizan para los errores posteriores.
- Las comillas triples no producen un valor: `variable texto = """nota""";` no es una asignación válida. Para cadenas con saltos usa `\n` dentro de una cadena normal.

**En el IDE:** escribe `/*` o `"""` en un archivo `.st`. Aparece una sugerencia de comentario multilínea; acepta con Enter o Tab. Se inserta el cierre y el cursor queda en la línea interior, conservando la sangría. También puedes solicitar la sugerencia con Ctrl+Espacio después del delimitador. Mientras escribes dentro de un comentario no aparecen sugerencias de código. Un bloque sin cerrar se colorea hasta el final del archivo para que sea visible lo que falta cerrar.

Practica ambos formatos con el ejemplo completo `examples/comentarios.st`.

## 3. Tipos y variables

| Tipo | Representación y uso |
| --- | --- |
| `entero` | i64, con operaciones comprobadas contra desbordamiento |
| `flotante` | f64 IEEE-754, como `12.5` o `1e3` |
| `booleano` | `verdadero`/`falso`; se imprime `True`/`False` |
| `cadena` | Texto UTF-8 inmutable |
| `lista` | Secuencia mutable de enteros |
| `diccionario` | Claves cadena calculadas en ejecución y valores enteros |
| `tupla` | Secuencia inmutable de enteros |
| `conjunto` | Colección mutable de enteros únicos |
| `nulo` | Valor almacenable que representa ausencia de dato |
| `nada` | Ausencia de retorno de una función; no es un tipo de variable |

Las anotaciones de variables son opcionales. El tipo inferido permanece estable: una variable entera no puede recibir después una cadena. No hay promoción numérica implícita: escribe `0.0` para una variable flotante.

```st
variable nombre: cadena = "Ana";
variable edad: entero = 20;
variable precio: flotante = 12.5;
variable activo: booleano = verdadero;
variable ausencia: nulo = nulo;
mostrar(nombre);
mostrar(edad);
mostrar(precio);
mostrar(activo);
mostrar(ausencia);
// Salida: Ana, 20, 12.5, True, nulo, cada valor en una línea.
```

Una variable usada después de una condición debe declararse antes y asignarse donde corresponda. Los nombres creados solo dentro de ramas no se exportan de forma general. El verificador rechaza lecturas de variables que podrían no inicializarse, por ejemplo tras un bucle vacío. Las funciones usan parámetros y locales; no acceden directamente a variables del bloque principal.

## 4. Operadores

De mayor a menor precedencia: signo unario `+`/`-`; `* / %`; `+ -`; comparaciones e identidad; `no`; `y`; `o`. Los paréntesis permiten agrupar. No hay comparaciones encadenadas: escribe `1 < x y x < 10`.

| Operación | Tipos y reglas |
| --- | --- |
| `+ - * / %` | Enteros; división trunca hacia cero, resto sigue el signo del dividendo |
| `+ - * /` | Dos flotantes; `%` no está definido para ellos |
| `+` con cadena | Concatena; convierte el otro operando si es un escalar |
| `+= -= *= /= %=` | Asignación aumentada compatible con el tipo; cadena solo admite `+=` |
| `== !=` | Escalares del mismo tipo; cadenas por contenido. En colecciones, la implementación compara referencias |
| `< <= > >=` | Dos enteros o dos flotantes |
| `y o no` | Enteros/booleanos; resultado booleano; `y` y `o` tienen cortocircuito |
| `es` | Identidad entre colecciones del mismo tipo o dos valores nulos |

No hay todavía igualdad estructural de colecciones. Para dejar claro que comparas alias usa `es`.

```st
variable total = 10;
total += 5;
mostrar(total / 4);             // 3
mostrar(-7 % 3);                // -1
mostrar(2.5 * 2.0);             // 5.0
mostrar(total > 10 y verdadero); // True
mostrar("total: " + total);     // total: 15
```

El mínimo i64 puede expresarse como `-9223372036854775807 - 1`: el literal positivo `9223372036854775808` no cabe en el analizador. Los flotantes se imprimen con hasta seis decimales, al menos uno, y notación científica para magnitudes no nulas menores que `1e-6` o mayores o iguales que `1e12`. Esta representación no garantiza recuperar todos los bits al leerla otra vez. Operaciones f64 pueden producir `inf`, `-inf` y `nan`.

## 5. Condiciones y bucles

```st
variable edad = 20;
si edad < 18 {
    mostrar("menor");
} sino_si edad < 65 {
    mostrar("adulto");
} sino {
    mostrar("mayor");
}

// También existe la grafía sino si, por compatibilidad.
variable contador = 0;
mientras contador < 3 {
    mostrar(contador);
    contador += 1;
}

variable suma = 0;
para i en rango(0, 5) {
    si i == 2 { continuar; }
    suma += i;
}
mostrar(suma); // 8

mientras verdadero {
    romper;
}
pasar; // No realiza ninguna operación.
```

`rango(fin)` comienza en cero; `rango(inicio, fin)` excluye `fin`. Solo incrementa en uno; no admite paso negativo ni tercer argumento. Actualmente el límite final se reevalúa en cada vuelta y la variable del bucle se incrementa después del cuerpo. Evita modificarla o usar límites con efectos si necesitas un recorrido fijo. No hay todavía `para elemento en lista`.

## 6. Funciones

```st
// Las firmas permiten comprobar tipos y llamadas antes de generar instrucciones.
funcion saludar(persona: cadena) -> cadena {
    retornar "Hola, " + persona;
}
funcion doble(valor: flotante) -> flotante {
    retornar valor * 2.0;
}
funcion aviso() -> nada {
    mostrar("listo");
    retornar;
}
mostrar(saludar("Ana")); // Hola, Ana
mostrar(doble(1.25));    // 2.5
aviso();
```

Los parámetros requieren anotación. Se admiten hasta seis argumentos, funciones definidas más adelante y recursión dentro del presupuesto de pila. Omitir el tipo de retorno equivale a `nada`. Toda ruta alcanzable de una función que devuelve un valor debe retornar. No hay funciones anidadas, argumentos opcionales, cierres, argumentos por nombre ni funciones como valores.

## 7. Cadenas completas en el núcleo actual

Comillas simples o dobles, con escapes `\n`, `\r`, `\t`, `\\`, `\'` y `\"`. Los saltos de línea literales requieren escapes; las comillas triples delimitan comentarios ST, no cadenas de texto.

```st
variable texto = "añ😀z";
mostrar(longitud(texto));             // 4 puntos de código Unicode
mostrar(texto[1]);                    // ñ
mostrar(texto[-2]);                   // 😀
mostrar(buscar(texto, "😀"));          // 2
mostrar(buscar(texto, "ausente"));    // -1
mostrar(reemplazar(texto, "ñ😀", "!")); // a!z
mostrar("edad: " + 20);               // edad: 20
mostrar(cadena(verdadero));           // True
mostrar(interpolar("{} cuesta {}", "pan", 1.5)); // pan cuesta 1.5
mostrar(interpolar("{{valor}}: {}", 42));         // {valor}: 42
```

- Índices y búsqueda cuentan puntos de código, no bytes ni grafemas. Índices negativos cuentan desde el final. Los índices inválidos terminan con diagnóstico.
- Las cadenas son inmutables: no puedes asignar `texto[0]`. `+=` crea otra cadena y reasigna la variable.
- `buscar` distingue mayúsculas, devuelve la primera coincidencia o -1; buscar una cadena vacía devuelve 0.
- `reemplazar` sustituye todas las coincidencias no superpuestas. Un reemplazo vacío elimina coincidencias; un patrón vacío produce error. No usa expresiones regulares.
- `cadena(valor)` convierte escalares con el formato de impresión, sin salto de línea. No convierte colecciones.
- `interpolar` necesita una plantilla literal con `{}`. Los argumentos se evalúan una vez, en orden. `{{`/`}}` producen llaves; los marcadores con nombre no existen aún.
- La conversión implícita solo se aplica a concatenación y `+=` de cadena; no cambia las reglas de asignación ni la aritmética numérica.

## 8. Listas, tuplas y conjuntos

```st
variable numeros: lista = [10, 20];
numeros.agregar(30);
agregar(numeros, 40);
numeros[1] = 21;
mostrar(numeros[-1]);       // 40
mostrar(longitud(numeros)); // 4

// Copiar una referencia no copia el contenido de la colección.
variable alias = numeros;
alias[0] = 99;
mostrar(numeros[0]);        // 99
mostrar(alias es numeros); // True

// Una lista mixta conserva su orden y permite cambiar el tipo de una posición.
variable ficha = ["Ada"; 36; 1.65; verdadero; nulo];
mostrar(ficha[0]);           // Ada
mostrar(tipo(ficha[1]));     // entero
ficha[1] = "treinta y seis";
ficha.agregar(2026);
mostrar(ficha[-1]);          // 2026

variable punto: tupla = tupla(10, 20);
mostrar(punto[1]);          // 20
variable unicos: conjunto = conjunto(1, 1, 2);
unicos.agregar(3);
mostrar(longitud(unicos));  // 3
```

`[]`, `tupla()` y `conjunto()` crean colecciones vacías. Las tuplas conservan orden y permiten lectura por índice, pero rechazan asignación y `agregar`. Los conjuntos eliminan duplicados y no permiten indexación. Sus búsquedas son lineales.

Las listas son **ordenadas y mutables**. Un literal con tipos diferentes infiere `lista[mixto]`; cada posición guarda una etiqueta de tipo junto a su valor. `mostrar`, `tipo`, `cadena`, `entero`, `flotante` y `booleano` comprueban esa etiqueta al recuperar un elemento. Para una lista mixta inicialmente vacía escribe `variable datos: lista[mixto] = [];`. Las listas con un solo tipo conservan sus comprobaciones estáticas y los diccionarios siguen siendo homogéneos. Tuplas y conjuntos conservan elementos enteros. Todavía no hay borrado, rebanadas ni impresión completa de una colección. La asignación aumentada de un índice no se admite; usa `numeros[0] = numeros[0] + 1`.

## 9. Diccionarios con claves dinámicas

```st
variable clave: cadena = "ed" + "ad";
variable persona: diccionario = {clave: 20};
persona["edad"] = 21;
persona["puntos"] = 100;
mostrar(persona[clave]);     // 21
mostrar(longitud(persona)); // 2
```

Las claves deben ser cadenas, pero pueden ser variables, concatenaciones o llamadas. Se comparan por contenido, no por dirección. Una clave repetida reemplaza el valor sin aumentar la longitud. Las claves ausentes terminan con diagnóstico. Los valores siguen siendo enteros; no hay borrado ni iteración de pares. El almacenamiento actual es una búsqueda lineal, no una tabla hash.

## 10. Módulos

Archivo `math.st`:

```st-module-math
// Las funciones importadas comparten el espacio de nombres del programa.
funcion cuadrado(n: entero) -> entero {
    retornar n * n;
}
```

Archivo `main.st`, al lado del anterior:

```st-module-main
importar "math.st";
mostrar(cuadrado(6)); // 36
```

Compila `main.st`. Las importaciones deben aparecer al principio y usan rutas relativas al archivo importador. Solo se importan módulos `.st`; se rechazan rutas absolutas y ciclos. Cada ruta canónica se carga una vez; las dependencias se inicializan antes del importador.

Todas las funciones comparten nombres: las duplicadas son errores. No existen aún alias, `desde`, exportaciones, paquetes versionados ni llamadas `math.cuadrado`. Las variables superiores se combinan en el bloque principal; las funciones siguen sin acceso directo a ellas.

## 11. Ejemplo integrado: calcular una compra

```st
// Función con retorno en ambos caminos; no usa entrada interactiva todavía.
funcion descuento(total: flotante) -> flotante {
    si total >= 100.0 {
        retornar total * 0.9;
    } sino {
        retornar total;
    }
}

variable cliente = "Ana";
variable unidades = [2, 3, 1];
variable cantidad = 0;
para i en rango(longitud(unidades)) {
    cantidad += unidades[i];
}
variable importes = {"productos": cantidad};
variable total: flotante = descuento(125.0);
mostrar(interpolar("Cliente: {}, productos: {}, total: {}", cliente, importes["productos"], total));
// Cliente: Ana, productos: 6, total: 112.5
```

## 12. IDE

Inicia `pyforge-ide .`desde la terminal de Linux o PowerShell/CMD y abre `http://127.0.0.1:8765`. Para otro proyecto, pasa su ruta como argumento. 

tendras un ST IDE, desde tu navegador favorito. 


### Ocultar bloques largos en el IDE

Usa **▾** junto al número de línea para plegar funciones, clases, condiciones, bucles y comentarios multilínea ST. **▸** vuelve a mostrarlos; **Desplegar todo** abre todos los bloques. Python dispone de plegado conservador por indentación. Los números de línea mantienen su valor original y los bloques pueden plegarse de forma anidada.

El código oculto permanece en el documento: **Guardar** y **Ejecutar** incluyen todas sus líneas. Puedes seguir editando fuera de los bloques plegados. Ctrl+Z deshace y Ctrl+Y o Ctrl+Mayús+Z rehace hasta 50 ediciones. El estado de plegado se reinicia al abrir otro archivo.

### Buscar, reemplazar y comprobar

Pulsa **Buscar** o Ctrl+F para localizar texto desde la posición actual; Enter avanza, Mayús+Enter retrocede y la búsqueda vuelve al principio al llegar al final. Ctrl+H abre también el campo de reemplazo. **Reemplazar** cambia la coincidencia seleccionada y **Todo** aplica todos los cambios como una sola edición que puedes deshacer.

**Comprobar** o Ctrl+Mayús+B guarda el archivo y analiza su sintaxis y tipos sin ejecutar el programa. Los diagnósticos aparecen en la salida. F5 continúa siendo la acción para guardar, compilar y ejecutar.

### Organizar los paneles

El atajo **Ctrl+B** oculta y muestra el explorador sin cerrar el archivo ni descartar cambios. Arrastra el borde superior de la terminal hacia arriba para ampliarla y hacia abajo para reducirla. El separador también admite foco de teclado: flecha arriba/abajo ajusta la altura y Home/End selecciona el mínimo/máximo permitido. Siempre se reserva espacio para el editor.

## 13. Diagnósticos, memoria y seguridad

Los errores léxicos/sintácticos ST incluyen línea, columna, fragmento y cursor; los sintácticos indican el token encontrado. El cargador añade el archivo. Los errores posteriores identifican su etapa; aún no todos tienen ubicación exacta.

Los enteros tienen comprobaciones de desbordamiento, división/resto por cero y el caso mínimo i64 dividido entre -1. Los fallos terminan con estado 1, sin envolver silenciosamente. Esto se conserva con y sin optimización.

La IR verifica variables definitivamente inicializadas, retornos y profundidad de pila compatible entre ramas. El código generado comprueba índices y claves, mantiene referencias estables cuando crecen colecciones y usa presupuestos de 256 MiB de reservas del runtime y 512 KiB de pila generada.

Estos presupuestos no equivalen a garantizar todo el consumo del proceso: excluyen memoria del compilador, datos estáticos y efectos externos. El límite de pila presupone el entorno Linux habitual. Cadenas y cabeceras viven hasta finalizar; no hay recolección de basura. La concatenación repetida puede alcanzar el límite de memoria.

**No hay equivalencia demostrada con la seguridad de Rust**, comprobador de préstamos ni auditoría formal del ensamblador. Los fallos internos de aritmética, índices, memoria y entrada/salida abortan. Las excepciones explícitas creadas con `lanzar` sí se capturan (sección 20). No se garantiza protección total frente a fuentes patológicas, errores del compilador o restricciones particulares del sistema.

## 14. Cómo está conectado el código

```text
archivo ST → cargador de módulos → lexer ST → parser ST ┐
                                                       ├→ AST → semántica → IR
archivo Python → lexer/parser Python ──────────────────┘
→ optimización opcional → verificación de flujo → ensamblador → as → ld
```

| Archivo | Responsabilidad y funciones a estudiar |
| --- | --- |
| `src/st/lexer.rs` | `KEYWORDS` y `keyword` clasifican palabras; `next_kind` consume literales y símbolos |
| `src/st/parser.rs` | `parse_module` lee módulos; `statement`, `expression`, `primary`, `atom` construyen el AST; `interpolation` expande plantillas |
| `src/st/modules.rs` | `load`/`visit` resuelven dependencias, ciclos y carga única |
| `src/st/diagnostic.rs` | `render` presenta posiciones y contexto del error |
| `src/ast.rs` | `TypeName`, `Expr` y `Stmt` representan tipos, expresiones y sentencias |
| `src/semantic.rs` | `analyze`, `check_block`, `infer_expr` y `check_index` validan nombres, tipos y usos |
| `src/lower.rs` | `Lowerer::stmt`/`expr` traducen a pila y saltos; `text_conversion` y `float_op` seleccionan operaciones |
| `src/ir.rs` | `Instr`, `CollectionOp` y `ScalarOp` definen operaciones y rutinas nativas |
| `src/optimizer.rs` | `fold` simplifica sin ocultar fallos aritméticos |
| `src/verify.rs` | `verify` comprueba flujo, inicialización y equilibrio de pila |
| `src/codegen/llvm.rs` | `lower_module`/`lower_function` generan y verifican LLVM IR |
| `src/codegen/llvm_compat.rs` | Runtime Linux y puente privado de excepciones aún no migradas |
| `src/codegen/collections.s` | Reservas, crecimiento, índices y contabilidad de memoria |
| `src/codegen/scalars.s` | Cadenas, conversiones, flotantes y claves textuales |
| `src/driver.rs` | `compile_file` selecciona lenguaje; `build_executable` ensambla y enlaza |
| `src/project.rs` | Proyectos, manifiesto y archivo de entrada |
| `src/bin/pyforge*.rs` | Interfaces de línea de comandos |
| `ide/server.py` | Archivos, API local, papelera y ejecución de procesos |
| `ide/app.js` | Editor, sugerencias, sincronización y controles |
| `ide/language.py`, `ide/python_language.py` | Vocabulario ST y reconocimiento léxico Python |

Una expresión normalmente apila una palabra de 64 bits. Los enteros contienen el valor, los flotantes sus bits y las cadenas/colecciones una referencia. El tipo semántico determina cómo interpretarla. Los descriptores de texto contienen longitud y bytes UTF-8; las colecciones guardan longitud, capacidad y búfer. No son objetos genéricos capaces de contener cualquier tipo.

## 15. Cambiar sintaxis o añadir capacidades

Para dar otro nombre a una operación existente, añade un alias a `KEYWORDS`, por ejemplo `("imprimir", Keyword::Mostrar)`. Conserva `mostrar` para no romper programas anteriores. Reconstruye el compilador. Añadir un nombre para una operación inexistente no implementa su comportamiento.

Para una capacidad nueva:

1. Define sintaxis, tipos, orden de evaluación y comportamiento ante errores.
2. Añade tokens y reglas al parser; representa el significado en el AST.
3. Valida tipos y operaciones en semántica.
4. Traduce a IR indicando cuántos valores consume y produce.
5. Implementa las rutinas nativas y verifica registros, pila, memoria y límites.
6. Actualiza el verificador/optimizador si introduces instrucciones nuevas.
7. Añade ejemplos y pruebas de éxito/error con y sin optimización, incluyendo regresiones Python.
8. Documenta el comportamiento real y actualiza plantillas del IDE si corresponde.

No todos los cambios requieren palabras reservadas: muchas capacidades de biblioteca deberían ser funciones ordinarias cuando exista el soporte de sistema necesario.

## 16. Qué falta para un lenguaje de propósito general más completo

| Área pendiente | Trabajo necesario |
| --- | --- |
| Diccionarios mixtos y genéricos | Extender las etiquetas de las listas a valores de diccionario, iteradores y APIs genéricas |
| Entrada y archivos avanzada | Flujos persistentes, modos de apertura, binarios y errores recuperables; la API de texto básica ya existe |
| Excepciones avanzadas | Jerarquías y recuperación de fallos internos; las excepciones explícitas de cadenas ya funcionan |
| Objetos avanzados | Herencia, interfaces, genéricos y visibilidad; campos tipados y métodos ya funcionan |
| Patrones avanzados | Capturas, desestructuración, guardas y exhaustividad; los patrones literales ya funcionan |
| Paquetes | Versionado, visibilidad, tipos importados en anotaciones y carga dinámica; alias y selección de funciones/constructores ya funcionan |
| Biblioteca estándar avanzada | Ampliar cobertura y APIs asíncronas; red, documentos y subprocesos ya existen como integraciones opcionales |
| Web | Completar el DSL Django nativo; el puente Django Python con vistas ST ya funciona |
| Herramientas | Diagnósticos con ubicaciones completas, depuración, formato y análisis del IDE |
| Memoria de larga duración | Política de liberación o recolección y pruebas de estrés |

La tabla distingue lo pendiente de las bases implementadas. Para aplicaciones HTTP usa el puente Django de la sección 28; `examples/pagina.st` solo genera HTML estático. La compatibilidad con todo Python es un proyecto distinto de ampliar ST.

## 17. Revisión y pruebas

```bash
cargo test
bash tests/smoke.sh
python3 -m unittest discover -s ide -q
cargo doc --no-deps
```

La revisión cubre vocabulario y gramática ST, tipos, generación, diagnósticos, límites, comandos y las pruebas de API del IDE. No es una auditoría formal ni una comprobación visual del navegador. Las pruebas no convierten las capacidades pendientes en disponibles.

Este manual es la referencia unificada de uso. Los documentos anteriores se conservan para no eliminar información o enlaces existentes, pero algunas secciones describen etapas históricas y pueden mencionar restricciones ya superadas.

Ejecuta las pruebas de la versión actual siguiendo [arquitectura y verificación](README.md); los conteos históricos no sustituyen una nueva ejecución.


## 18. Ampliación: colecciones tipadas y entrada/salida

Esta sección actualiza las restricciones históricas de colecciones de enteros. Las listas admiten elementos homogéneos tipados o elementos de tipos distintos mediante `lista[mixto]`. Los diccionarios admiten valores homogéneos de tipo `entero`, `flotante`, `booleano`, `cadena`, `nulo` o colecciones compatibles. Tuplas y conjuntos mantienen elementos enteros. Consulta también la sección 31.

```text
variable nombres: lista[cadena] = [];
nombres.agregar("Ana");
mostrar(nombres[0]);
variable precios: diccionario[flotante] = {};
precios["pan"] = 1.5;
mostrar(precios["pan"]);
variable banderas = [verdadero, falso]; // Inferencia: lista[booleano].
variable perfil = ["Ana"; 27; verdadero]; // Inferencia: lista[mixto].
perfil[1] = 28;
perfil[2] = "activo"; // Una posición mixta puede cambiar de tipo.
variable vacia: lista[mixto] = [];
vacia.agregar("primero");
vacia.agregar(2);
```

Las anotaciones `lista` y `diccionario` sin parámetro conservan el significado histórico de elementos enteros. Para un literal vacío de otro tipo, usa la anotación explícita. `mixto` solo es válido dentro de `lista[mixto]`: no existe una variable aislada `variable x: mixto`. Las claves de diccionario siguen siendo cadenas calculadas en ejecución. Se comprueba el tipo de toda inserción, asignación, acceso y argumento de función. Al recuperar un elemento mixto, las operaciones compatibles consultan su etiqueta y una conversión incompatible produce una excepción ST capturable. No hay impresión directa de colecciones.

```text
funcion primer_nombre(nombres: lista[cadena]) -> cadena {
    retornar nombres[0];
}
variable nombres = ["Ana", "Luis"];
mostrar(primer_nombre(nombres));
```

### Entrada y archivos de texto

```text
mostrar("Nombre:");
variable nombre: cadena = entrada();
escribir("saludo.txt", "Hola, " + nombre);
mostrar(abrir("saludo.txt"));
```

- `entrada()` lee una línea de la entrada estándar y elimina el LF final. Conserva un CR si forma parte de la entrada. En EOF devuelve cadena vacía. También admite `entrada("Ingresa tu nombre: ")`, que muestra el mensaje sin añadir un salto de línea.
- `abrir(ruta)` lee el contenido completo de un archivo y devuelve cadena. No devuelve un descriptor persistente; cierra el archivo antes de retornar.
- `escribir(ruta, contenido)` crea o **trunca** el archivo y escribe exactamente el contenido, sin añadir salto de línea. Devuelve ausencia de valor. Las rutas son relativas al directorio de ejecución, o absolutas; los programas tienen los permisos del usuario. No es un aislamiento de archivos.
- Lectura por llamada limitada a 1 MiB; rutas de hasta 4095 bytes y sin NUL. La entrada debe ser UTF-8 válido. Se rechazan secuencias sobrelargas, incompletas, sustitutos y valores fuera del rango Unicode.
- Las llamadas al sistema reintentan lecturas/escrituras interrumpidas y manejan transferencias parciales. Los fallos de estas operaciones terminan con diagnóstico y estado 1; no se convierten en excepciones recuperables de `lanzar`.
- La lectura reserva un búfer de aproximadamente 1 MiB que permanece hasta finalizar, incluso si se lee poco texto. Sigue aplicándose el presupuesto global de memoria del runtime. Esta primera implementación debe optimizarse para programas que leen muchas líneas.

En el IDE, pulsa **Ejecutar** y escribe cada respuesta en la línea de la terminal inferior cuando aparezca el mensaje. **Enter** o **Enviar** entrega una línea al programa; **Fin de entrada** (o Ctrl+D con la línea vacía) cierra su entrada. **Detener** cancela también un programa que esté esperando una respuesta. Cada envío admite hasta 4095 bytes UTF-8. La terminal acepta respuestas al programa activo, no comandos del sistema.

### Implementación revisada

`ElementType` en `ast.rs` representa el tipo de elemento, incluidos los niveles anidados y `Dynamic` para una lista mixta. `TypeName::collection`, `element`, `is_list` e `is_dict` centralizan la clasificación. El parser limita `mixto` a `lista[mixto]`; semántica valida cada operación. El runtime almacena cada elemento de lista en dos palabras: etiqueta y contenido. Los accesos dinámicos copian ambas palabras a una caja estable administrada por páginas, de modo que crecer la lista no deja referencias colgantes. El código existente de límites, crecimiento y presupuesto de memoria sigue activo.

`entrada`, `abrir` y `escribir` se registran en KEYWORDS y generan operaciones `ScalarOp`. `src/codegen/io.s` implementa rutas terminadas en NUL, llamadas al sistema, cierre y validación UTF-8; sus rutinas están comentadas. `ide/server.py` conecta una tubería a la entrada del binario generado; `send_input` entrega las respuestas sin bloquear el servidor y `command` recoge su salida. `ide/app.js` envía las líneas mediante `/api/input` y consulta `/api/status` para habilitar el campo solo durante la ejecución. Sin argumento de proyecto ni `PYFORGE_PROJECT`, el IDE abre el directorio desde el que se inició. El compilador y el programa se ejecutan con la carpeta del proyecto como directorio de trabajo. El explorador rechaza rutas externas, pero esto no es una jaula del sistema operativo: las funciones de archivos del programa conservan los permisos del usuario.

Ejemplos incluidos: `examples/colecciones_tipadas.st` y `examples/entrada_archivos.st`. El segundo crea o reemplaza `saludo.txt` cuando se ejecuta. Las pruebas usan directorios temporales y no escriben sobre archivos del usuario.

Las secciones siguientes documentan las ampliaciones posteriores de patrones, módulos, clases y excepciones. Las bibliotecas avanzadas siguen pendientes.

## 19. Patrones literales y módulos con alias

Esta sección describe `coincidir`, `caso`, `desde` y `como`. Las clases y excepciones se explican en las secciones 20 y 21.

### Selección por patrones

```text
variable estado = "listo";
coincidir estado {
    caso "pendiente" { mostrar("Todavía no"); }
    caso "listo" { mostrar("Puedes continuar"); }
    sino { mostrar("Desconocido"); }
}
```

El valor se evalúa exactamente una vez. Los casos se prueban en orden y solo se ejecuta el primero que coincide. `sino`, opcional, debe ser el último; sin coincidencia ni `sino`, no se ejecuta ningún cuerpo.

Los patrones son literales de entero, flotante, cadena, booleano o nulo; deben tener el mismo tipo que el valor. Se admite signo unario en patrones numéricos. No se admiten llamadas, variables, rangos, capturas, guardas ni desestructuración de colecciones. Las cadenas se comparan por contenido y los flotantes con sus reglas existentes. Los casos duplicados son válidos, pero el primero tiene prioridad; no hay comprobación de exhaustividad de valores.

Se conserva el significado de `retornar`, `romper` y `continuar` dentro del cuerpo. Para una función con retorno, el verificador exige que no quede un camino alcanzable sin retornar; no supone que una lista de casos cubra todo el tipo.

Practica la selección completa con `examples/patrones.st`.

### Importaciones con alias y selección

```text
importar "math.st" como matematicas;
desde "math.st" importar doble como duplicar;
mostrar(matematicas.doble(10));
mostrar(duplicar(10));
```

Este ejemplo requiere que `math.st` defina `doble`. El proyecto completo está en `examples/modules_alias/main.st`.

`importar "archivo.st" como alias` expone la API pública mediante `alias.funcion(...)` y `alias.variable`. `desde "archivo.st" importar nombre` selecciona una función o variable pública; puedes añadir `como nombre_local` y separar varias selecciones con comas. Los nombres privados, inexistentes y las colisiones con destinos distintos son errores de compilación.

Dos módulos pueden definir funciones homónimas si se usan alias o selección con nombres distintos. Las llamadas internas y la recursión se resuelven dentro del módulo. Un archivo se carga e inicializa una sola vez por ruta canónica, aunque se importe con varios alias. Continúan las comprobaciones de extensión, rutas relativas y ciclos.

Las funciones sin modificador continúan siendo públicas para conservar los proyectos anteriores. `publica funcion` expresa esa intención y `privada funcion` limita la llamada al archivo propietario. Todos los estilos de importación usan ahora la misma resolución aislada: `importar "archivo.st";` conserva las llamadas simples a las funciones públicas, pero un módulo debe importar directamente lo que utiliza. Una dependencia importada no se reexporta por accidente.

Las variables superiores son privadas por defecto. Solo `publica variable nombre = valor;` permite leerlas desde otro módulo. Una variable importada es de solo lectura para el consumidor; el módulo propietario puede reasignarla conservando su tipo. Las funciones no capturan variables superiores implícitamente: reciben los datos como parámetros. Esta regla evita estado global oculto y mantiene marcos locales verificables.

```st
// config.st
publica variable version: cadena = "1.0";
privada variable secreto = "interno";

publica funcion duplicar(valor: entero) -> entero {
    retornar valor * 2;
}

privada funcion validar(valor: entero) -> booleano {
    retornar valor >= 0;
}
```

```st
// main.st
importar "config.st" como config;
mostrar(config.version);
mostrar(config.duplicar(21));
```

`publica` y `privada` solo pueden preceder a `funcion` o `variable` en el nivel del módulo. La visibilidad de métodos y campos de clases todavía no forma parte de esta regla. Las variables públicas no son constantes: son escrituras privadas y lecturas públicas. Paquetes remotos versionados, importaciones Python y carga dinámica continúan fuera del alcance actual.

Vic y PyForge IDE colorean `publica`/`privada`, muestran las variables en amarillo y ofrecen fragmentos para visibilidad y conversiones. El autocompletado es una ayuda local; la comprobación del compilador decide finalmente la visibilidad, los tipos y la existencia de cada nombre.

### Estado que permanece pendiente

Los patrones compuestos siguen pendientes; las bibliotecas locales se explican en la sección 31. Las clases, estructuras, métodos y excepciones explícitas se describen a continuación. El lenguaje no está completo al 100 %. Los ejemplos y pruebas de esta actualización validan únicamente las capacidades descritas como disponibles.

## 20. Excepciones explícitas nativas

```st
funcion validar(edad: entero) {
    si edad < 0 { lanzar "La edad no puede ser negativa"; }
}
intentar {
    validar(-1);
} capturar error {
    mostrar(error);
} finalmente {
    mostrar("Validación terminada");
}
```

`lanzar` recibe una **cadena**, incluida una expresión calculada. Busca el `capturar` activo más cercano, incluso a través de funciones o métodos. El nombre después de `capturar` recibe el mensaje. Puedes relanzarlo con `lanzar error;` o construir otro. No hay jerarquías de clases de excepción ni selección por tipo todavía.

`intentar` exige `capturar nombre { ... }`, `finalmente { ... }` o ambos, en ese orden. `finalmente` se ejecuta una vez al terminar normalmente, retornar, romper, continuar o propagar una excepción explícita. El valor de `retornar expresión` se evalúa antes de la limpieza. Si la limpieza retorna, lanza o sale mediante un salto permitido por su contexto, esa nueva salida sustituye a la pendiente. Un error lanzado dentro del capturador no vuelve a entrar en ese mismo capturador.

```st
funcion numero() -> entero {
    intentar { retornar 7; }
    finalmente { mostrar("Limpieza antes de devolver 7"); }
}
mostrar(numero());
```

Declara antes de `intentar` cualquier variable que necesites consultar tanto en el capturador como en la limpieza. El análisis de inicialización es conservador: una excepción puede llegar antes de las asignaciones del cuerpo. No asume que el cuerpo se ejecutó por completo. El nombre del capturador no se puede leer después si existe un camino normal que no lo inicializa.

Una excepción sin capturar escribe `Error ST: excepción sin capturar: ...` en stderr y termina con código 1. **Los fallos internos de índices, división, desbordamiento, memoria, pila y archivos continúan siendo fatales**: esta implementación captura `lanzar`, no cualquier fallo del sistema. `finalmente` no se ejecuta al abortar por esos fallos ni al detener externamente el proceso. No hay liberación automática de todos los objetos abandonados; siguen sujetos al presupuesto de memoria existente.

### Cómo están conectadas las funciones

| Archivo / función | Responsabilidad |
| --- | --- |
| `src/st/lexer.rs`, `KEYWORDS` | Registra las cuatro palabras; el IDE también las lee |
| `src/st/parser.rs`, `statement` | Lee mensajes, capturadores y limpieza; crea `Stmt::Throw` y `Stmt::Try` |
| `src/semantic.rs`, `check_block` | Exige mensajes cadena y tipos estables; comprueba los cuerpos |
| `src/st/modules.rs`, `resolve_block` | Resuelve llamadas importadas también dentro de excepciones |
| `src/lower.rs`, `temporary` | Reserva locales internos para valores de retorno y mensajes guardados |
| `src/lower.rs`, `try_stmt` | Genera manejadores; una limpieza envuelve el cuerpo/capturador para cubrir también sus excepciones |
| `src/lower.rs`, `cleanup_to` | Retira manejadores de dentro hacia fuera y ejecuta limpiezas antes de retornos y saltos; conserva el contexto léxico de bucles |
| `src/ir.rs` | Representa `TryBegin`, `TryEnd`, `ExceptionValue` y `Throw` |
| `src/verify.rs`, `verify` | Comprueba pila, inicialización, coincidencia de manejadores al unir ramas y ausencia de manejadores pendientes al retornar |
| `src/codegen/llvm_compat.rs`, `emit_fn` | Reserva registros de cuatro palabras dentro del marco, incluidos en el límite de pila |
| `src/codegen/exceptions.s`, `pf_throw` | Retira el manejador, restaura `rsp`/`rbp` y salta al capturador; no usa Python ni libc |

Los cuatro campos del registro son: manejador anterior, pila guardada, marco guardado y dirección del capturador. Los registros pertenecen a marcos vivos; las salidas normales los retiran para evitar referencias a marcos destruidos. El estado global es para ejecutables monohilo. Ejemplo completo: `examples/excepciones.st`.

## 21. Clases, estructuras y métodos

```st
clase Persona {
    variable nombre: cadena;
    variable edad: entero;
    funcion saludar() -> cadena {
        retornar "Hola, soy " + yo.nombre;
    }
}
implementar Persona {
    funcion cumplir_anios() {
        yo.edad = yo.edad + 1;
    }
}
variable persona: Persona = nuevo Persona("Ana", 20);
persona.cumplir_anios();
mostrar(persona.saludar());
mostrar(persona.edad);
```

Cada campo se declara con `variable nombre: tipo;`. El constructor recibe **todos** los campos en orden; no hay campos sin inicializar ni valores predeterminados. Se admiten hasta seis campos por el convenio nativo de argumentos actual. También se puede llamar al constructor sin `nuevo`; con `nuevo` se exige una llamada que devuelva un objeto.

Los métodos usan `yo` implícitamente; no lo escribas en la lista de parámetros. Admiten hasta cinco parámetros explícitos además del receptor. Pueden retornar escalares u objetos, llamar otros métodos y lanzar excepciones. `yo` fuera de un método es un error. `implementar Nombre` añade métodos a una clase declarada en el mismo archivo, incluso si su declaración aparece después. Los métodos duplicados se rechazan.

`objeto.campo` lee un campo; `objeto.campo = expresión;` lo modifica comprobando su tipo. Para incrementarlo usa `objeto.edad = objeto.edad + 1;`: no hay asignación aumentada de campos todavía. Se pueden encadenar accesos y llamadas, como `caja.persona.saludar()` o `nuevo Persona("Ana", 20).edad`.

`estructura` tiene actualmente el mismo comportamiento que `clase`: ambas crean tipos nominales por referencia. No implica copia por valor. Dos declaraciones con campos idénticos siguen siendo tipos diferentes. Asignar un objeto a otra variable comparte la referencia; los cambios se ven desde ambos nombres. `es`, `==` y `!=` comparan identidad de objetos del mismo tipo, no comparan automáticamente todos sus campos.

Los campos pueden contener escalares, colecciones admitidas u otros objetos. Esto no habilita listas de objetos ni colecciones arbitrariamente anidadas. El objeto no se puede indexar como lista, imprimir directamente, medir con `longitud` ni reinterpretar como entero. Imprime un campo o el resultado de un método.

En módulos, los constructores se importan como funciones:

```text
// persona.st define la clase Persona del ejemplo.
importar "persona.st" como personas;
variable p = nuevo personas.Persona("Ana", 20);
mostrar(p.saludar());
```

También se admite `desde "persona.st" importar Persona como CrearPersona;` y `nuevo CrearPersona(...)`. Métodos y campos se resuelven a partir del tipo inferido del objeto. Las anotaciones con nombres de clases están disponibles para las clases declaradas en el mismo archivo; no se admiten aún anotaciones de clases importadas como `personas.Persona`. Dos módulos pueden definir clases homónimas sin confundir sus campos o métodos.

### Dónde ampliar la implementación

| Archivo / función | Responsabilidad |
| --- | --- |
| `src/ast.rs`, `TypeName::Object` | Identidad nominal separada de enteros y colecciones |
| `src/st/parser.rs`, `parse_module` | Pre-registra clases del archivo para reconocer referencias adelantadas en tipos |
| `src/st/parser.rs`, `class` | Lee campos y métodos; comprueba campos duplicados y el límite del constructor |
| `src/st/parser.rs`, `method` | Añade `yo` como primer parámetro y asigna un símbolo interno |
| `src/st/parser.rs`, `primary` / `atom` | Lee miembros, llamadas encadenadas y construcción con `nuevo` |
| `src/st/objects.rs`, `symbol` | Produce nombres privados únicos por clase y clase de miembro |
| `src/st/objects.rs`, `define` | Genera constructor, getters y setters tipados a partir de los campos |
| `src/st/objects.rs`, `member_call` | Resuelve un miembro por tipo y antepone una sola evaluación del receptor |
| `src/semantic.rs`, `infer_expr` / `check_block` | Verifica miembros, argumentos y asignaciones con las firmas generadas |
| `src/lower.rs`, `expr` / `stmt` | Convierte campos internos en accesos de posición comprobados y métodos en llamadas nativas |
| `src/st/modules.rs`, `resolve_expr` | Distingue llamadas de alias de módulo y métodos; conserva símbolos privados de clases |

Los objetos reutilizan el almacenamiento comprobado de colecciones, pero su representación no se expone como lista al programa. Los constructores inicializan cada posición con su tipo declarado, y solamente los accesores generados conocen los índices. El almacenamiento mantiene referencias estables y el límite global de memoria; no hay todavía recolector ni destructores. Las identidades internas se asignan al analizar y no son una API estable para el usuario.

No se implementan herencia, interfaces, visibilidad privada, sobrecarga, genéricos ni atributos dinámicos. Esta base permite programas orientados a objetos, pero no equivale a todo el modelo de objetos de Python. Ejemplo ejecutable: `examples/clases.st`.

## 22. Entrada con mensaje

Ya se puede escribir exactamente:

```st
variable nombre: cadena = entrada("Ingresa tu nombre: ");
mostrar("Hola, " + nombre);
```

El mensaje se evalúa una sola vez y se escribe en stdout **sin añadir salto de línea**, antes de leer stdin. Puede ser una cadena literal, una variable o una expresión que produzca cadena. Se admite un solo mensaje opcional; `entrada()` conserva su comportamiento y `entrada("")` no imprime bytes.

El resultado siempre es cadena UTF-8: elimina el salto `\n` leído y devuelve cadena vacía al llegar a EOF sin contenido. No convierte la respuesta a entero, flotante o booleano. Se conservan los límites y reglas de lectura de la sección 18.

Escribe el nombre cuando aparezca el mensaje. En el IDE, utiliza la línea de entrada de la terminal inferior y pulsa Enter. La respuesta se refleja en la salida y el programa continúa con `Hola, Ana`. Puedes responder a sucesivas llamadas a `entrada()` sin reiniciar la ejecución.

El ejemplo `examples/entrada_mensaje.st` permite probar mensajes, Unicode y varias lecturas consecutivas.

### Comprobación de esta ampliación

Las pruebas Rust cubren ejecución con y sin optimización: excepciones entre funciones recursivas, limpiezas en retornos/bucles, relanzamiento, mensajes sin capturar, inicialización conservadora, clases nominales, campos heterogéneos, alias de referencias, métodos, módulos con clases homónimas y entrada con mensaje/Unicode/EOF. Las pruebas negativas rechazan miembros inexistentes, tipos incorrectos y manejadores inconsistentes.

Esta ampliación añade capacidades concretas; no certifica un lenguaje «completo al 100 %» ni ausencia absoluta de errores. La sección 16 mantiene los trabajos todavía pendientes.

Resultado comprobado el 10 de septiembre de 2026: **47 pruebas Rust** (43 de integración ST y 4 del verificador), **24 pruebas del IDE**, regresiones nativas de Python y generación de documentación Rust aprobadas. Los tres ejemplos nuevos se ejecutaron con los binarios de distribución reconstruidos. La comprobación JavaScript con Node no pudo ejecutarse porque Node no está instalado; no se realizó comprobación visual del navegador.

## 23. Instalación y regeneración de paquetes

La guía [Instalación y empaquetado](INSTALACION.md) explica el `.deb`, el `.exe` para Windows con WSL, el PATH y `python3 scripts/package.py --watch`. Los instaladores nuevos contienen una instantánea coherente del compilador, KEYWORDS y recursos del IDE; una instalación existente se actualiza ejecutando el nuevo paquete.

La copia unificada de la raíz, `explicacion.md`, se actualiza desde este manual con `python3 scripts/update-explanation.py`. Edita este manual y ejecuta ese comando antes de regenerar los paquetes para mantener ambos documentos sincronizados.

## 24. Biblioteca estándar: time, os, math, io y json

Estos cinco módulos están incluidos dentro de `pyforgec`. No necesitas crear `math.st` ni instalar Python para utilizarlos. Son una biblioteca inicial propia de ST con las funciones enumeradas aquí; no reproducen toda la API de los módulos homónimos de Python. Los ejecutables siguen siendo ELF Linux x86-64 independientes, también ejecutables mediante WSL.

### Importar y elegir nombres

```st
importar "math";
importar "time" como reloj;
desde "json" importar codificar como a_json;

mostrar(math.raiz(81.0)); // 9.0; el argumento debe ser flotante
mostrar(a_json("Hola")); // "Hola", incluyendo las comillas JSON
variable inicio: entero = reloj.monotono();
reloj.dormir(20); // milisegundos
mostrar(reloj.monotono() - inicio);
```

Las importaciones van al comienzo del archivo y terminan en `;`. Sin `como`, el nombre estándar es también su alias: `math.raiz(...)`. Con `desde`, solo se importan las funciones seleccionadas. Los nombres `time`, `os`, `math`, `io` y `json` **no son palabras reservadas nuevas**.

`importar "math.st" como calculos;` sigue cargando un archivo local. Solo los cinco nombres exactos sin extensión seleccionan módulos incrustados. No hay búsquedas en PATH, descargas automáticas ni sustitución de la biblioteca por un archivo local. Un módulo se carga una vez aunque llegue por distintas dependencias. Los auxiliares con prefijo `aux_` de la biblioteca no se exportan.

### time: relojes y esperas

| Función | Resultado y comportamiento |
| --- | --- |
| `time.ahora()` | `entero`: milisegundos desde el epoch Unix; puede cambiar al ajustar el reloj |
| `time.monotono()` | `entero`: milisegundos monotónicos; compara diferencias para medir duraciones |
| `time.dormir(milisegundos: entero)` | `nada`: espera sin bucle activo; los negativos lanzan una excepción ST |

El tiempo monotónico no es una fecha. `dormir` puede tardar más de lo solicitado por la planificación del sistema. Una espera interrumpida se reintenta y sus errores se informan. No hay calendarios, zonas horarias, temporizadores asíncronos ni tareas concurrentes en este módulo inicial.

### os: directorios y archivos

| Función | Resultado y comportamiento |
| --- | --- |
| `os.directorio_actual()` | `cadena`: directorio absoluto actual del proceso, UTF-8, hasta 4095 bytes |
| `os.existe(ruta: cadena)` | `booleano`: verdadero si existe; falso ante ruta ausente o componente que no es directorio; sigue enlaces |
| `os.crear_directorio(ruta: cadena)` | `nada`: crea un directorio, sin crear padres; permisos `0777` limitados por la `umask` |
| `os.borrar_archivo(ruta: cadena)` | `nada`: elimina un archivo o enlace; no elimina directorios ni árboles |

Los errores de las operaciones de rutas se lanzan como cadenas ST con el código negativo `errno`: por ejemplo, `-2` significa ausencia y `-17`, nombre existente. `existe` no oculta errores de permisos como si fueran ausencia. Un enlace roto no cuenta como destino existente. Rutas vacías, con NUL o demasiado largas siguen siendo errores fatales del conversor de rutas común; tampoco se puede capturar un fallo de memoria del runtime.

Las rutas relativas parten del directorio de ejecución. En el IDE es la raíz del proyecto. Estas funciones conservan los permisos del usuario: **no aíslan el acceso al sistema de archivos**. No ejecutan comandos del sistema. Esta versión no añade variables de entorno, listado de directorios, subprocesos ni cambios de directorio.

### io: entrada y salida de texto

| Función | Resultado y comportamiento |
| --- | --- |
| `io.leer_linea(mensaje: cadena)` | `cadena`: muestra el mensaje, lee una línea y elimina LF; `""` permite leer sin mensaje |
| `io.leer_texto(ruta: cadena)` | `cadena`: contenido UTF-8 completo del archivo, hasta 1 MiB |
| `io.escribir_texto(ruta: cadena, contenido: cadena)` | `nada`: crea o reemplaza el contenido; no añade salto de línea |
| `io.imprimir(texto: cadena)` | `nada`: imprime texto con salto de línea |

Son adaptadores de `entrada`, `abrir`, `escribir` y `mostrar`. Esas operaciones anteriores siguen disponibles. EOF devuelve cadena vacía, igual que una línea vacía. No se exponen descriptores, archivos binarios, modos de apertura ni flujos persistentes. Los errores de lectura/escritura y de UTF-8 siguen siendo fatales, como en la API anterior.

```st
importar "io";
importar "os";

// Este ejemplo reemplaza saludo.txt en el directorio actual.
variable nombre: cadena = io.leer_linea("Tu nombre: ");
io.escribir_texto("saludo.txt", "Hola, " + nombre + "\n");
io.imprimir(io.leer_texto("saludo.txt"));
mostrar(os.existe("saludo.txt"));
```

### math: operaciones con tipos explícitos

| Función | Resultado y comportamiento |
| --- | --- |
| `math.pi()` | `flotante`: aproximación f64 de π |
| `math.euler()` | `flotante`: aproximación f64 de e |
| `math.absoluto(valor: flotante)` | `flotante`: valor absoluto; normaliza cero negativo |
| `math.absoluto_entero(valor: entero)` | `entero`: valor absoluto; el mínimo i64 causa el error de desbordamiento del runtime |
| `math.minimo(a: flotante, b: flotante)` | `flotante`: menor valor |
| `math.maximo(a: flotante, b: flotante)` | `flotante`: mayor valor |
| `math.raiz(valor: flotante)` | `flotante`: raíz cuadrada de hardware; los negativos lanzan excepción ST |
| `math.potencia(base: flotante, exponente: entero)` | `flotante`: potencia por cuadrados, también con exponentes negativos |

Usa `9.0` donde se espera flotante, no `9`. `potencia(0.0, 0)` devuelve `1.0`; cero con exponente negativo lanza excepción ST. El algoritmo admite el mínimo i64 como exponente sin negarlo directamente. Los flotantes conservan IEEE-754 y el formato de impresión del núcleo: no se añade precisión arbitraria ni se garantiza que sus operaciones produzcan valores finitos. No se incorporan todavía trigonometría, logaritmos o estadística.

```st
importar "math";

intentar {
    mostrar(math.raiz(-1.0));
} capturar error {
    mostrar("No se pudo calcular: " + error);
}
mostrar(math.potencia(2.0, -3)); // 0.125
```

### json: validar, construir y consultar documentos

`lista[mixto]` mezcla valores dentro de una lista ST, pero ST todavía no tiene diccionarios mixtos ni un árbol dinámico capaz de representar cualquier documento JSON. Por eso este módulo conserva cada valor compuesto como **texto JSON validado**. Puedes consultar documentos anidados y heterogéneos sin reinterpretarlos como colecciones ST incompatibles. Los números del documento conservan su representación textual: validar o extraer `1e400` no lo convierte a flotante ni pierde sus dígitos.

La gramática se basa en [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.html). Se aceptan objetos, arreglos, cadenas, números, `true`, `false` y `null`; se rechazan comas finales, comentarios, ceros iniciales y escapes inválidos. Como política de interoperabilidad Unicode, se rechazan sustitutos UTF-16 aislados; los pares válidos se convierten en un escalar UTF-8. Se permiten claves repetidas y `obtener` devuelve la primera coincidencia decodificada.

| Función | Resultado y comportamiento |
| --- | --- |
| `json.validar(texto: cadena)` | `booleano`: comprueba sintaxis y límites; captura errores ST de validación |
| `json.comprobar(texto: cadena)` | `nada`: lo mismo, pero lanza una excepción con el motivo |
| `json.codificar(texto: cadena)` | `cadena`: serializa una **cadena ST** con comillas y escapes JSON |
| `json.decodificar(texto: cadena)` | `cadena`: decodifica un **valor JSON de tipo cadena**, incluyendo `\uXXXX` |
| `json.obtener(texto: cadena, clave: cadena)` | `cadena`: extrae el valor JSON de una clave de objeto; ausencia lanza excepción |
| `json.elemento(texto: cadena, indice: entero)` | `cadena`: extrae un valor de arreglo; índices negativos o inexistentes lanzan excepción |
| `json.tipo(texto: cadena)` | `cadena`: `cadena`, `objeto`, `arreglo`, `numero`, `booleano` o `nulo` |
| `json.de_entero(valor: entero)` | `cadena`: número JSON con todos los dígitos i64 |
| `json.a_entero(texto: cadena)` | `entero`: convierte un entero JSON dentro de i64; rechaza fracción y exponente, incluso `1.0` o `1e0` |
| `json.de_booleano(valor: booleano)` | `cadena`: `true` o `false` en minúsculas |
| `json.a_booleano(texto: cadena)` | `booleano`: exige un booleano JSON |
| `json.valor_nulo()` | `cadena`: texto JSON `null` |
| `json.arreglo(valores: lista[cadena])` | `cadena`: construye un arreglo cuyos elementos son fragmentos JSON válidos |
| `json.objeto(claves: lista[cadena], valores: lista[cadena])` | `cadena`: construye un objeto, escapando claves ST y validando sus valores JSON; exige longitudes iguales |

`codificar` no es un serializador genérico de objetos ST. Para números fraccionarios puedes utilizar un fragmento JSON como `"1.25"` y comprobarlo; esta API no añade conversión entre JSON y f64. No hay JSON Schema, lectura incremental ni formato bonito.

```st
importar "json";

// Las claves son cadenas ST; cada valor contiene su representación JSON.
variable claves: lista[cadena] = ["nombre", "edad", "activo"];
variable valores: lista[cadena] = [
    json.codificar("Ana"), json.de_entero(20), json.de_booleano(verdadero)
];
variable documento: cadena = json.objeto(claves, valores);
mostrar(documento); // {"nombre":"Ana","edad":20,"activo":true}
mostrar(json.decodificar(json.obtener(documento, "nombre"))); // Ana
mostrar(json.a_entero(json.obtener(documento, "edad"))); // 20
mostrar(json.elemento("[null,{\"x\":1}]", 1)); // {"x":1}
mostrar(json.validar("[1,]")); // False
```

Los límites son **4096 caracteres Unicode por documento** y **64 niveles de contenedores**. Se comprueban también los documentos construidos. Los errores de sintaxis, tipo, índice, tamaño o profundidad se pueden capturar. Estos límites no sustituyen el presupuesto de memoria global: las cadenas temporales permanecen reservadas según el runtime actual, y muchas operaciones sucesivas pueden agotar sus 256 MiB. Un fallo de memoria sigue terminando el proceso. La implementación en ST prioriza claridad y todavía no es apropiada para archivos JSON grandes o servidores de larga duración.

### Cómo están conectadas las piezas

| Archivo y función | Responsabilidad |
| --- | --- |
| `src/st/modules.rs`, `visit` | Reconoce nombres estándar, crea una identidad virtual, deduplica y asigna alias por defecto |
| `src/st/modules.rs`, `load` / `resolve_expr` | Separa espacios de nombres, resuelve llamadas y excluye auxiliares estándar de las exportaciones |
| `src/st/stdlib.rs`, `source` / `contains` | Registra los módulos y sus fuentes mediante `include_str!` |
| `src/st/stdlib.rs`, `load` | Analiza las fuentes ST y añade funciones puente nativas como AST tipado |
| `src/st/stdlib.rs`, `NATIVES` / `native` | Define nombre, argumentos, resultado y operación IR de cada primitiva |
| `src/stdlib/*.st` | Implementa funciones públicas y auxiliares comentados utilizando el propio lenguaje |
| `src/semantic.rs`, `infer_expr` | Comprueba tipos y aridad de los intrínsecos usando el registro único |
| `src/lower.rs`, `expr` | Convierte sus `TextOp` en operaciones `ScalarOp` |
| `src/ir.rs`, `ScalarOp::runtime` | Asocia cada operación con su símbolo ensamblador y número de argumentos |
| `src/codegen/stdlib.s` | Implementa reloj, espera, rutas, raíz cuadrada y codificación/decodificación Unicode |
| `src/codegen/llvm_compat.rs` | Incluye el nuevo ensamblador junto al runtime existente |
| `ide/language.py`, `standard_modules` | Extrae las firmas públicas de las mismas fuentes ST |
| `ide/app.js`, `complete` | Sugiere funciones después del alias y un punto, por ejemplo `math.ra` |

Las primitivas reciben palabras de 64 bits en el convenio interno y devuelven una palabra en `rax`. Las cadenas son referencias a un descriptor con longitud en bytes y contenido UTF-8. Los flotantes viajan como sus bits f64; `sqrtsd` calcula la raíz. Las operaciones Linux comprueban sus resultados; los auxiliares ST convierten en excepciones los errores recuperables documentados. El programa final no interpreta ST ni invoca Python.

El analizador JSON usa posiciones Unicode comprobadas: `aux_valor_fin` recorre valores y contenedores, `aux_cadena_fin` comprueba cadenas, `aux_numero_fin` aplica la gramática numérica y `aux_unicode` combina escapes UTF-16. `comprobar` exige consumir todo el documento. `validar` transforma las excepciones de ese recorrido en un booleano. Los comentarios de cada función describen sus precondiciones y resultado.

### Agregar funciones y módulos

1. Para ampliar un módulo existente, edita su archivo en `src/stdlib/`. Declara funciones con parámetros y retorno tipados y añade comentarios y pruebas. Usa `aux_` en auxiliares privados. No hace falta modificar `KEYWORDS` para funciones normales.
2. Para crear un módulo nuevo, añade `src/stdlib/utilidades.st` y registra `"utilidades" => include_str!("../stdlib/utilidades.st")` en `source`, dentro de `src/st/stdlib.rs`. Las fuentes estándar actuales no importan otros módulos; sus llamadas internas deben apuntar a funciones del mismo archivo o a operaciones incorporadas.
3. Si necesitas una capacidad de CPU o sistema que ST no puede expresar, añade una firma a `NATIVES`, una variante de `ScalarOp`, su mapeo en `runtime` y una rutina comentada en `src/codegen/stdlib.s`. Comprueba argumentos, tamaños, errores, registros y ABI. Los puentes del AST, la semántica y el lowering se conectan mediante esa firma, sin añadir palabras reservadas.
4. Añade casos nativos en `tests/st.rs`, incluyendo entradas inválidas y ejecución con y sin optimización. Amplía esta sección y los ejemplos. El IDE lee firmas locales automáticamente, pero el compilador necesita recompilarse para incorporar su código.

```st
// Contenido propuesto para src/stdlib/utilidades.st; requiere registrar el módulo.
// Devuelve el doble conservando los controles de desbordamiento del núcleo.
funcion doble(valor: entero) -> entero {
    retornar valor * 2;
}
```

Para desarrollar y distribuir los cambios:

```bash
cargo test --locked
cargo build --release --locked
python3 -m unittest discover -s ide -q
python3 scripts/update-explanation.py
python3 scripts/package.py
python3 scripts/build-site.py
```

Los empaquetadores ya incluyen `src/stdlib/*.st` y reconstruyen los binarios: un `.deb` o `.exe` regenerado contiene la biblioteca nueva. Los instaladores ya descargados no cambian por sí solos; hay que instalar la versión nueva. El curso se genera de este manual, y su vigilante también detecta cambios en las fuentes de la biblioteca.

El ejemplo completo está en `examples/biblioteca_estandar.st`. Las pruebas cubren los cinco módulos, alias, selección, convivencia con `math.st` local, tipos incorrectos, auxiliares no exportados, archivos, duraciones, límites, construcción y consultas JSON. También se contrastó la validación de 400 documentos y mutaciones con el analizador JSON de Python; esa comparación es una comprobación de desarrollo, no una dependencia del ejecutable ST.

Verificación de esta ampliación (12 de septiembre de 2026): **57 pruebas Rust**, **31 pruebas del IDE**, **6 pruebas de web/empaquetado**, comprobaciones JavaScript de comentarios y módulos, y regresiones nativas del subconjunto Python aprobadas. Los cinco bloques ST de esta sección también pasan `pyforgec check`. Estas pruebas no equivalen a una garantía de ausencia absoluta de errores.

## 25. Inferencia, consulta de tipos y conversiones

### Variables sin anotaciones

El tipo después de `:` es opcional en las variables con valor inicial. Esto ya formaba parte de ST y se conserva:

```st
variable nombre = "Ana";       // cadena
variable edad = 20;            // entero
variable precio = 12.5;        // flotante
variable activo = verdadero;   // booleano
variable edades = [20, 30];    // lista[entero]
mostrar(tipo(nombre));
mostrar(tipo(edades));
```

Puedes seguir escribiendo `variable edad: entero = 20;` si quieres que el compilador compruebe una anotación explícita. Los parámetros y retornos de funciones siguen utilizando las anotaciones de la gramática actual.

**Inferencia no significa que una variable cambie de tipo durante la ejecución.** ST conserva tipos estáticos. Si `texto` contiene una cadena, `texto = entero(texto);` se rechaza por cambiar el tipo de esa variable. Guarda el resultado convertido en otra variable, por ejemplo `variable numero = entero(texto);`. Reasignar un valor del mismo tipo sigue permitido.

### Declaración obligatoria y existencia comprobada

`variable` crea e inicializa un nombre exactamente una vez en su ámbito. Una asignación sin `variable` solo modifica un nombre que ya existe:

```st
variable intentos = 0; // declaración e inicialización
intentos = intentos + 1; // reasignación válida: continúa siendo entero
```

`intetos = 1;` —con una letra ausente— produce `variable no declarada` en vez de crear estado accidentalmente. Repetir `variable intentos = 2;` en el mismo ámbito también es un error. Toda declaración exige un valor inicial; no existe una lectura de memoria sin inicializar. El verificador de flujo intersecta los nombres inicializados al unir ramas y rechaza usos que no estén definidos en todos los caminos alcanzables.

Estas reglas se parecen a una parte del modelo estático de Rust, pero ST todavía no implementa su comprobador completo de préstamos, tiempos de vida o alias. Los recursos nativos utilizan además `usar`, explicado en la sección correspondiente.

### Consultar con tipo(valor)

`tipo` recibe una expresión y devuelve una **cadena en español**. El argumento se comprueba y se evalúa una sola vez, incluyendo sus efectos y posibles errores. Normalmente devuelve el tipo estático conocido por el compilador. Cuando el argumento es un elemento de `lista[mixto]`, consulta su etiqueta en ejecución y devuelve su tipo real.

| Valor | Resultado de `tipo(valor)` |
| --- | --- |
| `20` | `"entero"` |
| `2.5` | `"flotante"` |
| `"hola"` | `"cadena"` |
| `verdadero` | `"booleano"` |
| `nulo` | `"nulo"` |
| `[1, 2]` o una lista vacía sin anotación | `"lista[entero]"` |
| `["a"]` | `"lista[cadena]"` |
| `["a"; 2; verdadero]` | `"lista[mixto]"` |
| Segundo elemento de esa lista | `"entero"` |
| `{"edad": 20}` | `"diccionario[entero]"` |
| Diccionarios/listas de otros escalares | `"diccionario[tipo]"` / `"lista[tipo]"`, con el tipo del elemento |
| Tupla / conjunto | `"tupla"` / `"conjunto"` |
| Instancia de clase o estructura | `"objeto"`; no se expone todavía su nombre nominal |

```st
variable dato = "20";
mostrar(tipo(dato)); // cadena
mostrar(tipo(20) == "entero"); // True
mostrar(tipo(tipo(20))); // cadena
variable datos = ["20"; 3; 4.5; falso];
mostrar(tipo(datos));     // lista[mixto]
mostrar(tipo(datos[2]));  // flotante
mostrar(entero(datos[0]) + datos[1]); // 23
```

Se rechazan llamadas sin argumento, con varios argumentos, con variables inexistentes o con funciones que no devuelven valor. `tipo` no es una palabra reservada nueva: una función definida o importada con ese nombre conserva prioridad. `json.tipo(documento)` sigue consultando el tipo **JSON**, mientras que `tipo(documento)` devuelve `"cadena"` si el documento está almacenado como texto ST. Esta consulta se añade a ST; no cambia el soporte experimental de `type()` en archivos Python.

### Convertir explícitamente

```st
variable texto = "20";
variable numero = entero(texto);
variable decimal = flotante(numero);
variable respuesta = booleano(numero);
variable nuevamente_texto = cadena(numero);
mostrar(numero + 1);       // 21
mostrar(tipo(decimal));    // flotante
mostrar(respuesta);        // True
mostrar(nuevamente_texto); // 20
```

| Función | Entradas y reglas |
| --- | --- |
| `entero(valor)` | Acepta entero, booleano, flotante o cadena decimal; trunca flotantes hacia cero y exige rango i64 |
| `flotante(valor)` | Acepta flotante, entero, booleano o cadena decimal con punto y exponente opcionales |
| `booleano(valor)` | Cero numérico, cadena/colección vacía y `nulo` son falsos; los demás valores con contenido y los objetos son verdaderos |
| `cadena(valor)` | Conversión escalar existente: conserva el formato de impresión del núcleo |

`booleano("falso")` devuelve verdadero porque la cadena no está vacía. No analiza palabras como `true`/`false`; para texto JSON usa `json.a_booleano`. No se convierten listas ni objetos a enteros o flotantes, ni se reinterpretan referencias de memoria como números.

Las conversiones numéricas de texto aceptan espacios ASCII exteriores y signo `+` o `-`. `entero("1.0")` y `entero("1e2")` se rechazan; `flotante(".5")`, `flotante("1.")` y `flotante("1e2")` son válidos. No se admiten bases alternativas, separadores `_`, coma decimal, dígitos Unicode, ni textos `nan`/`inf`.

La conversión decimal a f64 conserva hasta 19 dígitos significativos antes de aplicar la escala; no garantiza el mismo redondeo último bit que Python. El exponente escrito está limitado a magnitud 10000. Un resultado que desborda f64 se rechaza; un valor demasiado pequeño puede convertirse en cero. Convertir enteros grandes a f64 puede perder precisión, propia de ese formato. La impresión conserva la precisión del formateador existente.

Los errores de valor o rango en estas conversiones numéricas **se pueden capturar**:

```st
intentar {
    variable numero = entero("veinte");
    mostrar(numero);
} capturar error {
    mostrar("Conversión rechazada: " + error);
}
```

Los errores estáticos, como convertir una lista a entero, se detectan al compilar y no llegan a `capturar`.

### Implementación y pruebas

`src/st/modules.rs` resuelve la función contextual `tipo` después de conocer las funciones visibles, preservando nombres de usuario e importaciones. `src/st/parser.rs` reconoce `entero(...)`, `flotante(...)` y `booleano(...)` también como expresiones. `src/semantic.rs` comprueba aridad y tipos antes de generar código.

`TypeName::st_name`, en `src/ast.rs`, produce los nombres públicos en español. `src/lower.rs` evalúa el argumento de `tipo`, descarta su valor y coloca el descriptor de una cadena estática **sin salto de línea**; así funcionan tanto imprimir como comparar o concatenar el resultado. La corrección evita utilizar el internador antiguo de mensajes de impresión, que añade un salto LF.

Las conversiones numéricas se representan como `ScalarOp` en `src/ir.rs`. `src/codegen/conversions.s` implementa las comprobaciones i64, instrucciones SSE2, análisis decimal acotado y lanzamiento mediante `pf_throw`. Los valores booleanos de texto/colecciones usan su longitud; las otras conversiones booleanas comprueban cero. Todas las funciones nuevas tienen comentarios de implementación.

`examples/inferencia_y_conversiones.st` contiene un programa completo. Las regresiones comprueban tipos escalares, colecciones, objetos, efectos evaluados una sola vez, compatibilidad con `json.tipo`, inferencia, conversiones válidas, límites numéricos y errores capturables, con y sin optimización.

Verificación de esta ampliación (13 de septiembre de 2026): 61 pruebas Rust, 32 pruebas del IDE, 6 de web/empaquetado, regresiones nativas Python y pruebas JavaScript de paneles, plegado, módulos y comentarios aprobadas. Los ejemplos ST de esta sección también pasan la comprobación del compilador.

## 26. Colección de 80 proyectos prácticos

La carpeta `examples/proyectos/` contiene **80 proyectos independientes**, numerados del 01 al 80. Su `README.md` presenta un catálogo con enlaces. Cada proyecto incluye `src/main.st`, `PyForge.toml`, instrucciones específicas, entrada y salida esperadas bajo `pruebas/`, y los módulos locales o datos que necesita.

| Carpetas | Área | Ejemplos de aplicaciones |
| --- | --- | --- |
| 01–10 | Juegos de consola | Tres en raya, ahorcado, Nim, laberinto, aventura y combate por turnos |
| 11–20 | Negocio | Facturas en centavos, caja, presupuesto, inventario y cotizaciones |
| 21–30 | Organización | Tareas, agenda, estudio, hábitos, reuniones y turnos |
| 31–40 | Texto y datos | Palíndromos, identificadores URL, CSV, frecuencias y estadísticas |
| 41–50 | Archivos y JSON | Notas, diario, copias, configuración, catálogos y registros |
| 51–60 | Algoritmos | Búsqueda binaria, criba, monedas, matrices y recorridos de grafos |
| 61–70 | Modelado | Cuenta, biblioteca, carrito, reservas, cronómetro, pila y módulos |
| 71–80 | Informes | HTML, barras, CSV, Markdown, calendario, vCard, RSS y SVG |

Son ejemplos funcionales acotados para estudiar y adaptar, no productos de producción ni una demostración de capacidades que el lenguaje todavía no tiene. Los juegos usan texto. Las páginas son HTML estático; no levantan servidores ni incluyen un framework web. Las tasas, personas, contactos y datos son ficticios. Consulta el apartado de alcance de cada proyecto.

### Abrir un proyecto

Desde la raíz de esta distribución, por ejemplo:

```bash
cd examples/proyectos/03_tres_en_raya
pyforge run
```

También puedes ejecutar `pyforgec run src/main.st` o `pyforge-ide .` desde esa carpeta. Las rutas de datos son relativas al directorio de ejecución: abre la carpeta del proyecto, no solo su archivo ST. Las instrucciones indican qué entradas introducir y qué archivos se crean o reemplazan.

### Verificación y distribución

Desde el repositorio de desarrollo, `python3 scripts/verify-projects.py` comprueba todos los proyectos con y sin optimización. Copia cada uno a una carpeta temporal, suministra la entrada prevista, acota el tiempo/volumen de salida y compara bytes de salida y archivos generados. No modifica los originales. Para uno solo utiliza `--project 03_`. Puedes indicar otro compilador con `--compiler /ruta/pyforgec`. Python se usa únicamente en el verificador; las aplicaciones son ST compilado a nativo.

`examples/proyectos/catalogo.json` identifica entradas, fixtures y archivos esperados. `scripts/project_catalog.py` selecciona esos recursos para la distribución y excluye resultados u otros archivos que el usuario haya añadido. El empaquetador ejecuta las pruebas de los proyectos antes de crear instaladores. La web ofrece `proyectos-st.zip` con los 80 proyectos completos; el ZIP de código de ejemplos por sí solo no contiene todas sus instrucciones y datos.

Para agregar un proyecto sigue la misma estructura, registra su caso en el catálogo y comprueba ambos modos de optimización. Si cambias los resultados de uno existente, actualiza sus pruebas y las expectativas de archivos. Mantén entradas pequeñas y usa las restricciones actuales de colecciones, JSON y memoria del lenguaje.

## 27. Documentos, redes e integraciones opcionales

El núcleo ST sigue compilando a ELF Linux. Los nuevos módulos `archivos`, `db`,
`documentos`, `red`, `cifrado`, `comandos`, `sistema`, `vpn`, `gui` e `integraciones`
utilizan un auxiliar opcional Python 3.11 o posterior. No equivalen a un ecosistema
completo ni a un sandbox; los ejecutables que los usan necesitan ese auxiliar y,
según la operación, bibliotecas externas. Windows sigue utilizando WSL.

### Primer programa y permisos

```st
importar "integraciones";
mostrar(integraciones.informacion());
```

La información muestra dependencias realmente detectadas. Para acceder a archivos,
red o procesos debes declarar capacidades en `[permisos]` de `PyForge.toml` de la
carpeta actual. Los nombres desconocidos se rechazan. El IDE dispone de un botón
**Permisos** para consultar esa tabla; no autoriza operaciones automáticamente.

```toml
[permisos]
leer_archivos = true
escribir_archivos = true
base_datos = true
```

### SQLite con parámetros

```st
importar "db";
variable conexion = db.conectar("agenda.db");
intentar {
    db.ejecutar(conexion, "CREATE TABLE IF NOT EXISTS personas(nombre TEXT)", "[]");
    db.ejecutar(conexion, "INSERT INTO personas VALUES (?)", "[\"Ana\"]");
    db.confirmar(conexion);
    mostrar(db.consultar(conexion, "SELECT nombre FROM personas LIMIT 10", "[]"));
} finalmente { db.cerrar(conexion); }
```

Los parámetros y resultados compuestos se transportan como texto JSON. No cambia
el sistema estático de tipos de ST. Nunca concatenes valores del usuario al SQL.

### Capacidades y límites

Hay archivos locales, SQLite, texto TXT/RTF, adaptadores DOCX/PDF y LibreOffice,
TCP/UDP, cliente TLS, SSH/SFTP, FTPS, cifrado autenticado, contraseñas PBKDF2 y
procesos sin shell. La GUI declarativa utiliza Qt en otro proceso. VPN únicamente
detecta herramientas e inspecciona perfiles; no modifica conexiones ni firewall.
Faltan edición avanzada de documentos, cifrado de archivos por bloques,
administración VPN y aislamiento completo del sistema, entre otras funciones.

Las APIs anteriores `abrir`, `escribir` y `os` mantienen sus permisos del sistema;
la tabla nueva no las restringe. No ejecutes código hostil basándote solo en ella.

### Ejemplos y mantenimiento

Se añadieron 12 proyectos independientes en `examples/integraciones`, cada uno con
manifiesto y comentarios. Ejecútalos desde su propia carpeta, en una copia temporal.
Los ejemplos que necesitan bibliotecas externas lo indican explícitamente.

La guía completa está en `docs/INTEGRACIONES.md`, las firmas se generan desde las
fuentes en `docs/INTEGRACIONES.md` y la validación está en
`docs/README.md`. Editar `src/stdlib` actualiza sugerencias; para cambiar
el código ejecutado hay que reconstruir compilador, programas e instaladores.

### Aplicaciones completas de práctica

La colección `examples/proyectos_avanzados` añade 20 proyectos independientes: seis juegos y catorce aplicaciones. La guía [MANUAL_ST.md](MANUAL_ST.md) describe módulos, dependencias, pruebas y límites. Los permisos se configuran por proyecto y las conexiones de prueba usan servidores locales.

### Menú de inicio del IDE

La aplicación instalada abre un menú para entrar al editor, abrir o crear proyectos, consultar el manual y apoyar ST. Su configuración se actualiza desde JSON mediante un script, sin recompilar. Consulta `docs/INSTALACION.md`. El autocierre está siempre activo; JSON y TOML tienen colores y SQLite dispone de una vista de solo lectura.

### Interfaces gráficas y navegación del IDE

Consulta `docs/INTEGRACIONES.md` para conocer cada función de gui, sus conexiones, controles, eventos y puntos de extensión. El manual del IDE se presenta con índice y colores. La cabecera de la terminal ofrece Inicio y el atajo Ctrl+Alt+H para volver al menú sin enviar comandos al programa.


## 28. Crear sitios web con ST y SQL

Un proyecto web combina lógica ST nativa con Django para HTTP, sesiones, formularios y consultas SQL parametrizadas. No hace que el compilador admita todo Python ni compila Django a ST.

```bash
pyforge new mi_web --web
cd mi_web
python3 -m venv .venv
.venv/bin/python -m pip install -r web/requirements.txt
export PYFORGE_WEB_PYTHON="$PWD/.venv/bin/python"
pyforge web --port 8000
```

Edita `src/vistas.st` para la lógica, `web/templates` para HTML y `web/static` para CSS/JS. La plantilla sencilla incluye una vista ST, capturas aisladas por sesión y culebrita con reglas nativas. Los ejemplos anteriores de notas, inventario y agenda siguen en examples/web. El foro oficial está en `website/backend`. El servidor local no es el despliegue público: consulta `docs/INTEGRACIONES.md` para HTTPS, WSGI, secretos y límites.

## 29. Privacidad y temas del IDE

En Inicio puedes crear una bóveda `.stvault` o abrir una existente con contraseña. El contenedor se cifra con AES-GCM y la copia de trabajo reside en tmpfs; Bloquear guarda y revoca las sesiones. No convierte carpetas existentes ni protege frente a procesos que controlen tu sesión abierta. El modo normal sigue disponible. Consulta `docs/INSTALACION.md` antes de usarlo con datos importantes.

Los temas Normal, Hacker y Pro oscuro cambian la presentación y se recuerdan localmente. No conceden permisos adicionales ni cambian la ejecución del programa.

## 30. Motor 2D comprobable

Ejecuta `pyforgec run examples/games/01_motor_verificable.st` para estudiar entidades, colisiones y frames. El juego gráfico está en `examples/web/culebrita`; sigue su README. ST calcula las reglas, Django conserva la sesión y JavaScript dibuja. No es un motor 3D ni ejecuta Minecraft o Call of Duty. Consulta `docs/README.md` para límites y ampliación.


### Juegos y ejemplos para continuar

El motor `juego` (alias `motor` y `motor2d`) calcula movimiento, gravedad, salto,
colisiones AABB, escenas y frames. No dibuja por sí mismo. Empieza por:

```bash
pyforgec run examples/games/01_motor_verificable.st
```

La salida es `True`, `False`, `63`, `64`, `1`, `1`, en líneas separadas.
Los ejemplos `snake.st`, `pong.st`, `breakout.st`, `plataforma.st`, `motor_2d.st`
y `motor_avanzado.st` están en `examples/games/`. La culebrita web completa está en
[examples/web/culebrita](../examples/web/culebrita/README.md): ST calcula las reglas,
Django conserva la sesión y JavaScript dibuja y recoge el teclado. No incluye un
motor 3D, audio, mundos abiertos ni compatibilidad con Minecraft o Call of Duty.

Para practicar tareas diarias, consulta los catálogos de
[80 proyectos](../examples/proyectos/README.md),
[20 proyectos avanzados](../examples/proyectos_avanzados/README.md) y
[12 integraciones](../examples/integraciones/README.md).


## 31. Colecciones anidadas y bibliotecas locales

ST compila estas capacidades a código nativo. No necesita Python para ejecutar
las colecciones ni importar bibliotecas ST. La herramienta opcional de copia y
verificación de dependencias sí usa Python 3.

### Matrices y estructuras anidadas

```st
variable matriz = [[1; 2]; [3; 4]];
matriz[1][0] = 9;
mostrar(matriz[1][0]); // 9
variable contactos: diccionario[lista[cadena]] = {};
contactos["equipo"] = ["Ana"; "Luis"];
contactos["equipo"].agregar("Eva");
mostrar(contactos["equipo"][-1]); // Eva
mostrar(tipo(contactos)); // diccionario[lista[cadena]]
```

Las listas homogéneas conservan un único tipo de elementos y permiten operaciones
estáticas directas. Una mezcla como `[1; "texto"]` infiere `lista[mixto]` y
conserva orden, mutación y etiquetas en ejecución. También puedes declarar
`lista[mixto]` en variables, parámetros y retornos. Una lista vacía sin anotación
sigue siendo `lista[entero]`; usa la anotación explícita para llenarla con varios
tipos. Los diccionarios mantienen claves de cadena y valores homogéneos.

Las listas, incluidas las mixtas, pueden contener otras listas, diccionarios,
tuplas, conjuntos u objetos. Los diccionarios tipados también pueden contener
colecciones compatibles. Tuplas y conjuntos conservan elementos enteros.
Las referencias se comparten: asignar una fila a otra variable no la copia.
Los accesos conservan las comprobaciones de límites del runtime. No hay recolección
de basura nueva. `mostrar` requiere acceder a los valores individuales; para una
colección recuperada desde una lista mixta muestra su nombre de tipo, no recorre
automáticamente todo el contenido. La representación de tipos
es acotada: admite hasta quince niveles internos de contenedores y rechaza una
profundidad mayor con un error de compilación.

### Bibliotecas del proyecto

```text
mi_proyecto/
  PyForge.toml
  Bibliotecas.lock.json
  src/main.st
  bibliotecas/estadistica/src/lib.st
```

```st
importar "@estadistica" como estadistica;
variable ventas = [[10; 20]; [30; 40]];
mostrar(estadistica.sumar_filas(ventas));
```

`@nombre` busca el proyecto más cercano que contiene `PyForge.toml` y carga
`bibliotecas/nombre/src/lib.st`. Los módulos estándar y las rutas relativas
anteriores siguen funcionando. Dentro de una biblioteca puedes importar otros
archivos `.st` con rutas relativas. Usa alias para evitar colisiones. El ejemplo
completo está en `examples/bibliotecas_locales`; ejecuta `pyforge run` dentro de él.

Para copiar una biblioteca local que tenga `src/lib.st`, desde este repositorio:

```sh
python3 scripts/bibliotecas.py agregar estadistica /ruta/a/estadistica --proyecto /ruta/a/mi_proyecto
python3 scripts/bibliotecas.py verificar --proyecto /ruta/a/mi_proyecto
```

Con el `.deb`, la herramienta queda en
`/usr/share/pyforge/scripts/bibliotecas.py`. Copia solamente fuentes `.st`, rechaza
enlaces simbólicos y no sobrescribe una dependencia existente. Conserva la licencia
y atribución de cada biblioteca por separado. El archivo de bloqueo registra
SHA-256 de cada fuente copiada; súbelo junto con `bibliotecas/` a tu repositorio.
Ejecuta `verificar` antes de compilar en integración continua. **El compilador no
verifica automáticamente ese bloqueo**: permite editar fuentes locales y la
verificación separada detecta esas modificaciones.

Esta es una base local del ecosistema: no incorpora registro público, resolución
de versiones remotas, actualizaciones automáticas ni firmas de autores. Los hashes
detectan cambios respecto del bloqueo; no certifican que una biblioteca sea fiable.

### Dónde ampliar el código

- `src/ast.rs`: `ElementType::Nested`/`Dynamic`, codificación acotada y nombres de `tipo()`.
- `src/semantic.rs`: comprueba listas tipadas y limita las operaciones dinámicas seguras.
- `src/st/parser.rs`: lee anotaciones recursivas como `lista[lista[cadena]]`.
- `src/lower.rs`: conserva las referencias y reutiliza operaciones comprobadas del runtime.
- `src/st/modules.rs`: `library_path` resuelve `@nombre`; `visit` detecta ciclos.
- `scripts/bibliotecas.py`: copia explícita y bloqueo local; no ejecuta instaladores.
- `tests/st.rs` y `tests/test_bibliotecas.py`: regresiones nativas y verificación de hashes.

## 32. Runtime Rust nativo: archivos y buffers binarios

El módulo experimental `nativo` se enlaza al ejecutable Linux x86-64. No inicia
Python ni intercambia JSON. Se añade junto a `io`, `os`, `archivos`, `db`, `red` y
las demás integraciones; no las sustituye ni cambia sus contratos.

```st
importar "nativo";
variable datos = nativo.desde_texto("Hola ñ");
intentar {
    mostrar(nativo.version()); // ABI 1
    mostrar(nativo.tamano(datos)); // 7 bytes UTF-8
    nativo.poner(datos; 0; 104);
    mostrar(nativo.texto(datos)); // hola ñ
} finalmente { nativo.cerrar(datos); }
```

### API y propiedad de recursos

| Función | Resultado y contrato |
| --- | --- |
| `version()` | Entero de versión de ABI; actualmente 1. |
| `buffer(tamano)` | Identificador de buffer de 0 a 65536 bytes, inicialmente cero. |
| `desde_texto(texto)` | Copia los bytes de una cadena a un buffer nuevo. |
| `tamano(buffer)` | Longitud en bytes, no en caracteres. |
| `obtener(buffer; indice)` | Byte entero entre 0 y 255. Índices no negativos. |
| `poner(buffer; indice; valor)` | Modifica un byte; valida índice y rango 0–255. |
| `texto(buffer)` | Valida UTF-8 y copia a una cadena ST. Lanza error si no es texto válido. |
| `abrir_archivo(ruta; modo)` | Archivo abierto: 0 lectura, 1 creación exclusiva, 2 creación/truncado explícito. |
| `leer(archivo; cantidad)` | Buffer nuevo con hasta 1–65536 bytes solicitados; una lectura corta es válida. |
| `escribir_archivo(archivo; buffer)` | Escribe todo el buffer y devuelve el número de bytes. |
| `cerrar(recurso)` | Cierra archivo o libera buffer; también invalida sus alias. |

Los identificadores son enteros comprobados, no punteros ni un tipo `bytes` propio
del sistema de tipos. No los sumes ni inventes: el runtime comprueba que existan y
que correspondan al recurso esperado. No se reutilizan identificadores. Esta API
no es un sistema de préstamos, una capacidad de seguridad ni gestión automática de
recursos. Cierra cada identificador con `finalmente`.

Hay **32 recursos vivos compartidos entre archivos y buffers**. Los buffers son
de longitud fija y se pueden reutilizar o cerrar para liberar ranuras. Su reserva
estática ocupa aproximadamente 2 MiB, separada del presupuesto histórico del heap
ST. Al cerrar se borran los bytes usados, sin garantizar la eliminación de copias
externas. `texto()` crea una copia en el heap ST: esa cadena sigue viviendo hasta
terminar el programa, como las cadenas anteriores. Convertir repetidamente buffers
a cadenas no elimina esa limitación de memoria.

### Archivos, errores y permisos

`leer` devuelve un buffer de longitud cero al llegar a EOF. Cierra también ese
buffer. Para archivos grandes, procesa bloques y libera cada uno. Una lectura no
promete devolver todos los bytes pedidos. Una escritura que falla puede dejar una
parte escrita; no es una transacción ni una publicación atómica del archivo.

El modo 1 rechaza un destino existente; el modo 2 puede destruir su contenido.
Las rutas se resuelven desde el directorio de trabajo o pueden ser absolutas;
se rechazan ruta vacía, NUL y más de 4095 bytes. La API usa los permisos normales
del proceso y **no aplica `[permisos]` del auxiliar Python**. No es un sandbox.
Los archivos nuevos usan permisos 0600 sujetos a umask. Las aperturas usan CLOEXEC
y NONBLOCK; los dispositivos especiales pueden producir errores no presentes en
archivos regulares. No hay aislamiento contra enlaces simbólicos.

Los fallos esperados se convierten en excepciones ST mediante `lanzar`:

- Recurso cerrado, inexistente o de otro tipo.
- Límite de 32 recursos alcanzado.
- Tamaño, índice, byte, ruta o modo fuera del contrato.
- Contenido que no es UTF-8 válido.
- Error del sistema identificado por su número `errno`.

Un fallo interno de Rust termina con estado 70. El agotamiento del heap ST mantiene
su diagnóstico fatal anterior; no se convierte automáticamente en excepción.

### Ejemplos y compilación

`examples/nativo_buffers.st` no crea archivos. `examples/nativo_archivos.st` crea
`nativo-demo.bin` sin sobrescribirlo; ejecútalo desde una carpeta de trabajo temporal.
No elimina el archivo al terminar. Para repetirlo usa otra carpeta o retira el
archivo de prueba después de revisarlo.

```bash
pyforgec build examples/nativo_archivos.st -o dist/nativo-archivos
```

La compilación del propio PyForge genera la biblioteca mediante `build.rs` y Rust.
El compilador distribuido la lleva incrustada y la extrae solo para enlazar programas
que la usan. El usuario final no necesita Rust ni Python para ese programa; compilar
ST sigue necesitando GNU `as` y `ld`. El backend actual sigue siendo Linux x86-64.

### Dónde extenderlo y qué falta

- `runtime/native/lib.rs`: ABI C 1, ranuras, llamadas Linux y operaciones de buffers.
- `src/stdlib/nativo.st`: API pública y traducción de errores a excepciones ST.
- `src/st/stdlib.rs`: firmas privadas tipadas; el IDE descubre las funciones públicas.
- `src/ir.rs`: operaciones y símbolos de la ABI.
- `src/codegen/llvm_compat.rs`: puente de reserva al heap ST, con el convenio de llamadas nativo.
- `src/driver.rs`: enlace condicional y limpieza del archivo estático intermedio.
- `build.rs`: construcción e incrustación del runtime; no requiere descargar crates.
- `tests/st.rs`: límites, vida de recursos, binarios, errores y ambas optimizaciones.

El runtime es monohilo y experimental. No incorpora red, TLS, SQL o gráficos nativos
nuevos, ni genéricos, préstamos, tareas o recolector. El siguiente incremento debe
introducir tipos de recurso y ownership verificables antes de añadir concurrencia.

## 33. Propiedad comprobada de recursos nativos

El módulo experimental `recursos` añade `archivo_nativo`, `buffer_nativo` y el
bloque contextual `usar`. Estos tipos no son enteros: no se pueden sumar, convertir,
imprimir, guardar en colecciones u objetos ni asignar a otra variable. Una función
puede recibirlos como préstamos durante una llamada, pero no guardarlos o retornarlos.

```st
importar "recursos";

funcion mostrar_buffer(datos: buffer_nativo) -> nada {
    mostrar(recursos.texto(datos)); // préstamo válido durante esta llamada
}

usar datos = recursos.desde_texto("Hola") {
    mostrar_buffer(datos);
    recursos.poner(datos; 0; 104);
    mostrar(recursos.texto(datos)); // hola
} // cierre generado automáticamente
```

`usar nombre = fabrica(...) { ... }` acepta solamente una llamada cuya firma
garantiza un recurso nuevo. La variable existe únicamente dentro del bloque. El
compilador genera la limpieza al terminar normalmente y también al ejecutar
`retornar`, `lanzar`, `romper` o `continuar`. Los bloques se pueden anidar y se
cierran desde el más interior. Si la adquisición falla, el bloque no comienza y
los propietarios exteriores activos siguen limpiándose.

`usar` es contextual: una función anterior llamada `usar(...)` sigue siendo válida.
`buffer_nativo` y `archivo_nativo` solo se reconocen como tipos en anotaciones. El
IDE y la extensión los colorean y sugieren desde `CONTEXTUAL_WORDS`, sin agregarlos
a `KEYWORDS` ni reservarlos globalmente.

### API tipada

| Función | Propiedad |
| --- | --- |
| `recursos.buffer(tamano)` | Produce un `buffer_nativo` nuevo. |
| `recursos.desde_texto(texto)` | Produce un buffer nuevo con una copia. |
| `recursos.abrir_archivo(ruta; modo)` | Produce un `archivo_nativo` nuevo. |
| `recursos.leer(archivo; cantidad)` | Pide prestado el archivo y produce un buffer nuevo. |
| `recursos.escribir_archivo(archivo; datos)` | Pide prestados ambos recursos. |
| `recursos.tamano`, `obtener`, `poner`, `texto` | Piden prestado el buffer; no lo cierran. |

Una función envoltorio puede retornar un recurso únicamente si su cuerpo retorna
una sola llamada a una fábrica que el análisis ya reconoce como productora de
propiedad nueva. No se acepta reenviar un parámetro prestado. Esto evita declarar
una función de identidad como fábrica y crear dos propietarios para el mismo id.

No hay función pública `cerrar` en `recursos`. El cierre generado llama a auxiliares
privados del módulo; otro archivo no puede importarlos. La API anterior `nativo`
continúa disponible y conserva identificadores enteros y cierre manual. No hay
conversiones entre ambas APIs, para que código anterior siga funcionando sin
debilitar las reglas nuevas.

### Garantías y límites

- Propiedad exclusiva y préstamos se comprueban léxicamente dentro de un archivo
  compilado. No constituyen aún un sistema general de préstamos para objetos.
- El runtime vuelve a validar tipo y vida del identificador. El análisis estático
  y la comprobación dinámica protegen capas diferentes.
- Una cadena devuelta por `recursos.texto` es una copia normal de ST y puede salir
  del bloque. El recurso original se cierra; la copia conserva la vida histórica
  del heap de cadenas.
- Los recursos no se guardan en campos, listas o diccionarios. Tampoco se capturan
  porque ST todavía no tiene cierres de funciones.
- El runtime permanece monohilo y limitado a 32 recursos vivos de 64 KiB por buffer.
- Un fallo durante el cierre puede reemplazar una excepción que estaba saliendo
  del bloque. Aún no existe un canal separado de errores de limpieza suprimidos.
- Los permisos y riesgos de rutas son los documentados para `nativo`; `recursos`
  mejora la propiedad, no crea una sandbox del sistema operativo.

El ejemplo completo está en `examples/recursos_propiedad.st`. Los nodos se definen
en `ast.rs`; el parser reconoce `usar`; semántica propaga el efecto de fábrica y
rechaza escapes; lowering reutiliza la maquinaria comprobada de `finalmente`; y
`resources.s` traduce errores del runtime después de volver de Rust, para no saltar
a través de una pila Rust activa.

## 34. Teclado inmediato para juegos de terminal

El módulo `teclado` permite leer una tecla sin esperar Enter. Debe ejecutarse desde
una terminal interactiva en Linux x86-64.

```st
importar "teclado";
importar "time";

variable activo = verdadero;
variable tecla = teclado.ninguna();
si teclado.es_terminal() {
    mientras activo {
        tecla = teclado.leer(16);
        si tecla == teclado.izquierda() o tecla == 97 {
            mostrar("mover a la izquierda"); // flecha izquierda o A
        }
        si tecla == 113 o tecla == teclado.escape() { activo = falso; }
        time.dormir(16);
    }
}
```

| Función | Resultado |
| --- | --- |
| `teclado.es_terminal()` | Indica si stdin admite captura inmediata. |
| `teclado.leer(ms)` | Código de tecla o `-1` si no hubo entrada; espera de 0 a 1000 ms. |
| `arriba`, `abajo`, `derecha`, `izquierda` | Códigos estables 1001, 1002, 1003 y 1004. |
| `escape`, `espacio`, `enter`, `ninguna` | Códigos 27, 32, 10 y -1. |

Las letras y números usan su byte ASCII: `W` minúscula es 119, `A` es 97 y `Q`
es 113. Las flechas se reconocen desde sus secuencias ANSI. La versión actual no
devuelve eventos separados de pulsación y liberación, combinaciones modificadoras,
ratón ni múltiples teclas mantenidas simultáneamente.

Antes de cada lectura, el runtime conserva la configuración del terminal; desactiva
temporalmente entrada canónica, eco y señales de control; y restaura la configuración
antes de retornar por éxito, tiempo agotado o error. La espera se limita a un segundo
para reducir el intervalo en modo inmediato. Como en otros programas de terminal,
`SIGKILL`, un fallo del kernel o la pérdida abrupta del equipo no permiten ejecutar
limpieza. El ejemplo completo está en `examples/teclado_terminal.st`.

## 35. Núcleo propio del motor 3D

`importar "motor3d";` incorpora una base 3D escrita en ST y compilada a código
nativo. Esta etapa establece el modelo de datos que usarán el renderizador, la
física y las herramientas, sin añadir una dependencia gráfica obligatoria a todos
los ejecutables.

```st
importar "motor3d";

variable motor = motor3d.crear_motor("nivel"; 1280; 720);
motor.iniciar();
motor.actualizar(0.016);

variable movimiento = motor3d.matriz_traslacion(motor3d.vector(2.0; 3.0; 4.0));
variable punto = movimiento.transformar_punto(motor3d.vector(1.0; 1.0; 1.0));
mostrar(punto.x); // 3.0
```

### Capacidades implementadas

- `Vector3`, cuaterniones normalizados, transformaciones y matrices 4×4.
- Cámara con perspectiva configurada, planos de recorte y validación.
- Mallas trianguladas con vértices e índices comprobados; fábrica de cubos.
- Material PBR descriptivo, color, metalicidad, rugosidad y nombre de shader.
- Luces direccionales, puntuales y de foco con parámetros validados.
- Colisiones de cajas AABB, consulta de puntos y cuerpos rígidos lineales.
- Animación interpolada, escena validada y ciclo de actualización con delta acotado.
- Catálogo de recursos con nombres únicos, tipos, estados, tamaños, presupuesto de
  memoria y rechazo de uso doble después de liberar.
- Autocompletado de las funciones `motor3d` en el IDE y VS Code.

El ejemplo verificable está en `examples/games/motor_3d_nucleo.st`. El cubo contiene
8 vértices, 36 índices y 12 triángulos. `Malla3D.validar()` rechaza índices fuera del
rango o datos que no forman ternas. El integrador limita el delta a un segundo para
evitar saltos accidentales y no actualiza cuerpos estáticos.

### Frontera actual

Este módulo ST por sí solo **no renderiza píxeles ni abre dispositivos de audio**.
El crate opcional `crates/pyforge_engine3d` ya aporta una ruta gráfica y salida de
audio separadas. La separación mantiene pequeños los programas de consola y permite
instalar PyForge aunque un equipo no tenga GPU o servidor de audio. Todavía falta
una ABI persistente que exponga esas operaciones nativas directamente a ST.

## 36. Backend gráfico y editor 3D opcionales

`crates/pyforge_engine3d` mantiene el código gráfico fuera del compilador principal.
Los programas de terminal y los ejecutables ST que no lo usan conservan su tamaño y
no necesitan controladores gráficos. El paquete instala tres binarios adicionales:
`pyforge-engine3d`, `pyforge-editor3d` y `pyforge-game3d`. La CLI inicia los dos
primeros de esta forma:

```bash
pyforge motor3d --vulkan
pyforge motor3d --opengl
pyforge motor3d --editor
pyforge motor3d --juego ruta/al/binario-st
pyforge motor3d --juego cristales ruta/al/binario-st
```

El renderer utiliza `winit` para la ventana y `wgpu` para negociar Vulkan u
OpenGL/GLES. La ruta verificada crea una superficie, elige formato sRGB y modo de
presentación, sube vértices, índices y una textura RGBA, crea sampler y bind group,
valida WGSL, dibuja y presenta cada cuadro. El redimensionado vuelve a configurar la
superficie. Un fallo interno del controlador se convierte en diagnóstico en lugar de
derribar silenciosamente el IDE.

El crate incluye importadores OBJ triangulado y glTF 2.0, texturas PNG/JPEG, shaders
WGSL, poses esqueléticas, matrices de skinning y mezcla WAV estéreo espacial. Cada
entrada tiene límites de bytes, dimensiones, vértices, articulaciones o duración.
El formato JSON de escena lleva versión, rechaza campos desconocidos y limita el
número de entidades.

El editor ofrece menús e iconos, jerarquía buscable, inspector de nombre, posición,
rotación, escala, malla y material, selección y un historial acotado. Permite crear,
duplicar y eliminar entidades, enfocar la selección, ocultar la rejilla o los nombres
y redimensionar la jerarquía, el inspector y el panel inferior. Abre y guarda
archivos `.stscene`; valida antes de guardar y publica mediante reemplazo atómico.
Su región central contiene un viewport en perspectiva con rejilla, cámara orbital,
selección de cajas y gizmos manipulables de traslación, rotación y escala. Este
viewport usa el pintor de `egui`; proyecta caras sólidas, aplica sombreado sencillo
por cara, contornos y una paleta por nombre de material. Todavía no incrusta la
salida texturizada del renderer PBR.

### Estado verificado

- Vulkan abre la ventana y presenta el pipeline texturizado en este equipo.
- El pipeline actual dibuja tres instancias, prueba profundidad y aplica iluminación
  PBR básica. Todavía no equivale a PBR completo ni incorpora un mapa de sombras.
- La superficie se reconfigura tras pérdida u obsolescencia; un timeout omite el
  cuadro y la falta de memoria detiene el render con diagnóstico.
- OpenGL se compila como alternativa. El controlador disponible en este equipo
  falla al crear su superficie; PyForge devuelve un error controlado y recomienda
  Vulkan o actualizar Mesa.
- La mezcla espacial carga WAV y calcula canales estéreo. `audio::open_output` envía
  PCM S16LE a `pw-cat` o `aplay`; `AudioBackend::Auto` prefiere PipeWire y usa ALSA
  como alternativa. La reproducción depende de esos clientes y de un dispositivo
  disponible, y aún no se expone directamente a código ST.
- OBJ/glTF se importan en memoria; aún falta conectar todos sus materiales,
  animaciones y texturas al grafo de escena visible.

Estas limitaciones evitan llamar “motor profesional completo” a una primera ruta
vertical. Sombras, IBL, PBR completo, múltiples mallas importadas visibles, streaming,
animación glTF en GPU, recuperación completa de dispositivo y un viewport PBR dentro
del editor siguen requiriendo trabajo y pruebas de rendimiento.

## 37. Viewport, audio y Portal Runner visual

El viewport integrado mantiene su cámara y sus transformaciones fuera de `egui`, de
modo que las operaciones se prueban sin abrir una ventana. Sus controles son:

| Acción | Control |
| --- | --- |
| Orbitar | Arrastrar con botón derecho |
| Desplazar objetivo | Arrastrar con botón central |
| Zoom | Rueda del ratón |
| Mover entidad | `W` y arrastrar un eje |
| Rotar entidad | `E` y arrastrar un eje |
| Escalar entidad | `R` y arrastrar un eje |

Las transformaciones comienzan desde una copia estable para no acumular error en cada
cuadro. La escala se limita a un mínimo positivo y los cuaterniones se normalizan.
Una manipulación completa añade un único punto al historial de deshacer.

Para probar la salida física con un WAV:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --example audio_espacial -- sonido.wav auto
```

Para inspeccionar un recurso sin renderizarlo:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --example inspeccionar_recurso -- modelo.gltf
```

El ejemplo `examples/games/portal_runner_visual` mantiene en ST el movimiento, los
portales, los niveles y la puntuación. La ventana Rust captura las teclas y ejecuta
una transición ST corta por movimiento. Esta arquitectura evita crecimiento de
memoria en el runtime persistente experimental, a costa de crear un proceso por
acción; no debe usarse como bucle definitivo para un juego de alto rendimiento.

```bash
cargo build --release
export PATH="$PWD/target/release:$PATH"
examples/games/portal_runner_visual/test_logic.sh
examples/games/portal_runner_visual/build_and_run.sh
```

La guía detallada está en `examples/games/portal_runner_visual/README.md`. Para
ampliar el protocolo se deben cambiar juntos la salida de `src/main.st` y el parser
de `game_main.rs`; una discrepancia se devuelve como error y no se interpreta como
estado parcial.

## 38. Cuatro juegos 3D gráficos

`pyforge-game3d` funciona ahora como ejecutor común de cuatro perfiles. Todos usan
la misma ventana, cámara, HUD y escena sólida; las reglas de cada partida están en
su propio `src/main.st`:

| Perfil | Proyecto | Objetivo y finalización |
| --- | --- | --- |
| `portales` | `portal_runner_visual` | Alcanzar cinco portales; 100 puntos por portal |
| `cristales` | `cristales_3d` | Recoger cinco cristales evitando dos drones y rocas |
| `laberinto` | `laberinto_3d` | Llegar a la salida antes de 120 turnos sin cruzar muros |
| `guardianes` | `guardianes_3d` | Defender el núcleo durante seis rondas y atacar drones |

Desde una instalación, compila la lógica y elige el perfil:

```bash
cd examples/games/cristales_3d
pyforgec build src/main.st -o juego
pyforge motor3d --juego cristales ./juego
```

Los controles comunes son `WASD` o flechas, `R` para reiniciar y `Q`/`Escape` para
salir. Guardianes utiliza además `Espacio` o `Enter` para atacar. El botón derecho
orbita la cámara, el central desplaza su objetivo y la rueda controla el zoom.

Cada proyecto incluye condiciones de victoria, derrota y reinicio. La guía de juego
está en `examples/games/JUEGOS_3D.md`. Son juegos 3D de cuadrícula dentro del renderer
sólido actual; no deben confundirse con juegos PBR de modelos glTF, sombras o
animación esquelética.

## 39. LLVM 21 como backend único

PyForge compila la gramática propia de ST mediante un único recorrido de generación:
Inkwell 0.9 construye módulos LLVM 21, LLVM los verifica y, cuando corresponde,
aplica `default<O2>`. Se retiraron el selector `--backend`, el enum público de
generadores y la construcción sin LLVM. El subconjunto Python beta comparte estas
etapas posteriores, pero ST conserva lexer, parser y módulos propios.

```text
fuente .st
  → lexer y parser ST → AST → análisis semántico
  → IR de pila → optimización ST → verificador
  → lowering Inkwell → verificación LLVM → default<O2>
  → ensamblador → runtime ST → GNU as/ld → ELF Linux x86-64
```

### Construir el compilador

LLVM es una dependencia obligatoria. En Ubuntu/Debian:

```bash
sudo apt install llvm-21-dev libllvm21 binutils
LLVM_SYS_211_PREFIX=/usr/lib/llvm-21 cargo build --release
```

`LLVM_SYS_211_PREFIX` debe contener `bin/llvm-config`. Los scripts
`build-release.sh`, `demo.sh` y `tests/smoke.sh` cargan `scripts/llvm-env.sh`, que
respeta una ruta definida por el usuario y también reconoce la copia local de
desarrollo bajo `target/tools/llvm-21`. No guardes una ruta personal en el código.

### Compilar e inspeccionar ST

```bash
pyforgec check examples/functions.st
pyforgec emit-ir examples/functions.st -o programa.ir
pyforgec emit-llvm examples/functions.st -o programa.ll
pyforgec emit-asm examples/functions.st -o programa.s
pyforgec build examples/functions.st -o programa
pyforgec run examples/functions.st
pyforgec language-json

# Proyecto con PyForge.toml
pyforge check
pyforge build
pyforge run
```

`--no-opt` desactiva el optimizador ST y el pipeline LLVM O2. `--keep` conserva
`.s` y `.o`. `emit-ir` muestra la IR de PyForge y `emit-llvm` la IR que recibe el
generador de máquina. `--backend` se rechaza con un diagnóstico porque LLVM ya no
es una elección opcional.

`language-json` devuelve las palabras de `KEYWORDS` y `CONTEXTUAL_WORDS` del
propio binario en un JSON pequeño. PyForge IDE lo consulta, sin shell y con tiempo
límite, para que una palabra nueva aparezca al recompilar e instalar el compilador.
Vic genera el mismo catálogo desde el checkout durante `build.rs` y conserva una
copia portable actualizada por `vic_ide/scripts/sincronizar_pyforge.py`.

### Dónde se implementa

| Archivo o función | Responsabilidad |
| --- | --- |
| `Cargo.toml` | Fija Inkwell y LLVM 21 como dependencias obligatorias. |
| `src/driver.rs`, `compile_program` | Ejecuta siempre verificación y emisión LLVM. |
| `src/codegen/llvm.rs`, `emit_linux_artifacts` | Crea el target machine, verifica, optimiza y emite IR/ensamblador. |
| `lower_module` | Declara funciones antes de sus cuerpos para recursión y llamadas adelantadas. |
| `lower_function` | Convierte instrucciones verificadas en bloques y operaciones LLVM. |
| `runtime_call` | Declara auxiliares del runtime con su ABI de palabras de 64 bits. |
| `src/codegen/llvm_compat.rs` | Aporta el runtime ensamblador y la transición privada de excepciones. |
| `src/bin/pyforgec.rs` | Expone check, IR, LLVM IR, ensamblador, build, run y el catálogo JSON del lenguaje. |
| `src/bin/pyforge.rs` | Compila proyectos siempre mediante LLVM. |
| `tests/llvm_backend.rs` | Ejecuta funciones, control, colecciones y excepciones en ELF real. |
| `scripts/verify-projects.py` | Compila 80 proyectos con optimización activada y desactivada. |
| `scripts/package.py` | Exige LLVM, prueba el único recorrido y declara `libllvm21`. |

La IR de PyForge es una máquina de pila. `lower_function` crea posiciones locales
con `alloca` para unir el flujo con seguridad; O2 las promueve a SSA y crea los
nodos `phi`. Las operaciones enteras usan auxiliares comprobados para conservar los
diagnósticos de división por cero y desbordamiento.

### Compatibilidad privada de excepciones

La ABI existente guarda el manejador, `rsp` y `rbp`, y salta a una etiqueta concreta.
Por eso `uses_native_exception_abi` detecta `intentar`, `capturar`, `finalmente` y
`lanzar`; `emit_compatibility_module` coloca esa implementación como ensamblador
inline dentro del módulo LLVM. No es un segundo backend ni se puede seleccionar por
CLI. Se conserva para no cambiar la limpieza de retornos y bucles hasta disponer de
una ABI de errores LLVM completamente probada.

`src/codegen/llvm_compat.rs` también contiene datos y funciones del runtime que LLVM
llama. Borrar ese archivo ahora rompería cadenas, colecciones, entrada, recursos y
excepciones. Para retirarlo hay que migrar cada símbolo a Rust/LLVM, comprobar su
ABI y ejecutar toda la matriz antes de eliminarlo.

### Agregar sintaxis o una instrucción

1. Define su gramática, tipo, resultado, errores y efecto de pila.
2. Actualiza lexer/parser ST, AST, semántica, lowering, IR y verificador.
3. Añade el caso LLVM en `lower_function` o un auxiliar mediante `runtime_call`.
4. Revisa desbordamientos, punteros, alineación, propiedad y errores antes de emitir.
5. Añade casos válidos y rechazados, con O2 activado y desactivado.
6. Ejecuta el catálogo y actualiza manual, curso y ejemplos.

```bash
LLVM_SYS_211_PREFIX=/usr/lib/llvm-21 cargo test --all-targets
python3 scripts/verify-projects.py --compiler target/debug/pyforgec
python3 -m unittest discover -s tests -p 'test_*.py' -q
```

### Límites reales

- El runtime y el enlace siguen limitados a Linux x86-64. Inkwell incluye el target
  AArch64, pero ST todavía no tiene runtime, enlace, pruebas ni paquete AArch64.
- Windows ejecuta el compilador Linux dentro de WSL; aún no genera PE nativo.
- LLVM optimiza y genera máquina; no aporta automáticamente propiedad, préstamos ni
  seguridad de memoria. Esas reglas deben comprobarse en ST y en su runtime.
- PyForge depende dinámicamente de `libLLVM.so.21.1`. No existe edición mínima sin
  LLVM. Inkwell está fijado en `Cargo.lock` y toda actualización exige la matriz.
- La ruta inline de excepciones pasa por un módulo LLVM, pero su cuerpo todavía no
  se convierte a SSA ni recibe las optimizaciones de LLVM.

## 40. Editor 3D controlado desde archivos ST

`importar "editor3d";` carga un módulo estándar escrito en ST que publica una
escena para `pyforge-editor3d`. Este recorrido convierte el archivo `.st` en la
fuente editable de la escena: el usuario puede crearlo con código, aplicarlo en el
viewport, mover objetos con gizmos y exportar las transformaciones de vuelta a ST.

```st
importar "editor3d";

editor3d.iniciar("Habitación"; 2);

editor3d.entidad("Suelo"; "plano"; "suelo");
editor3d.posicion(0.0; -0.1; 0.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(8.0; 0.2; 8.0);

editor3d.entidad("Jugador"; "cubo"; "jugador");
editor3d.posicion(0.0; 0.75; 0.0);
editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
editor3d.escala(1.0; 1.5; 1.0);

editor3d.terminar();
```

`iniciar` recibe el nombre y la cantidad exacta de entidades. Para cada entidad
se escriben cuatro llamadas consecutivas: metadatos, posición XYZ, cuaternión XYZW
y escala XYZ. Esta división respeta el máximo actual de seis parámetros por función
en el ABI de ST. Una cadena vacía en malla o material representa `None` en la escena.

### Flujo dentro del editor

1. Ejecuta `pyforge motor3d --editor`.
2. Abre **ST Script de escena** en el panel inferior.
3. Escribe la ruta del `.st`, pulsa **Abrir** y revisa el código.
4. Pulsa **Aplicar ST**. La compilación ocurre en un hilo para no congelar la ventana.
5. Selecciona una entidad y usa `W`, `E` o `R` para mover, rotar o escalar.
6. Pulsa **Exportar visual** para guardar la escena actual como llamadas ST.

La ruta de `pyforgec` es configurable. El editor busca primero
`PYFORGE_COMPILER`, después un ejecutable instalado junto a sí y finalmente
`target/debug/pyforgec` durante el desarrollo. El script se guarda de forma atómica
antes de compilar. La recarga automática es opcional y empieza desactivada.

### Ventana y controles

La ventana incluye menú, barra de iconos, jerarquía con filtro, viewport, inspector,
consola, editor ST y ayuda. Los paneles laterales e inferior son redimensionables y
pueden ocultarse desde **Ventana**.

Cada zona tiene un nombre visible. **Jerarquía** a la izquierda enumera objetos;
**Viewport 3D** ocupa el centro; **Inspector** a la derecha edita la selección;
**Consola**, **Script ST** y **Ayuda** comparten el panel inferior. La ayuda se abre
al iniciar para explicar la ventana. La barra superior usa texto explícito:
**[+] Nuevo**, **[O] Abrir**, **[S] Guardar**, **Deshacer**, **Rehacer**,
**[D] Duplicar** y **[X] Eliminar**. Los controles del viewport se llaman
**W · Mover**, **E · Rotar**, **R · Escalar**, **Rejilla**, **Nombres** y
**Restaurar vista**.

La interfaz de `egui` usa Glow para evitar que ciertos controladores Mesa/Vulkan
conviertan el atlas de letras en rectángulos. El renderer 3D independiente conserva
`wgpu` y su selección Vulkan/OpenGL; cambiar la interfaz no cambia ese pipeline.

| Acción | Control |
| --- | --- |
| Seleccionar | Clic en el volumen o en la jerarquía |
| Enfocar | `F`, botón **Enfocar** o doble clic en la jerarquía |
| Mover / Rotar / Escalar | `W` / `E` / `R` y arrastrar X, Y o Z |
| Orbitar / Desplazar / Zoom | Botón derecho / central / rueda |
| Duplicar / Eliminar | `Ctrl+D` / `Supr` |
| Deshacer / Rehacer | `Ctrl+Z` / `Ctrl+Y` |
| Guardar documento activo | `Ctrl+S` |

El inspector edita posición, rotación en grados, escala, nombre de malla y material.
La escena se puede guardar además como `.stscene` JSON versionado para intercambio
interno. El icono del ejecutable está incrustado desde el mismo rayo amarillo y
negro usado por la web y las asociaciones `.st`.

### Validación y ejecución

El editor no interpreta la salida libre como una escena. Exige las marcas
`PYFORGE_EDITOR3D_V1` y `FIN_PYFORGE_EDITOR3D_V1`, el número declarado de entidades
y trece líneas válidas por entidad. Limita el fuente a 4 MiB, la salida a 4 MiB,
la escena interactiva a 10 000 entidades, la compilación a 90 segundos y la
ejecución a 15 segundos. Rechaza números no finitos, escalas nulas, cuaterniones
vacíos, nombres de control y resultados incompletos. Una falla conserva la escena
que ya estaba abierta y aparece en la consola.

El proceso se crea con argumentos separados y sin shell. Estas comprobaciones
protegen el formato y la capacidad de respuesta; un `.st` sigue siendo código y
puede importar módulos con permisos de archivos, procesos o red. **Aplicar ST** y
la recarga automática deben usarse con fuentes de confianza.

### Archivos de implementación

| Archivo | Responsabilidad |
| --- | --- |
| `src/stdlib/editor3d.st` | API pública que escribe el protocolo. |
| `src/st/stdlib.rs` | Registra el módulo incrustado para compilación y autocompletado. |
| `crates/pyforge_engine3d/src/st_scene.rs` | Compila, limita, analiza y exporta escenas ST. |
| `crates/pyforge_engine3d/src/editor.rs` | Ventana, comandos, paneles, inspector e historial. |
| `crates/pyforge_engine3d/src/editor_viewport.rs` | Cámara, selección, rejilla y gizmos. |
| `examples/editor3d/escena_profesional.st` | Escena de cinco entidades completamente en ST. |
| `examples/editor3d/README.md` | Tutorial corto y controles reproducibles. |

Prueba el puente sin abrir una ventana:

```bash
cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
  --example validar_escena_st -- \
  target/debug/pyforgec examples/editor3d/escena_profesional.st
```

La ventana de la captura titulada **Portal Runner 3D · PyForge ST** pertenece a
`pyforge-game3d`; muestra un juego y su HUD. La herramienta de edición se abre con
`pyforge-editor3d` o `pyforge motor3d --editor` y es la que contiene los paneles
descritos aquí.

El viewport sigue siendo una representación sólida de cajas. Los marcadores `luz`
y `camara` se editan y exportan, pero todavía no controlan luces ni cámaras del
renderer GPU. Las mallas OBJ/glTF, PBR, sombras y animación esquelética continúan
en la ruta gráfica separada documentada en la sección 36.
