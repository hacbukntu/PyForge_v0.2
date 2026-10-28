# Historial de cambios

## Sin publicar

- Las listas ST son ordenadas y mutables y ahora pueden inferir o declarar
  `lista[mixto]`; cada posición conserva una etiqueta nativa para cadenas,
  enteros, flotantes, booleanos, nulo, colecciones y objetos.
- Acceso, cambio, `agregar`, pertenencia, `mostrar`, `tipo()` y conversiones
  comprobadas funcionan con elementos mixtos sin reinterpretar memoria.
- El curso del IDE oculta capítulos internos del proyecto y conserva lecciones y
  ejemplos de uso. El editor añade búsqueda/reemplazo y comprobación sin ejecutar.

- ST separa declaraciones y reasignaciones: toda variable nace inicializada,
  las redeclaraciones y asignaciones a nombres inexistentes se rechazan y el
  tipo inferido permanece estable.
- Los módulos admiten funciones y variables `publica`/`privada`; las variables
  exportadas son de solo lectura para el consumidor y los símbolos internos de
  cada archivo se aíslan.
- `pyforgec language-json` expone el vocabulario real a los editores. Vic y
  PyForge IDE incorporan colores, autocompletado y fragmentos para visibilidad y
  conversiones explícitas `entero`/`flotante`.

- LLVM 21 mediante Inkwell 0.9 es ahora obligatorio y el único backend público;
  se retiraron `--backend`, el enum de selección y la construcción sin LLVM.
- Lowering de funciones, control de flujo, llamadas, escalares y colecciones a
  LLVM IR verificable; optimización `default<O2>` y comando `emit-llvm`.
- Compatibilidad completa del catálogo ST en ambos modos de optimización. Las
  excepciones conservan temporalmente su ABI probada como ensamblador inline LLVM.
- El antiguo archivo del generador pasó a `llvm_compat.rs`, privado para runtime y
  excepciones; el empaquetador prueba el único recorrido y declara `libllvm21`.

- `pyforge-editor3d` incorpora menús e iconos, jerarquía buscable, inspector de
  posición/rotación/escala/recursos, paneles redimensionables, consola, atajos,
  creación, duplicación, eliminación y enfoque de entidades.
- Nuevo módulo estándar `editor3d`: una escena se crea en `.st`, se compila en
  segundo plano con límites de tiempo/salida, se aplica al viewport y puede
  exportarse de los gizmos de vuelta a código ST. Incluye ejemplo, validador sin
  ventana, pruebas con/sin optimización e icono oficial incrustado.

- Viewport 3D sólido integrado en el editor con cámara orbital, caras sombreadas,
  materiales por color, selección y gizmos comprobados.
- Salida física opcional de audio espacial mediante `pw-cat` (PipeWire) o `aplay`
  (ALSA), con selección automática y reproducción fuera del hilo del juego.
- `pyforge-game3d` y Portal Runner visual: reglas y progresión en ST, ventana nativa
  y protocolo de transición validado. El `.deb` incluye ahora los tres binarios 3D.
- Cuatro juegos terminados de cuadrícula 3D: Portal Runner, Arena de Cristales,
  Laberinto Neón y Guardianes del Núcleo, con victoria/derrota y pruebas de lógica.
- Ejemplos compilables para inspeccionar OBJ/glTF/PNG/JPEG y reproducir WAV; manual,
  curso y documentación de instalación sincronizados con el alcance comprobado.

- Backend opcional `pyforge_engine3d`: ventana winit, Vulkan/OpenGL mediante wgpu,
  pipeline WGSL, textura GPU, OBJ/glTF, pose esquelética, audio espacial matemático
  y editor visual con jerarquía, inspector y deshacer/rehacer.
- El editor 3D abre y guarda escenas versionadas mediante reemplazo atómico; el
  renderer recupera superficies perdidas/obsoletas y diferencia timeout de falta
  de memoria GPU.
- El pipeline GPU añade profundidad, tres instancias visibles y sombreado PBR
  básico validado por Vulkan; el shader evita indexación dinámica no portable.

- Primer núcleo propio `motor3d`: vectores, cuaterniones, matrices 4×4,
  transformaciones, cámara, mallas indexadas, materiales, luces, AABB, animación,
  cuerpo rígido, escenas, ciclo determinista y gestor de recursos con presupuesto.
  El backend GPU continúa separado.

- Módulo nativo `teclado` para juegos de terminal: lectura inmediata, espera
  no bloqueante, flechas normalizadas y restauración segura de `termios`.

- `pyforge new nombre --web` y `pyforge web`: plantillas ST + Django, HTML/CSS/JS, cuentas, foro con capturas y notas SQL por propietario.
- IDE: bóvedas cifradas opcionales, sesiones con contraseña, bloqueo y temas Normal/Hacker/Pro.
- Entrada al foro en la web y guías de despliegue y privacidad con límites explícitos.

- Selector nativo de carpetas para abrir y crear proyectos desde el menú, con cancelación segura.
- Preparación de Git: exclusión de perfiles, secretos y salidas; catálogos de fixtures, auditor y hook previo al commit.

- Manual local estilizado con índice y colores; Inicio y Ctrl+Alt+H desde la terminal.
- Comentarios y guía de arquitectura, personalización y ampliación del módulo gráfico Qt.

- IDE: barra simplificada, Ctrl+B, autocierre permanente y página local de apoyo. Colores JSON/TOML y vista SQLite de solo lectura.
- Menú inicial configurable mediante JSON y script, con apertura y creación de proyectos y acceso al manual.

- 20 aplicaciones y juegos adicionales en `examples/proyectos_avanzados`, con fuentes comentadas, manifiestos, datos reproducibles y verificador de laboratorios locales.
- ZIP independiente para la web y selección explícita de recursos en instaladores, excluyendo salidas y secretos de ejecución.

- Colección de 80 proyectos ST independientes con juegos, herramientas cotidianas, algoritmos, clases y generadores de documentos.
- Verificador con copias temporales, comparación de resultados y dos modos de optimización; recursos completos incluidos en instaladores y ZIP de la web.

- Consulta ST `tipo(valor)` completada y corregida para devolver texto sin salto de línea; conserva funciones de usuario y json.tipo.
- Conversiones explícitas entero/flotante/booleano, errores numéricos capturables e inferencia de variables verificada.
- Botón Archivos/Ctrl+B y redimensionado de terminal desde su borde superior, con acceso por teclado.

- Plegado visual de bloques ST y bloques Python por indentación, con flechas, números originales y opción Desplegar todo.
- Documento completo independiente de la vista plegada para conservar código en edición, historial, guardado y ejecución.

- Módulos estándar incrustados time, os, math, io y json, con alias y selección de funciones sin depender de Python.
- Primitivas Linux nativas, JSON acotado con escapes Unicode y errores ST, y firmas estándar en el autocompletado del IDE.
- Ejemplos, pruebas nativas y guía de implementación/extensión en el manual y explicacion.md.

- Comentarios ST multilínea `/* ... */` y `""" ... """`, con diagnóstico de apertura si falta el cierre.
- Coloreado y plantillas de comentarios en el IDE, conservando sangría y colocando el cursor dentro del bloque.


- Web estática en español con descargas verificadas, curso generado desde el manual, buscador, progreso local, vocabulario del lexer y ejemplos descargables.
- Generador y vigilancia de la web sin Node ni cambios en el compilador.


- Generador de paquetes `.deb` y `.exe` para Windows con WSL, PATH por plataforma y recursos del IDE sincronizados.
- Vigilancia de fuentes con instantáneas y pruebas antes de publicar nuevos instaladores.


- Clases y estructuras nominales nativas: campos tipados, métodos con `yo`, `nuevo` y bloques `implementar`.
- Excepciones explícitas `lanzar`, `intentar`, `capturar` y `finalmente`, con limpieza de manejadores en retornos y bucles.
- Verificación de la pila de manejadores antes de generar ensamblador.
- `entrada(mensaje)` imprime una cadena sin salto de línea; se conserva `entrada()`.
- Ejemplos y explicación de arquitectura y límites en las secciones 20–22 de `docs/MANUAL_ST.md`.


- Añadidas importaciones ST por ruta relativa, con deduplicación y detección de ciclos.
- Añadidas listas de enteros y diccionarios de claves literales/valores enteros, con crecimiento y errores de acceso controlados.
- Sincronizados colores, sugerencias y plantillas del IDE con KEYWORDS de Rust.

- Añadida la entrada `.st` con léxico y gramática independientes, palabras en español y bloques entre llaves.
- Conservada la entrada Python y compartidas las etapas de AST, tipos, IR y generación nativa.
- Añadidos ejemplos ST, pruebas de integración y una guía para personalizar el lenguaje.

## 0.2.0

- Sustituida la arquitectura de traducción de Python a Rust por etapas propias de compilación.
- Añadidos el AST, el análisis semántico y de tipos, la IR de PyForge y el optimizador.
- Añadido el generador de ensamblador Linux x86-64.
- Añadidas las rutinas nativas de ejecución basadas en llamadas al sistema.
- Añadidas las interfaces de línea de comandos `pyforgec` y `pyforge`, esta última similar a Cargo.
- Añadida la generación del archivo de configuración del proyecto.
- Añadidos ejemplos, pruebas básicas y documentación de arquitectura.

## Ampliación nativa de tipos ST

- Tuplas inmutables de enteros, conjuntos de enteros únicos y valor nulo tipado.
- Operador de identidad `es` y método `.agregar(...)` para listas y conjuntos.
- Ejemplo `examples/tipos.st`, comentarios en español y regresiones nativas con/sin optimización.
- Las capacidades restantes del plan (flotantes, cadenas generales, objetos, excepciones e IO) siguen pendientes.

## Cadenas y flotantes ST

- `cadena` en variables, parámetros y retornos; concatenación, igualdad y longitud Unicode.
- `flotante` f64 con literales decimales/exponentes, aritmética, comparaciones e impresión nativa.
- `mostrar` acepta los cuatro tipos escalares; las salidas enteras y booleanas mantienen compatibilidad.
- Runtime comentado en `src/codegen/scalars.s`; guía `docs/MANUAL_ST.md` y ejemplo `examples/cadenas.st`.
- El formato de flotantes usa hasta seis decimales y las concatenaciones conservan memoria hasta finalizar.

## Operaciones de texto y claves dinámicas

- Índices Unicode, `buscar`, `reemplazar`, `cadena` e `interpolar` con marcadores `{}`.
- Concatenación con escalares y claves de diccionario evaluadas en ejecución.
- Runtime nativo comentado, ejemplos y pruebas con y sin optimización.
- Pendiente: elementos de colecciones distintos de enteros.

## Comprobaciones de seguridad nativa

- Aritmética i64 comprobada con igual comportamiento con/sin optimización.
- Verificador de flujo de IR: inicialización, retornos y profundidad de pila.
- Presupuestos de 256 MiB de reservas del runtime y 512 KiB de pila generada.
- Los programas con desbordamientos o lecturas sin inicializar ahora fallan explícitamente. Véase `docs/README.md`.

## Integraciones opcionales — 13 de septiembre de 2026

- Puente local entre ejecutables ST nativos y auxiliar Python incrustado; no cambia las APIs anteriores.
- Dependencias entre módulos estándar, nuevas firmas ST y alias leídos automáticamente por el IDE.
- Archivos, SQLite parametrizado y transacciones, documentos simples, TCP/UDP/TLS, adaptadores SSH/SFTP/FTPS, AES-GCM/PBKDF2/HMAC, procesos acotados sin shell.
- GUI declarativa opcional en proceso Qt separado; detección e inspección de VPN, sin administración de red.
- Autocierre de delimitadores configurable, protección del texto plegado, visor de permisos y enlace de donación.
- 12 proyectos nuevos, guías en español, copia previa y pruebas de integración y compatibilidad.
- Cobertura parcial explícita: consultar docs/INTEGRACIONES.md antes de usar capacidades avanzadas.

## Iconos del sistema — 14 de septiembre de 2026

- `website/logo.svg` es la fuente única de iconos MIME/SVG/PNG para Linux e ICO multirresolución para Windows.
- Registro `.st` por paquete Linux y por usuario Windows, conservando asociaciones existentes y retirando solo entradas propias.
- Abrir archivos desde el escritorio selecciona la fuente en ST IDE; no ejecuta su código.
- El empaquetador detecta cambios del logo, reconstruye iconos y actualiza cachés en instalación/desinstalación.

## Comunidad oficial y base web Rust

- Foro, registro y acceso propios en `website/backend`, excluidos del ZIP estático.
- `new --web` ofrece notas, inventario y agenda; tres proyectos en `examples/web`.
- Puente anterior conservado en `runtime/web_legacy`; proyectos existentes intactos.
- Parámetros y argumentos ST admiten `;`, manteniendo compatibilidad con comas.
- Crate independiente `pyforge_django`: axum/Tokio, sqlx/SQLite, Tera, Argon2id y ABI C.
  Todavía pendiente el enlace AOT de `importar "django"`, callbacks ST y sintaxis `modelo`.
- Guía de arquitectura, contratos de memoria, límites y ampliación en `docs/INTEGRACIONES.md`.

## Correcciones y motor 2D verificable

- Intrínsecos nativos con validación de tipos, pertenencia corregida y conversiones numéricas coherentes.
- Parámetros Python sin anotación restringidos a entero; firmas explícitas str/float y literales flotantes admitidos. El soporte continúa siendo un subconjunto.
- Capturas web aisladas por sesión/propietario mediante migración adicional, sin borrar datos.
- Culebrita con reglas ST, estado de sesión Django, CSRF y JavaScript externo compatible con CSP.
- Ejemplo corto del motor y proyecto completo en examples/web/culebrita.
- Curso con enlaces de cuenta; formulario del foro conserva el destino de regreso.
- Extensión VS Code con vocabulario incluido, tareas sin shell y sin anunciar depuración inexistente.
- No se añade motor 3D ni compatibilidad con Minecraft/Call of Duty.

- Colecciones anidadas homogéneas con anotaciones, inferencia, acceso y modificación encadenados; `tipo()` describe cada nivel.
- Bibliotecas locales `@nombre` y herramienta de copia/verificación SHA-256; no requiere un registro remoto.
- Corregidas las salidas de entrada de dos proyectos que impedían generar el `.deb`; se mantienen todas las verificaciones.

- Primer runtime Rust nativo experimental (ABI 1, Linux x86-64): módulo `nativo`, buffers binarios y archivos con cierre explícito, límites y errores recuperables.
- Enlace condicional de biblioteca estática incrustada: los programas que usan únicamente APIs nativas no necesitan Python. Las integraciones Python se conservan y se prueba su convivencia con `nativo`.
- Ejemplos `nativo_buffers.st` y `nativo_archivos.st`; API, permisos y limitaciones documentados en el manual. No añade aún gestión automática de memoria, concurrencia, red o motor 3D.
- Recursos nativos tipados `archivo_nativo` y `buffer_nativo`, con propiedad exclusiva mediante `usar`, préstamos en llamadas y cierre generado para salida normal, excepción, retorno y control de bucles.
- El análisis rechaza copiar, convertir, almacenar o devolver préstamos; la API manual `nativo` y todas las integraciones Python continúan disponibles.
