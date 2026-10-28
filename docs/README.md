# ST / PyForge: definición técnica y plan de desarrollo

**Documento de diseño.** Define la dirección propuesta del proyecto y el trabajo
necesario para alcanzarla. No declara que las capacidades futuras estén implementadas.
La arquitectura actual se conserva al final de este documento.

| Documento | Qué encontrarás |
| --- | --- |
| [Manual de ST](MANUAL_ST.md) | Sintaxis disponible, ejemplos y límites actuales. |
| [Instalación e IDE](INSTALACION.md) | Instaladores, editor, privacidad y publicación. |
| [Integraciones](INTEGRACIONES.md) | API disponible, dependencias, permisos, GUI y web. |
| Este README | Visión, decisiones propuestas, etapas y arquitectura del compilador. |

## Avance comprobable: LLVM 21 como backend único

El compilador usa obligatoriamente Inkwell 0.9 y LLVM 21. Baja la IR verificada a
funciones LLVM, verifica el módulo y aplica `default<O2>`. `pyforgec emit-llvm`
permite inspeccionar la IR resultante. El selector `--backend` y el backend público
anterior fueron retirados para que archivos y proyectos sigan un único recorrido.

El catálogo se prueba con LLVM y con optimización activada/desactivada.
Las excepciones conservan temporalmente su ABI ensambladora dentro del módulo LLVM;
el runtime final sigue siendo Linux x86-64. LLVM no sustituye el trabajo pendiente
de propiedad, gestión general de memoria ni plataformas adicionales. La sección 39
del [manual](MANUAL_ST.md) explica construcción, comandos, código y límites.

## Avance comprobable: herramientas 3D opcionales

El crate separado `crates/pyforge_engine3d` compila un renderer de demostración,
un editor y un ejecutor visual para juegos ST. El editor ya tiene viewport sólido,
cámara orbital, selección y gizmos de movimiento, rotación y escala. Ahora también
ofrece menús con iconos, jerarquía buscable, inspector de transformación y recursos,
consola, paneles redimensionables, atajos e historial. El módulo `editor3d` deja que
un `.st` cree la escena; el editor compila, valida y aplica ese documento, y puede
exportar los cambios visuales de vuelta a ST. La biblioteca de audio mezcla WAV
espacial y puede enviar PCM a PipeWire o ALSA mediante los clientes del sistema.
`examples/games/portal_runner_visual` conserva las reglas en ST y muestra el estado
en una ventana nativa.

Este avance no conecta todavía el grafo de escena al pipeline PBR: las mallas OBJ/glTF,
materiales y animaciones se importan o calculan en CPU, pero no se ven en el viewport.
Tampoco existen mapas de sombras, IBL ni streaming asíncrono de recursos. El estado
detallado y los comandos reproducibles están en las secciones 35–38 y 40 del manual.

## Avance comprobable: primer incremento nativo

Se implementó una primera ABI Rust experimental para archivos y buffers binarios,
expuesta mediante `importar "nativo";`. Hay cierre explícito, identificadores no
reutilizados, límites y errores recuperables. El enlace es nativo y condicional;
los módulos Python anteriores se conservan y pueden convivir con este módulo.
Consulta la sección 32 del [manual](MANUAL_ST.md) para API, permisos y límites.

Esto completa un primer recorrido del apartado 19, no las etapas 1–3 enteras:
los recursos todavía son identificadores enteros, no tipos propios con préstamos;
la gestión general de memoria, concurrencia, red y gráficos del plan siguen
pendientes. La reserva de buffers es fija y el runtime monohilo. Las cadenas
convertidas conservan la vida útil histórica del heap ST.

El ejemplo binario se ejecutó dentro de un entorno vacío con solo el ejecutable y
un directorio temporal: no había Python ni bibliotecas compartidas. Las pruebas
cubren ambas optimizaciones, cierre, alias inválidos, UTF-8, límites y archivos.

El incremento siguiente añade `archivo_nativo`, `buffer_nativo` y el bloque
contextual `usar`. La propiedad nueva se distingue de los préstamos en las firmas;
el compilador rechaza copias y escapes, y genera cierre en todas las salidas del
bloque. La ABI Rust sigue comprobando cada identificador. Consulta la sección 33
del manual. Esto es una base de propiedad para recursos concretos, todavía no un
sistema general de préstamos, concurrencia o gestión completa de memoria.

## 1. Definición del lenguaje que queremos construir

**ST será un lenguaje de propósito general, compilado a código nativo, con sintaxis
en español, tipos verificables y herramientas integradas. Su plataforma permitirá
construir aplicaciones de escritorio, servicios web, herramientas de sistemas y
juegos 2D/3D mediante bibliotecas públicas coherentes y un runtime nativo.**

PyForge será el conjunto de herramientas: compilador, gestor de proyectos,
bibliotecas, pruebas, análisis del editor y distribución. El motor de juegos será
una biblioteca y un conjunto de herramientas sobre ST; no una gramática distinta.
El framework web utilizará la misma base de tipos, tareas, errores y recursos.

La prioridad será que un programa correcto tenga un comportamiento definido,
que un programa incorrecto produzca un diagnóstico útil y que las capacidades se
puedan mantener entre versiones. Más palabras reservadas no equivalen a un lenguaje
más potente. Una biblioteca bien diseñada puede ampliar ST sin cambiar su sintaxis.

### Qué significa «propio»

El proyecto controlará la especificación, el compilador, el runtime, las interfaces
ST, las herramientas y la política de compatibilidad. Se podrán implementar
bibliotecas en ST y usar componentes nativos escritos en Rust mediante contratos
explícitos. El runtime de producción propuesto no necesitará iniciar Python para
realizar operaciones básicas o llamar a las bibliotecas migradas.

Esto no exige crear un sistema operativo, controladores GPU, protocolos ni
primitivas criptográficas desde cero. Las dependencias nativas deben estar
identificadas, fijadas, revisadas y sujetas a sus licencias. «Sin Python» y «sin
ninguna dependencia externa» son objetivos distintos.

### Qué significa «completo»

No existe un porcentaje universal que convierta un lenguaje en capaz de hacer
cualquier cosa. Para ST, **1.0 significará cumplir un alcance publicado y verificable**:
especificación estable, runtime definido, bibliotecas documentadas, plataformas
probadas y herramientas distribuibles. El motor 3D tendrá su propia versión y
criterios de publicación. Una versión estable del lenguaje no implica un motor AAA.

## 2. Punto de partida real

| Área | Disponible actualmente | Trabajo previsto |
| --- | --- | --- |
| Compilación | AOT mediante LLVM 21/Inkwell a ELF Linux x86-64; Windows mediante WSL | Runtime, ABI, enlace y paquetes para destinos adicionales. |
| Núcleo ST | Funciones, clases nominales, excepciones explícitas, patrones literales | Tipos y comportamientos especificados; capacidades avanzadas coherentes. |
| Colecciones | Listas tipadas o mixtas, mutables y ordenadas; diccionarios homogéneos anidados | Diccionarios mixtos, genéricos, iteradores y operaciones completas. |
| Memoria | Presupuestos y comprobaciones del runtime actual | Liberación de recursos y gestión de objetos de larga duración. |
| Bibliotecas | Módulos nativos iniciales y bibliotecas locales `@nombre` | Paquetes versionados y ABI de extensiones. |
| Servicios avanzados | Varias integraciones mediante auxiliar Python y JSON | Implementaciones nativas con valores y errores tipados. |
| Web | Puente Django Python con lógica ST; foro independiente | Framework nativo conectado de extremo a extremo a ST. |
| Juegos | Reglas 2D, núcleo 3D CPU, demostración Vulkan/OpenGL, audio PCM, editor con viewport sólido/gizmos y cuatro juegos de cuadrícula con reglas ST | Unificar grafo y renderer; sombras, IBL, animación glTF visible, streaming, física y herramientas de producción. |
| Herramientas | CLI, IDE y extensión de VS Code | Diagnósticos estructurados, formateador, depuración y perfilado. |

La base Rust `crates/pyforge_django` aún no equivale a un framework accesible mediante
un DSL ST completo. `motor3d` aporta el modelo y la simulación iniciales; el crate 3D
separado sí presenta una demostración mediante GPU, pero todavía no consume ese grafo
ST como un motor integrado. Los ejemplos del manual muestran la frontera exacta.

## 3. Arquitectura objetivo

```text
Aplicaciones escritas en ST
  ├── servicios web y herramientas
  ├── aplicaciones de escritorio
  └── juegos y editor de escenas
                │
     bibliotecas públicas de ST
                │
     valores, errores y recursos tipados
                │
     runtime nativo con interfaz estable
       ├── memoria y tareas
       ├── archivos, procesos y red
       ├── SQL, TLS y criptografía
       └── ventana, GPU, audio y dispositivos
                │
       sistema operativo y controladores

Fuentes ST → AST tipado → IR → optimización → backend → ejecutable
```

JSON seguirá disponible como formato de datos para HTTP, configuración e
intercambio. No será obligatorio para representar una cadena, un objeto o una
petición entre ST y su runtime. La representación en memoria y la serialización
externa se diseñarán por separado.

El compilador, el runtime, las bibliotecas y los recursos distribuidos deberán
identificar sus versiones compatibles. Una actualización no debe enlazar de forma
silenciosa estructuras con tamaños o contratos diferentes.

## 4. Decisiones de diseño que deben cerrarse primero

Estas son propuestas para revisar y especificar antes de introducir cambios
incompatibles. No son reglas ya incorporadas al compilador.

| Decisión | Dirección propuesta |
| --- | --- |
| Tipado | Estático con inferencia local; lo dinámico, si se añade, será explícito. |
| Números | Semántica definida de tamaño, desbordamiento, conversiones y flotantes. |
| Datos | Estructuras, enumeraciones con datos, genéricos y colecciones tipadas. |
| Ausencia y errores | Distinguir ausencia de valor, resultado fallido y fallo fatal del runtime. |
| Funciones | Valores de función, cierres y contratos de captura y vida útil. |
| Memoria | Propiedad y liberación determinista como objetivo, préstamos comprobados y memoria compartida explícita. |
| Recursos | Archivos, sockets, tareas y GPU con cierre definido incluso ante errores. |
| Concurrencia | Tareas cancelables; reglas comprobables para compartir datos entre ellas. |
| Interoperabilidad | Frontera nativa pequeña y documentada; operaciones inseguras aisladas. |
| Compatibilidad | Versiones de lenguaje y migraciones explícitas cuando cambie el significado. |

La propiedad y los préstamos suponen un trabajo importante en el sistema de tipos.
No se obtienen por escribir el runtime en Rust. Las referencias mutables compartidas
que hoy permite ST necesitarán una estrategia de transición; no se deben reinterpretar
silenciosamente bajo reglas nuevas.

Si una etapa inicial emplea conteo de referencias o arenas, deberá documentar ciclos,
liberación, latencias y límites. No se anunciarán garantías de préstamos hasta que
el compilador las compruebe. El renderizado y las tareas también necesitan reglas
para recursos que siguen en uso fuera del hilo que los creó.

## 5. Etapa 0 — Fijar una base compatible

1. Registrar las capacidades reales y las incompatibilidades conocidas.
2. Separar ejemplos ejecutables de propuestas de sintaxis.
3. Mantener pruebas de programas existentes, errores y ambas optimizaciones.
4. Registrar plataformas, dependencias y comportamiento de las versiones publicadas.
5. Conservar el puente Python mientras se construyen reemplazos equivalentes.

**Entregable:** una línea base reproducible y un inventario de APIs.
**Aceptación:** los programas existentes conservan sus resultados o reciben una
migración documentada; ninguna funcionalidad desaparece por sustituir el runtime.

## 6. Etapa 1 — Especificar y completar el núcleo

1. Definir gramática, precedencia, ámbitos, evaluación y conversión de tipos.
2. Especificar cadenas UTF-8, bytes, índices, colecciones y mutabilidad.
3. Diseñar enumeraciones con datos, genéricos e iteradores.
4. Definir funciones como valores y cierres antes de diseñar callbacks web o de juego.
5. Definir el comportamiento de excepciones y la limpieza de recursos.
6. Conservar posiciones de origen desde los tokens hasta los diagnósticos y la IR.

**Entregable:** especificación del núcleo y pruebas de conformidad vinculadas a
cada regla. **Aceptación:** los errores indican archivo, posición, causa y contexto;
las optimizaciones no cambian el comportamiento definido.

No se debe agregar sintaxis de modelos, eventos o concurrencia antes de que sus
valores y reglas puedan representarse en AST, tipos, IR y runtime.

## 7. Etapa 2 — Memoria y recursos de larga duración

1. Diseñar asignación, propiedad, transferencias, préstamos y destrucción.
2. Definir cómo se comparten objetos y cómo se evitan o gestionan los ciclos.
3. Asegurar limpieza al retornar, cancelar y propagar errores.
4. Separar memoria temporal por frame de datos persistentes de una aplicación.
5. Añadir seguimiento de reservas y pruebas de crecimiento sostenido.
6. Revisar todos los accesos del ensamblador y las futuras llamadas nativas.

**Entregable:** modelo de memoria comprobable, no solo un límite de consumo.
**Aceptación:** una aplicación de larga duración puede liberar recursos y repetir
operaciones sin crecimiento injustificado; las pruebas cubren alias, ciclos, cierres,
fallos y cancelación. Se documentan las garantías que dependen de código externo.

## 8. Etapa 3 — Runtime nativo e interoperabilidad

1. Crear una capa nativa versionada, inicialmente en Rust.
2. Definir ABI, representación de valores, alineación y propiedad de los argumentos.
3. Representar cadenas y bytes con longitud; no depender de terminadores implícitos.
4. Usar referencias o identificadores comprobados para recursos externos.
5. Evitar que excepciones o fallos crucen la frontera nativa sin traducción definida.
6. Añadir al enlazado del compilador la selección de runtime y bibliotecas compatibles.
7. Documentar callbacks, hilos permitidos y quién libera cada resultado.

**Entregable:** un programa ST que llame a una biblioteca nativa sin proceso Python
ni puente JSON. **Aceptación:** se prueban resultados, errores, tamaños inválidos,
recursos cerrados, incompatibilidad de versión y liberación.

Los comandos y nombres de crates de esta etapa se definirán durante la implementación;
este documento no anuncia una API FFI ya disponible en ST.

## 9. Etapa 4 — Biblioteca estándar nativa

Migrar por áreas pequeñas, conservando firmas compatibles cuando su contrato lo permita:

1. Bytes, texto, colecciones e iteradores.
2. Archivos y directorios, rutas, flujos y cierre de recursos.
3. Tiempo, aleatoriedad y procesos controlados.
4. TCP/UDP, resolución de nombres, plazos y cancelación.
5. Clientes seguros, criptografía de alto nivel y gestión de claves.
6. SQLite, consultas parametrizadas, transacciones y migraciones.
7. Serialización y formatos documentados.

**Entregable:** módulos ST con backend nativo y pruebas de contrato compartidas con
las implementaciones anteriores. **Aceptación:** la aplicación de prueba funciona
en un entorno sin Python; se publican sus otras dependencias reales.

No elimines una integración hasta documentar diferencias de codificación, límites,
permisos y errores. Documentos de oficina, PDF y GUI pueden migrar después del núcleo;
cada uno necesita una biblioteca y pruebas propias.

## 10. Etapa 5 — Tareas y concurrencia

1. Diseñar tareas con cancelación, plazos y propiedad de recursos.
2. Definir qué valores pueden cruzar hilos y cómo se sincronizan.
3. Añadir canales acotados y control de presión: los productores no pueden acumular
   datos indefinidamente cuando el consumidor es lento.
4. Separar operaciones bloqueantes del bucle de eventos.
5. Garantizar cierre ordenado, propagación de errores y observabilidad.

**Entregable:** servidor y cargador de recursos concurrentes sobre la misma base.
**Aceptación:** pruebas de desconexión, timeout, saturación, cancelación repetida y
cierre sin recursos pendientes. Más hilos no sustituyen las reglas de memoria.

## 11. Etapa 6 — Framework web nativo

Construirlo sobre los tipos, funciones como valores, red y tareas anteriores:

1. Petición y respuesta tipadas, rutas y middleware con orden definido.
2. Límites de cuerpo, cabeceras y tiempo; tratamiento explícito de proxies confiables.
3. Plantillas con escape por defecto y APIs diferenciadas para contenido confiable.
4. Acceso a SQL con parámetros, transacciones y migraciones versionadas.
5. Sesiones, autenticación y autorización por recurso.
6. Protección de operaciones que cambian estado, gestión de archivos subidos y cuotas.
7. Configuración separada para desarrollo y producción; secretos fuera del código.
8. Logs sin credenciales, métricas, errores públicos acotados y apagado ordenado.

**Entregable:** una aplicación completa escrita en ST: usuarios, permisos, formularios,
datos persistentes y plantillas, sin servidor Python.
**Aceptación:** pruebas de aislamiento entre usuarios, entradas hostiles, operaciones
concurrentes, migración, respaldo/restauración y comportamiento con recursos agotados.

Ningún framework convierte automáticamente toda aplicación en segura. El proyecto
necesitará modelo de amenazas, revisión de dependencias, auditorías y mantenimiento.
Las primitivas criptográficas deben proceder de implementaciones especializadas;
crear una API ST propia no exige inventar algoritmos de cifrado.

## 12. Etapa 7 — Plataforma gráfica nativa y juego 2D

Antes de un motor 3D, completar una aplicación gráfica de extremo a extremo:

1. Ventana, eventos, teclado, ratón, mandos y escalado de pantalla.
2. Acceso a GPU mediante una abstracción de backend y presentación de frames.
3. Texturas, geometría básica, texto y recursos con carga y descarga definidas.
4. Audio y mezcla básica.
5. Bucle de simulación con paso fijo y renderizado independiente.
6. Juego 2D completo con menú, pausa, guardado y distribución.

**Entregable:** un juego ejecutable nativo fuera del navegador y del IDE.
**Aceptación:** puede iniciarse, jugarse y cerrarse repetidamente; maneja pérdida de
foco, cambios de tamaño, recursos ausentes y errores de dispositivos.

La elección de bibliotecas gráficas debe documentarse y validarse para los destinos
soportados. Usar una API GPU existente permite concentrar el trabajo en ST y el motor.

## 13. Etapa 8 — Motor de renderizado 3D

La primera columna de matemática, cámara, geometría descriptiva y luces ya tiene una
implementación CPU comprobada en `src/stdlib/motor3d.st`. La tabla conserva el
trabajo del backend gráfico y la evolución necesaria para producción.

| Componente | Alcance inicial | Evolución profesional |
| --- | --- | --- |
| Matemática | Vectores, matrices, cuaterniones y transformaciones | Operaciones por lotes y optimizaciones medidas. |
| Cámara | Perspectiva, cámara libre y plano cercano/lejos | Múltiples cámaras, efectos y control cinematográfico. |
| Geometría | Mallas indexadas y materiales | Instancias, niveles de detalle y partición espacial. |
| Iluminación | Luces y materiales físicamente fundamentados | Sombras, entornos y posprocesado escalables. |
| GPU | Buffers, texturas, shaders y sincronización | Gestión de memoria, streaming y recuperación de errores. |
| Recursos | Importación y formatos internos versionados | Conversión incremental, caché y paquetes de distribución. |

1. Renderizar una malla con cámara y control del usuario.
2. Cargar una escena con varios materiales y recursos externos.
3. Añadir luces, sombras y selección de calidad.
4. Evitar procesar objetos fuera de la vista y medir trabajo CPU/GPU.
5. Gestionar la liberación diferida de recursos que la GPU sigue utilizando.
6. Definir formatos y compatibilidad de escenas, materiales y shaders.

**Entregable:** visor y escena 3D interactiva reproducibles.
**Aceptación:** pruebas de transformaciones, referencias visuales con tolerancias,
recursos inválidos y métricas de tiempo y memoria en hardware identificado.
Una captura de un cubo no constituye un motor de producción.

## 14. Etapa 9 — Motor de juegos y herramientas

La parte gráfica debe integrarse con:

- **Escenas y entidades:** identidad estable, jerarquías, componentes y ciclo de vida.
- **Física:** colisiones, consultas espaciales, cuerpos y sincronización con la simulación.
- **Animación:** esqueletos, mezclas, estados y eventos de animación.
- **Audio:** fuentes espaciales, mezcla, dispositivos y presupuestos.
- **Juego:** entrada configurable, UI, estados, guardado versionado y cargas de nivel.
- **Mundo:** streaming de zonas, recursos y límites de memoria.
- **Herramientas:** editor de escenas, inspector, importador, consola y perfilador.
- **Distribución:** empaquetado de recursos y ejecutable, configuración y diagnóstico.

Un sistema de entidades y componentes es una opción de diseño; deberá probarse
con acceso a datos, serialización y necesidades reales. No requiere convertirse
en una nueva colección de palabras reservadas del lenguaje.

**Entregable:** una muestra vertical jugable que atraviese todos los sistemas:
por ejemplo, un nivel 3D con personaje, cámara, colisiones, sonido, interfaz y guardado.
**Aceptación:** puede distribuirse y jugarse sin el entorno del desarrollador;
los recursos y escenas pueden modificarse sin rehacer el motor.

## 15. Etapa 10 — Multijugador y juegos de mayor escala

Solo después de tener simulación, tareas, red y herramientas de diagnóstico:

1. Diseñar un protocolo versionado con mensajes acotados.
2. Separar autoridad del servidor y presentación del cliente.
3. Añadir replicación de estado, interpolación y reconciliación cuando proceda.
4. Manejar reconexión, pérdida, retraso, duplicados y mensajes fuera de orden.
5. Validar acciones del cliente y limitar recursos por conexión.
6. Probar actualizaciones compatibles, persistencia y recuperación de fallos.

**Entregable:** una partida de prueba con servidor y varios clientes.
**Aceptación:** funciona bajo condiciones de red simuladas y rechaza estados inválidos;
se documentan los límites de jugadores, tráfico y recursos medidos.

Un mundo de bloques y un juego de acción 3D tienen requisitos diferentes. Construir
prototipos de ambos puede servir de validación, pero no da compatibilidad automática
con Minecraft o Call of Duty, sus contenidos ni sus motores.

## 16. Etapa 11 — Ecosistema y herramientas profesionales

1. Formato de paquete, versiones, dependencias y archivo de bloqueo reproducible.
2. Registro opcional con procedencia, integridad y política de retirada de paquetes.
3. Builds que distingan código del proyecto, dependencias y pasos de construcción.
4. LSP para símbolos, diagnósticos, referencias, renombrado y autocompletado semántico.
5. Formateador, análisis estático, pruebas y documentación de API.
6. Información de depuración, puntos de interrupción y trazas de pila.
7. Perfilado CPU, memoria y GPU; informe de rendimiento asociado a versión y equipo.
8. Matriz de plataformas, CI y pruebas de instalación, actualización y desinstalación.

**Entregable:** herramientas utilizables por alguien que no conoce el código del
compilador. **Aceptación:** puede crear, probar, depurar, empaquetar y actualizar
una aplicación con instrucciones reproducibles.

Las plataformas se incorporarán una a una. Windows nativo requiere backend,
enlazado, runtime y pruebas propios; un instalador que utiliza WSL no cubre ese hito.

## 17. Etapa 12 — Publicar una versión estable

Antes de etiquetar ST 1.0:

- Congelar y publicar el alcance y las reglas del lenguaje de esa versión.
- Clasificar todas las APIs como estables, experimentales o internas.
- Resolver defectos bloqueantes y publicar las limitaciones conocidas.
- Probar compatibilidad o proporcionar migraciones para cambios incompatibles.
- Validar bibliotecas, ejemplos, herramientas e instaladores desde fuentes limpias.
- Publicar política de soporte, dependencias, licencias y respuesta a vulnerabilidades.
- Probar aplicaciones de referencia web, escritorio y juegos según el alcance declarado.

El motor 3D se publicará con su propia matriz de capacidades, hardware y rendimiento.
No se marcará una etapa como terminada por haber añadido documentación o una interfaz
vacía. Tampoco se prometerá «cero errores», seguridad absoluta o compatibilidad universal.

## 18. Medir potencia y calidad

El rendimiento se evaluará con aplicaciones y cargas reproducibles, no con adjetivos.
Registrar tamaño del ejecutable, tiempo de compilación, uso de memoria, latencias,
reservas, CPU y GPU. Separar arranque, carga y funcionamiento sostenido.

Para una escena objetivo a 60 FPS, el presupuesto total por frame es aproximadamente
16,67 ms; a 120 FPS, 8,33 ms. Son presupuestos matemáticos, no resultados del motor
actual. Se deben fijar equipo, resolución, calidad y escena antes de exigirlos.
Medir también percentiles y pausas: un promedio aceptable puede ocultar bloqueos.

Para web, definir carga, tamaño de peticiones, operaciones SQL, concurrencia y límites.
Para memoria, probar sesiones largas y repetición de abrir/cerrar recursos. Para
seguridad, combinar pruebas de contratos, entradas malformadas, fuzzing y revisión.
No publicar cifras sin el programa y las condiciones que permitan reproducirlas.

## 19. Primer incremento recomendado

El próximo incremento implementable debe ser pequeño y cerrar un recorrido completo:

1. Especificar bytes, cadenas y un recurso de archivo nativo.
2. Definir la ABI de lectura/escritura, propiedad de buffers y traducción de errores.
3. Implementar esa ABI en Rust y conectarla al enlace de PyForge.
4. Exponerla en un módulo ST con operaciones tipadas.
5. Ejecutar el ejemplo sin Python, probar fallos y comprobar liberación repetida.
6. Mantener el módulo anterior durante la transición y documentar diferencias.

A partir de ese contrato, ampliar a sockets, tareas, servicios y recursos gráficos.
La secuencia evita construir web y 3D sobre representaciones incompatibles.

Los cambios del lenguaje corresponden a `lexer`, `parser`, AST, semántica, IR y
backend; los del runtime a una capa nativa separada; los del motor a bibliotecas
y herramientas. Las secciones siguientes describen los archivos actuales para
comenzar a trabajar sin confundir esta propuesta con una implementación terminada.

---

## Arquitectura actual, ampliación y seguridad de PyForge

Esta guía explica cómo funciona el compilador y dónde cambiar cada capacidad.
Para escribir programas consulta [el manual](MANUAL_ST.md); para instalar y usar
el editor, [instalación](INSTALACION.md); para servicios externos,
[integraciones](INTEGRACIONES.md).

### Recorrido del compilador

```text
fuente .st / subconjunto .py
  -> lexer y parser -> AST -> módulos -> análisis semántico
  -> lowering a IR -> optimización -> verificación de flujo
  -> LLVM 21/Inkwell: verificación, O2 y emisión
  -> ensamblador Linux x86-64 -> GNU as -> GNU ld -> ejecutable ELF
```

El compilador está escrito en Rust; no traduce los programas a Rust, C ni C++.
La generación es AOT, sin VM. Windows utiliza WSL; no se generan ejecutables PE
nativos de Windows. El runtime básico usa llamadas al sistema Linux. Las
integraciones opcionales añaden un auxiliar Python y sus dependencias.

| Archivo | Responsabilidad |
| --- | --- |
| `src/bin/pyforge.rs` | Proyectos, check/build/run, limpieza y aplicaciones web. |
| `src/bin/pyforgec.rs` | Compilación de archivos individuales. |
| `src/driver.rs` | Selección por extensión y coordinación de compilación/enlace. |
| `src/st/lexer.rs` | Tokens, literales, comentarios y tabla `KEYWORDS`. |
| `src/st/parser.rs` | Gramática ST, precedencia, declaraciones y AST. |
| `src/lexer.rs`, `src/parser.rs` | Subconjunto Python experimental independiente. |
| `src/ast.rs` | Expresiones, sentencias y tipos compartidos. |
| `src/st/modules.rs` | Importaciones, alias, ciclos y bibliotecas `@nombre`. |
| `src/st/stdlib.rs`, `src/stdlib/*.st` | Registro e implementación de módulos estándar. |
| `src/semantic.rs` | Nombres, firmas y compatibilidad de tipos. |
| `src/lower.rs`, `src/ir.rs` | Conversión a operaciones de pila y control de flujo. |
| `src/optimizer.rs` | Simplificaciones locales con aritmética comprobada. |
| `src/verify.rs` | Inicialización, retornos, profundidad de pila y manejadores. |
| `src/codegen/llvm.rs` | Único lowering público: verificación, O2 y emisión LLVM. |
| `src/codegen/llvm_compat.rs`, `src/codegen/*.s` | Runtime Linux y transición privada de excepciones. |
| `src/project.rs`, `src/web_project.rs` | Configuración y plantillas de proyectos. |

GNU `as` codifica instrucciones y `ld` enlaza ELF; no interpretan la gramática ST.
Un error de tipos debe resolverse antes de generar accesos a memoria. No basta
con añadir una palabra a `KEYWORDS` para implementar una función del lenguaje.

### Agregar sintaxis y funciones

1. Define sintaxis, entradas, resultado, errores y límites; escribe un ejemplo mínimo.
2. Añade el token y `KEYWORDS` en el lexer cuando realmente sea una palabra reservada.
3. Extiende el parser y el nodo AST, respetando precedencia y delimitadores.
4. Añade comprobaciones de tipos y firmas en semántica.
5. Traduce el nodo a IR en `lower`; añade runtime y backend solo si hacen falta.
6. Revisa optimización y verificación: todas las rutas deben conservar pila y tipos.
7. Prueba ejecución válida y rechazos con y sin optimización. Actualiza el manual.

Un método no aparece solo por aceptar un punto: necesita resolución del receptor,
firma y una implementación. Las clases son nominales; objetos y colecciones no
son enteros intercambiables. Los parámetros Python sin anotación conservan contrato
entero, incluido el retorno omitido; no proporcionan tipado dinámico de Python.
`len`, `abs`, `sqrt` y pertenencia comprueban tipos antes de generar operaciones.

### Modulos y bibliotecas

`visit` carga módulos por ruta canónica, detecta ciclos y conserva una sola
inicialización. Los alias evitan colisiones de funciones. Las bibliotecas `@nombre`
se buscan en `bibliotecas/nombre/src/lib.st` junto al manifiesto de proyecto más
cercano; no se descargan ni ejecutan instaladores. `scripts/bibliotecas.py` copia
fuentes y verifica el bloqueo SHA-256 de forma separada. El compilador no comprueba
ese bloqueo automáticamente. La sintaxis y los límites están en el manual.

`TypeName::collection` y `ElementType::Nested` describen contenedores tipados con
profundidad acotada; `ElementType::Dynamic` representa `lista[mixto]`. Cada entrada
de lista guarda una etiqueta y una palabra de contenido, también en listas
homogéneas, para que una lista tipada pueda entregarse con seguridad a una función
que acepta `lista[mixto]`. Las referencias se comparten; índices y crecimiento
reutilizan el runtime comprobado. Para añadir un módulo estándar, crea su `.st`, regístralo en
`src/st/stdlib.rs` y declara las primitivas necesarias con sus tipos.

El IDE lee `KEYWORDS` y las firmas públicas desde las fuentes; el navegador renueva
su vocabulario, pero el binario necesita recompilarse para aceptar capacidades
nuevas. La extensión VS Code incluye un vocabulario de reserva. Los comentarios de
las funciones ST alimentan la API integrada mediante `update-integrations-api.py`.

### IDE y servicios auxiliares

`ide/server.py` gestiona archivos, sesiones y ejecución; `app.js` conecta edición,
autocompletado y salida. `folding.js` conserva el documento completo al plegar;
los cambios pasan por su historial. `formats.js` colorea formatos adicionales.
`start_menu.py` valida el menú y `inicio.js` lo presenta. La terminal usa tuberías
para enviar respuestas al programa; el directorio de trabajo es el proyecto.
El editor limita sus rutas, pero no convierte los programas en código aislado.

Las integraciones recorren `src/stdlib/*.st` -> petición JSON -> runtime nativo ->
`runtime/integraciones.py`. Su registro valida capacidades y llama a bibliotecas
externas. Qt vive en `runtime/gui.py`, en otro proceso. Ese código se incrusta al
construir el compilador: editarlo requiere recompilar también los programas.

El sitio estático se genera desde el manual. El foro real usa `website/backend`.
La plantilla `pyforge new --web` combina Django Python con ejecutables ST; el crate
`crates/pyforge_django` es una base separada cuya ABI aún no está conectada al DSL ST.


### Seguridad del compilador

Esta actualización añade defensas concretas. No certifica que ST tenga las mismas garantías que Rust, ni demuestra ausencia total de errores de memoria. El código de ejecución generado sigue incluyendo ensamblador escrito a mano.

#### Comprobaciones implementadas

| Riesgo | Protección |
| --- | --- |
| Desbordamiento i64 en suma, resta, multiplicación o negación | Comprobación de desbordamiento y salida con diagnóstico y estado 1 |
| División/resto por cero, o mínimo i64 dividido entre -1 | Comprobación previa a `idiv`; evita la señal de fallo aritmético |
| Optimización que oculta un desbordamiento | Plegado de constantes mediante operaciones `checked_*`; conserva las operaciones que deben fallar |
| Variable inicializada solo en un bucle o rama | Verificación de inicialización en todos los caminos alcanzables de la IR |
| Función que llega al final sin devolver | Rechazo de compilación si se alcanza el final sin retorno |
| Temporales incompatibles al unir ramas | Análisis de profundidad de pila por flujo de control |
| Desalineación de llamadas por ramas | El generador utiliza profundidades verificadas, no un contador del recorrido textual |
| Corrupción de registros preservados | El generador usa r10, volátil, para operaciones temporales |
| Acceso fuera de rango | Se mantienen las comprobaciones de listas, tuplas y cadenas y de claves inexistentes |
| Reservas demasiado grandes | Comprobaciones de tamaños y presupuesto de 256 MiB de memoria viva reservada por el runtime |
| Recursión o marcos profundos | Comprobación de presupuesto de pila de 512 KiB antes de entrar en cada función generada |

Los fallos del núcleo descritos en la tabla (aritmética entera, índices, memoria y pila) terminan el programa. Las excepciones ST explícitas y los errores de valor/rango de las conversiones numéricas nuevas sí pueden recuperarse mediante `intentar`/`capturar`. Los programas que antes desbordaban enteros silenciosamente o leían valores sin inicializar ahora fallan explícitamente. Esto es un cambio intencionado de comportamiento. La aritmética flotante conserva sus reglas IEEE-754 y puede producir infinito o NaN.

#### Ejemplos

```text
// Termina con «desbordamiento de entero i64», tanto con como sin optimización.
mostrar(9223372036854775807 + 1);
```

```text
// Se rechaza antes de ejecutar: el bucle podría no inicializar la referencia.
mientras falso {
    variable texto = "hola";
}
mostrar(texto);
```

```text
// Se rechaza: existe un camino sin retorno de cadena.
funcion nombre(condicion: booleano) -> cadena {
    si condicion { retornar "Ana"; }
}
```

#### Memoria y recursos

El runtime contabiliza cada mmap redondeado a páginas de 4096 bytes para Linux x86-64. Al crecer una colección, el presupuesto incluye simultáneamente el búfer antiguo y el nuevo; descuenta el antiguo solo después de un munmap correcto. La comprobación ocurre antes de reservar, y también se comprueba el error devuelto por el sistema operativo.

El límite de 256 MiB afecta las reservas del runtime, no toda la memoria del proceso: no incluye el ejecutable, datos estáticos ni la memoria del compilador. Las cadenas y cabeceras siguen viviendo hasta finalizar el programa; no se ha añadido recolección de basura. Alcanzar el presupuesto aborta con un diagnóstico en lugar de seguir creciendo. La contabilidad es para el modelo monohilo actual.

El límite de pila se mide respecto al puntero de pila al iniciar el ejecutable. Cada entrada de función comprueba su marco, el máximo de temporales calculado y un margen de 512 bytes para rutinas auxiliares. Está pensado para el entorno Linux habitual; un límite de pila impuesto por el sistema inferior al presupuesto, fallos externos o cambios futuros de rutinas pueden requerir ajustes. No sustituye las protecciones del sistema operativo.

#### Dónde se implementa

- `src/verify.rs::verify`: recorre el grafo de flujo con una cola, exige pilas iguales al unir ramas e intersecta nombres inicializados. Retorna profundidades y máximo de temporales.
- `src/codegen/llvm.rs::lower_function`: consume profundidades verificadas y emite bloques LLVM solo para instrucciones alcanzables.
- `src/codegen/llvm_compat.rs::emit_fn`: conserva temporalmente el control de pila de funciones con la ABI histórica de excepciones.
- `src/optimizer.rs::fold`: usa operaciones enteras comprobadas al simplificar constantes.
- `src/codegen/collections.s::pf_col_alloc`: controla tamaños, presupuesto y errores de mmap. `pf_col_append` actualiza el presupuesto al liberar el búfer anterior.
- `src/codegen/scalars.s`: diagnósticos aritméticos compartidos con el mecanismo de errores nativos.
- `tests/st.rs`: ejecuta casos de desbordamiento, división por cero, recursión, límite de reservas e inicialización con y sin optimización, y exige estado 1 en vez de una señal.
- `tests/verify.rs`: comprueba directamente IR inválida, pilas incompatibles y nombres sin inicializar.

Los presupuestos están en `_start`/`emit_fn` de `llvm_compat.rs` y `pf_col_alloc` de `collections.s`. Cambiarlos requiere recompilar el compilador y los programas. No aumentes el presupuesto de pila sin revisar el límite del sistema y el margen de las rutinas nativas.

#### Garantías que todavía no se ofrecen

ST no incorpora comprobación de préstamos y tiempos de vida como sistema de tipos, ni liberación automática de todos los objetos. Tampoco cuenta con auditoría formal de su runtime, aislamiento de procesos, ni límites completos para recursos consumidos al compilar fuentes patológicas. Los errores del compilador o del ensamblador pueden afectar al código generado.

Estas pruebas verifican casos concretos; no equivalen a demostrar seguridad total. Una próxima auditoría debería incluir generación aleatoria de programas, pruebas de estrés, seguimiento de ubicaciones de origen y gestión de memoria de larga duración.

#### Consulta y conversión de tipos

`tipo(valor)` consulta un tipo estático y evalúa el argumento una sola vez; no inspecciona memoria mediante conversiones de punteros. `entero`, `flotante` y `booleano` comprueban los tipos de entrada antes del lowering. `src/codegen/conversions.s` valida texto, límites i64 y resultados flotantes antes de retornar, y usa `pf_throw` para errores numéricos recuperables. No se permite cambiar el tipo estático de una variable ya inicializada.

ST separa ahora declaración y reasignación. `variable nombre = valor;` debe
inicializar un nombre nuevo; `nombre = valor;` exige que el nombre ya exista, y
una segunda declaración en el mismo ámbito se rechaza. La IR conserva además la
comprobación de inicialización definitiva al unir ramas. Esto evita que un error
ortográfico cree una variable y evita lecturas de nombres no inicializados.

La frontera de módulos admite `publica funcion`, `privada funcion`,
`publica variable` y `privada variable`. Las funciones antiguas sin modificador
siguen siendo públicas; las variables son privadas salvo exportación explícita.
Las variables importadas solo se leen desde el consumidor. Cada archivo recibe
símbolos internos propios, por lo que dos módulos no comparten un local por usar
el mismo nombre.

La conversión decimal a f64 es aproximada y no garantiza redondeo idéntico a Python. Admite hasta 19 dígitos significativos retenidos y un exponente textual de magnitud máxima 10000; limita el escalado a 400 pasos, rechaza desbordamiento y permite subdesbordamiento a cero. Las conversiones no añaden aislamiento, precisión arbitraria ni garantías de seguridad equivalentes a Rust.

#### Integraciones opcionales

Consulta [permisos](INTEGRACIONES.md), [cifrado](INTEGRACIONES.md) y [Git](README.md).

Se usan bibliotecas existentes para TLS, SSH y cifrado. No se implementan esos
protocolos manualmente. Las APIs deniegan capacidades no declaradas, limitan datos,
redactan errores y no elevan privilegios. Estos controles no prueban seguridad
absoluta ni equivalencia con Rust.

No se han implementado sandbox OS, aislamiento de bibliotecas de documentos,
límites globales de memoria/CPU de todos los hijos ni protección completa contra
carreras de rutas. Procesos permitidos tienen los permisos normales del usuario.
No hay administración VPN, cambios de firewall, DNS o rutas. No se realizó una
auditoría externa ni detección certificada de ausencia de secretos.


### Publicar fuentes en GitHub

#### Qué se protege

`.gitignore` excluye compilaciones, instaladores, entornos Python, perfiles de navegador, configuración local de agentes, papelera del IDE, claves, bases de datos, archivos cifrados y copias de seguridad. Los archivos permanecen en el disco. Las plantillas `.env.example` y `secrets.example.toml` contienen ejemplos ficticios y sí se publican.

Las colecciones `examples/proyectos` y `examples/proyectos_avanzados` tienen listas de fuentes y datos públicos. Sus resultados generados quedan fuera de Git. Si añades fixtures al catálogo, actualiza esas listas:

```bash
python3 scripts/update-example-ignores.py
python3 scripts/check-publication.py
```

El auditor informa únicamente nombres de archivo y tipos de hallazgo, nunca valores privados. Detecta formatos conocidos de credenciales, claves privadas, ejecutables y recursos ajenos al catálogo. No detecta todos los secretos posibles: revisa los datos que decidas publicar, especialmente textos libres, documentos y ejemplos de terceros.

#### Antes de cada commit

El hook `.githooks/pre-commit` revisa todos los archivos del índice, incluso archivos sensibles añadidos con `git add -f`. Para activarlo en una copia nueva:

```bash
git config --local core.hooksPath .githooks
```

Git no activa hooks automáticamente al clonar. No se cambian opciones globales. Para ver la configuración actual:

```bash
git config --local --get core.hooksPath
```

Secuencia de revisión y publicación manual:

```bash
python3 scripts/check-publication.py
git add .
python3 scripts/check-publication.py --staged
git diff --cached --stat
git diff --cached
```

Revisa el contenido antes de crear el commit. El hook es una comprobación local, no una garantía ni una barrera que no se pueda omitir. No se creó ningún commit, remoto o publicación automáticamente durante esta preparación.

#### Lo que sigue siendo público

El enlace de donaciones PayPal de ST es un identificador público necesario para ese botón; no es una contraseña y permanece en el código. La licencia y los ejemplos didácticos permanecen publicados. Los binarios `.deb` y `.exe` se distribuyen por separado, por ejemplo mediante una publicación de versiones, no mediante el repositorio de fuentes.

#### Si alguna vez publicas un secreto

Ignorarlo después no lo elimina del historial: revoca o rota primero la credencial y revisa el historial antes de seguir publicando. El auditor revisa los archivos actuales o el índice, no todo el historial. Al comenzar esta preparación no había commits ni archivos seguidos en este repositorio.

### Verificacion y limites

Desde la raíz del repositorio:

```bash
cargo test
python3 -m unittest discover -s tests -p 'test_*.py' -q
python3 -m unittest discover -s ide -q
python3 scripts/verify-projects.py --compiler target/release/pyforgec
python3 scripts/check-publication.py
```

Construye primero `cargo build --release` para usar el binario de la cuarta orden.
Las pruebas de red utilizan loopback; algunas integraciones necesitan dependencias
opcionales y muestran omisiones si faltan. Qt sin pantalla no prueba apariencia;
las pruebas de documentos no garantizan compatibilidad universal. FTPS, LibreOffice
DOC e instalación real en Windows requieren validación adicional. No uses conteos
de una ejecución antigua como evidencia de la versión actual.

Las pruebas nativas comprueban salida y fallos con ambas optimizaciones. Conserva
casos de desbordamiento, índices inválidos, pila, tipos, excepciones, JSON, módulos
y bibliotecas. Para cada extensión del lenguaje prueba también entradas inválidas.
La construcción Debian ejecuta regresiones antes de publicar el paquete. Un cambio
de mensaje en un ejemplo puede hacer fallar su salida esperada: revisa el contrato,
no desactives la prueba para conseguir un instalador.

Quedan pendientes tipos dinámicos generales, registro remoto de paquetes, gestión
de memoria de larga duración, protocolos y GUI completos, integración del DSL
Django y el backend de renderizado 3D por GPU. El núcleo 3D CPU sigue siendo
experimental; el manual distingue
las capacidades disponibles de las propuestas.
