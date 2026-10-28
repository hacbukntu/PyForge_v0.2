# Integraciones opcionales de PyForge / ST

Estado: experimental. **Esta entrega no completa el
100 % de todas las capacidades posibles.** A continuación se distingue lo que funciona,
lo que necesita dependencias y lo que todavía no está implementado.

## Qué cambia y qué se conserva

El código ST sigue pasando por lexer, parser, análisis de tipos, IR y generación
ELF Linux x86-64. Los módulos `time`, `math`, `os`, `io` y `json` conservan sus
operaciones nativas. Los nuevos módulos llaman a un auxiliar **Python 3.11 o
posterior**, por `/usr/bin/python3` de forma predeterminada. Windows continúa utilizando **WSL**.
Los programas que usan estas nuevas APIs ya no son independientes de Python.
No se ha cambiado el lenguaje ST por un intérprete Python.

La fuente del auxiliar se incrusta al recompilar el compilador. No busca código
Python en el proyecto, no utiliza shell ni instala paquetes automáticamente.
Editar el runtime exige reconstruir el compilador y volver a compilar el programa.
El IDE obtiene las nuevas firmas desde los archivos ST y los alias del registro Rust.

## Comenzar después de instalar la versión actualizada

Copia uno de los [proyectos de ejemplo](../examples/integraciones/README.md) a una
carpeta de trabajo. Desde esa carpeta, ejecuta `pyforge run` o abre `pyforge-ide`.
La carpeta actual determina el manifiesto y la raíz de archivos de las integraciones.

Este programa crea un archivo nuevo:

```st
importar "archivos";
archivos.escribir_texto("nota.txt", "Hola desde ST", falso, "utf-8");
mostrar(archivos.leer("nota.txt", "utf-8"));
```

Su `PyForge.toml` necesita:

```toml
[package]
name = "mi_aplicacion"
version = "0.1.0"
[build]
entry = "src/main.st"
[permisos]
leer_archivos = true
escribir_archivos = true
```

Los permisos no declarados se deniegan en el auxiliar. El compilador todavía no
hace inferencia estática de permisos ni solicita aprobaciones interactivas.
El argumento `verdadero` de eliminar/sobrescribir es una confirmación del programa;
**no es una autorización interactiva del dueño del equipo**.

## Capacidad real

| Área | Implementado | Pendiente o limitado |
|---|---|---|
| Archivos | Texto con codificación, copia, movimiento, carpetas, listado, eliminación confirmada | No borrado recursivo; no sandbox del sistema operativo |
| SQLite | Conexiones, sentencias parametrizadas, consultas, transacciones, cierre, copia coherente | No migraciones automáticas ni restauración integrada |
| TXT/RTF | Crear ambos; leer y sustituir texto TXT | RTF existente requiere conversión externa; no editor RTF completo |
| DOCX | Crear párrafos, extraer párrafos y sustituir dentro de runs mediante python-docx | No API de imágenes, tablas, encabezados, estilos o sustitución entre runs |
| DOC | Conversión con LibreOffice instalado | No edición binaria propia; necesita conversión explícita |
| PDF | Crear texto sencillo, extraer texto, unir y extraer páginas | No OCR ni edición arbitraria de texto; fuente básica de ReportLab no cubre todo Unicode |
| TCP/UDP | Cliente/servidor IPv4, timeout, bytes base64, cierre | Sin cancelación asíncrona ni reintentos; UDP sin fiabilidad ni DTLS |
| TLS | Cliente con CA y nombre verificados, mínimo TLS 1.2 | No servidor TLS, mTLS ni selección de certificado cliente |
| SSH/SFTP | Contraseña desde entorno autorizado, known_hosts obligatorio, listar/subir/descargar | Sin autenticación por clave privada ni ejecución de órdenes SSH; autenticación y SFTP probados en loopback |
| FTP | FTPS explícito, canal de datos cifrado, listar y transferir | No FTP sin TLS; STOU asigna nombre único; sin mkdir/borrado remoto |
| Cifrado | AES-256-GCM, claves aleatorias, SHA256/512, HMAC, PBKDF2, comparación, base64 | Sin cifrado de archivos por bloques, Argon2id, gestor/rotación de claves ni borrado garantizado de secretos |
| Procesos | Programa y argumentos separados, lista permitida, timeout, salida acotada, entorno filtrado | Sin API de shell, administración, stdin, segundo plano o sandbox de procesos |
| GUI | Adaptador declarativo Qt, etiquetas, botones, entradas, áreas, casillas, opciones, listas, tablas y paneles; eventos | Pruebas Qt sin pantalla superadas; faltan árboles, menús, pestañas, diálogos, imágenes y diseñador |
| VPN | Detectar herramientas/interfaces e inspección conservadora de perfiles sin secretos | No conectar/desconectar, WireGuard/OpenVPN/IPsec operativo, kill switch, cambios de DNS/rutas/firewall ni pruebas de fugas |
| Permisos | Runtime deniega por defecto, valida manifiesto; IDE muestra permisos | No aislamiento completo, perfiles seguro/desarrollo/producción/pruebas ni elevación |
| Plataformas | Núcleo nativo Linux x86-64; instalación Windows con WSL | No ejecutables ST PE nativos ni pruebas en Windows real |

## API y módulos

Consulta las [firmas públicas y comentarios](INTEGRACIONES.md). Los módulos son
`integraciones`, `archivos`, `db`, `documentos`, `red`, `cifrado`, `comandos`,
`sistema`, `vpn` y `gui`. Existen alias `sistema.archivos`, `sistema.comandos`,
`red.tcp`, `red.udp`, `red.socket`, `red.ssl`, `red.ssh`, `red.ftp`, `red.sftp`,
`red.vpn` y `red.vpn.diagnostico`.

Usa un alias simple para nombres con puntos:

```st
importar "red.tcp" como tcp;
// tcp.tcp_conectar(host, puerto) utiliza las mismas firmas que red.
```

Las colecciones del lenguaje siguen siendo homogéneas. Argumentos/resultados
compuestos viajan como **texto JSON**, no como objetos dinámicos nuevos.
Las cadenas de respuesta del canal son de hasta 4095 bytes; el módulo JSON nativo
conserva sus límites anteriores. Los archivos manejados por copia/transferencia se
limitan a 8 MiB. Hay hasta 64 recursos abiertos por sesión. Se deben cerrar en
`finalmente`. El cierre de SQLite revierte cambios no confirmados.

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

## Dependencias y comandos

En Linux/WSL, comprueba primero:

```bash
/usr/bin/python3 --version
/usr/bin/python3 -c "import sqlite3, ssl; print(sqlite3.sqlite_version, ssl.OPENSSL_VERSION)"
```

Las siguientes órdenes son **para que las ejecute el administrador si decide
instalar esas capacidades**, no se ejecutan automáticamente:

```bash
sudo apt install python3-cryptography python3-docx python3-pypdf python3-reportlab python3-paramiko libreoffice
```

La disponibilidad y versión de paquetes dependen de la distribución. Para Qt,
consulta los paquetes PySide6/QtWidgets de tu distribución. El auxiliar usa
`/usr/bin/python3 -I`: un paquete instalado solamente en otro entorno virtual no
se vuelve disponible automáticamente. No uses `sudo pip` ni desactives PEP 668.

En Windows PowerShell o CMD, ejecuta las órdenes dentro de WSL:

```powershell
wsl /usr/bin/python3 --version
wsl /usr/bin/python3 -c "import sqlite3, ssl; print(sqlite3.sqlite_version)"
wsl
```

Ya dentro de WSL, cambia a la carpeta Linux de tu proyecto y ejecuta `pyforge run`.
Instalar una biblioteca con `py -m pip` en Windows **no** la instala en WSL.
`integraciones.informacion()` muestra versiones detectadas o `null`. La ausencia
produce `DEPENDENCIA_AUSENTE` al invocar esa función, no al importar el módulo.
No se certifica compatibilidad con todas las versiones de bibliotecas externas.

## Entorno virtual propio

También puedes instalar las dependencias en un entorno propio, sin modificar el
Python del sistema. En Linux/WSL, después de autorizar las descargas:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install python-docx pypdf reportlab cryptography paramiko PySide6
PYFORGE_PYTHON="$PWD/.venv/bin/python" pyforge run
```

`PYFORGE_PYTHON` se lee **al compilar** y exige una ruta absoluta. Esa ruta queda
dentro del ejecutable; el entorno debe seguir existiendo al ejecutarlo. Para
mover el programa a otro equipo, recompílalo con la ruta/dependencias de destino.
La variable no cambia ejecutables ya compilados. No se usa `eval` para esa ruta.
Las dependencias temporales de las pruebas no se instalan ni empaquetan como parte
del Python del usuario.

## Errores y seguridad

Los errores del auxiliar se convierten en excepciones ST con código y sugerencia.
Los detalles de excepciones externas se redactan para no mostrar contraseñas,
SQL, documentos o salidas sensibles. No captures un error para afirmar éxito.

```st
importar "archivos";
intentar { mostrar(archivos.leer("inexistente.txt", "utf-8")); }
capturar error { mostrar(error); }
```

Estas APIs no hacen a ST «tan seguro como Rust». Las comprobaciones de rutas
rechazan padres, rutas absolutas y enlaces, pero no resuelven todas las carreras
con modificaciones concurrentes del sistema de archivos. Procesos permitidos y
bibliotecas de documentos pueden acceder al sistema con los permisos del usuario.
No abras documentos hostiles sin aislamiento externo. Las funciones nativas
anteriores mantienen su comportamiento: los nuevos permisos no las restringen.

Los errores del transporte (auxiliar ausente, canal cerrado, espera agotada) también
son capturables. No reintentes automáticamente operaciones con efectos: un fallo de
respuesta no demuestra que la escritura o transacción no se haya ejecutado.

## Cómo extenderlo

1. Agrega una función comentada a `runtime/integraciones.py`. Valida argumentos,
   capacidades y límites antes del efecto. No uses `eval`, `exec` ni shell.
2. Registra su nombre exacto en `Runtime.operations`.
3. Expón una función tipada en `src/stdlib/<modulo>.st`. Usa `integraciones.pedir`,
   codifica cadenas con `json.codificar` y decodifica el tipo de respuesta.
4. Si es un módulo nuevo, añade `include_str!` en `src/st/stdlib.rs`.
5. Agrega pruebas del auxiliar y una compilación/ejecución ST real. Las bibliotecas
   ausentes deben marcarse como no verificadas, nunca sustituirse por éxitos falsos.
6. Ejecuta `python3 scripts/update-integrations-api.py`; actualiza manual y ejemplos.
7. Recompila el compilador, prueba, reconstruye paquetes y sitio web.

`src/codegen/integraciones.s` hace socketpair, fork, execve y transporte. El auxiliar
se inicia bajo demanda. `src/st/modules.rs` carga dependencias estándar en orden
antes de resolver nombres; esta ampliación permite a los nuevos módulos reutilizar
`json` sin duplicarlo. El backup y mapa previo están en
[README.md](README.md).

## Validación reproducible

Desde el repositorio, para desarrollo:

```bash
cargo build --release
cargo test
/usr/bin/python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 -m unittest discover -s ide -v
python3 scripts/verify-projects.py --compiler target/release/pyforgec
```

Las pruebas de red usan solamente loopback y certificados temporales. Un entorno
que prohíbe sockets debe autorizar esa prueba; un bloqueo no equivale a prueba
correcta. DOCX/PDF, Qt y SSH/SFTP se probaron en un entorno temporal con dependencias;
LibreOffice y FTPS todavía requieren validación adicional. Consulta el [informe de pruebas](README.md).

## Contratos de los modulos

### Archivos

`archivos` y el alias `sistema.archivos` operan dentro de la carpeta donde se ejecuta
el programa. Admiten lectura, escritura con codificación, crear carpetas, listar,
copiar, mover y eliminar archivos o carpetas vacías.

Se rechazan rutas absolutas, `..`, enlaces simbólicos, la propia raíz y nombres
protegidos. Una copia no reemplaza destinos. `eliminar(ruta, verdadero)` exige
capacidad de eliminación y confirmación explícita del programa. No hay borrado
recursivo. Los nombres y rutas siguen siendo datos, no instrucciones de shell.

El límite de archivos es 8 MiB en el auxiliar; el canal JSON es menor y limita lo
que puede devolverse de una vez. No es una API de streaming. Las rutas pueden
sufrir carreras si otro proceso modifica directorios concurrentemente: no es una
jaula de sistema de archivos. Usa aislamiento externo para código no confiable.

Ejemplo: [archivos](../examples/integraciones/archivos/src/main.st).
El módulo nativo `os` anterior mantiene su semántica y no hereda estos permisos.

### SQLite

Importa `db`. `conectar(ruta)` devuelve un entero de sesión. `ejecutar` y `consultar`
reciben SQL y un arreglo JSON separado de parámetros. No construyas SQL concatenando
valores del usuario. Los marcadores `?` se enlazan mediante sqlite3.

`confirmar` hace commit; `revertir` hace rollback; `cerrar` revierte transacciones
pendientes. `respaldar` usa la API de copia SQLite y exige confirmar/revertir antes;
no reemplaza un destino existente. No hay API de restauración: copia un respaldo
verificado a un nombre nuevo con las conexiones cerradas.

Las consultas devuelven hasta 100 filas JSON. Usa LIMIT/OFFSET. Se limitan la longitud
de SQL, los valores y el tiempo de consulta. ATTACH/DETACH, PRAGMA y funciones de
carga/escritura externa se rechazan para impedir escapes de ruta desde SQL.
CREATE TABLE/INDEX y las operaciones CRUD siguen disponibles.

Ejemplo completo: [agenda](../examples/integraciones/sqlite/src/main.st).
Permisos: `base_datos`, `leer_archivos`, `escribir_archivos`. Son datos locales:
no incluyas bases reales en Git.

### Documentos

Importa `documentos` y declara `documentos`, `leer_archivos` y `escribir_archivos`
según las operaciones. `crear(ruta, texto, sobrescribir)` selecciona formato por
extensión. `falso` exige un destino nuevo. `leer` extrae texto; `sustituir` modifica
TXT o texto contenido dentro de un run DOCX. La escritura utiliza publicación
atómica del archivo; no autoriza sobrescribir implícitamente.

- TXT: UTF-8 directo; otras codificaciones están en `archivos`.
- RTF: generación de texto con escapes Unicode. Lectura/edición compleja mediante conversión externa.
- DOCX: python-docx crea párrafos y conserva el documento al editar runs; no sustituye textos divididos entre runs ni recorre todas las tablas/encabezados.
- DOC: `convertir` utiliza LibreOffice headless con perfil temporal separado.
- PDF: ReportLab crea páginas de texto sencillas; pypdf extrae texto, une documentos y extrae páginas desde índice cero. No realiza OCR ni reconstruye diseño.
- DB: SQLite se gestiona con el módulo `db`, no con un editor de texto.

Ejemplos: [TXT/RTF](../examples/integraciones/documentos/src/main.st),
[Word/PDF](../examples/integraciones/word_pdf/src/main.st),
[conversión DOC](../examples/integraciones/convertir_doc/src/main.st),
[páginas PDF](../examples/integraciones/pdf_paginas/src/main.st).

No hay edición universal sin pérdidas. No se ha implementado la API solicitada de
imágenes, estilos, tablas ni posicionamiento arbitrario. DOCX/PDF se verificaron con bibliotecas reales en un entorno temporal y desde ST
compilado. La conversión DOC mediante LibreOffice aún no se ha validado.
Documentos hostiles requieren aislamiento externo; los límites de tamaño comprimido
no son un límite garantizado de CPU/memoria de bibliotecas descompresoras.

### Permisos

Copia y revisa las tablas de [config.example.toml](../config.example.toml) dentro del
`PyForge.toml` de tu proyecto. Se carga desde el directorio actual al iniciar el
auxiliar. Valores no booleanos y nombres desconocidos se rechazan. No se busca
otro manifiesto ascendiendo directorios y no se carga `.env` automáticamente.

Capacidades: `leer_archivos`, `escribir_archivos`, `eliminar_archivos`, `base_datos`,
`documentos`, `red`, `escuchar_red`, `cifrado`, `ejecutar_procesos`, `leer_entorno`,
`consultar_red`, `gui`. Todas se deniegan si no aparecen como `true`.

`[integraciones]` contiene listas `hosts`, `programas` y `entorno`. Host y nombre
son exactos; los programas deben ser rutas absolutas. Autorizar un ejecutable
potente permite lo que ese ejecutable pueda hacer: no basta para crear un sandbox.
No añadas shells o intérpretes para eludir las restricciones de una API.

El botón **Permisos** del IDE informa de la configuración; no concede capacidades.
Para revocar, cambia el valor a false y reinicia el programa. El auxiliar conserva
la configuración de su sesión. No existe elevación automática de privilegios.

Los permisos solo cubren estas integraciones: no restringen `abrir`, `escribir` ni
los módulos nativos anteriores. El análisis estático de permisos, permisos vinculados
criptográficamente al ejecutable, aprobación interactiva y aislamiento OS están
pendientes. No ejecutes programas no confiables basándote solo en este manifiesto.

### TCP y UDP

Importa `red` o `"red.tcp" como tcp`. Autoriza `red = true` para conexiones y
`escuchar_red = true` para servidores. Declara hosts exactos:

```toml
[permisos]
red = true
escuchar_red = true
[integraciones]
hosts = ["127.0.0.1", "localhost"]
```

`tcp_conectar(host, puerto)` devuelve un entero identificador.
`tcp_escuchar` y `tcp_aceptar` crean servidor y conexión aceptada.
`enviar` envía UTF-8; `enviar_bytes` recibe base64. `recibir` devuelve **base64**
para no romper bytes o caracteres divididos entre lecturas. TCP no conserva
fronteras de mensajes: tu protocolo debe delimitar y reunir los fragmentos.
`direccion` consulta el extremo local. `cerrar` libera el recurso.

`udp_abrir`, `udp_enviar` y `udp_recibir` gestionan datagramas IPv4. La recepción
incluye host, puerto y `datos_base64`. Datagramas mayores de 1024 bytes se rechazan.
UDP no garantiza entrega, orden ni confidencialidad. No hay DTLS.
Cada operación bloqueante tiene un timeout de cinco segundos; no hay reintentos
ni cancelación asíncrona. La lista de hosts no es un firewall ni bloquea cambios DNS.

Las pruebas `test_tcp_loopback_and_closed_handle`, `test_udp_receive` y
`test_tls_certificate_and_hostname` son ejemplos ejecutables de laboratorio local.
Para ST, consulta también las [firmas](INTEGRACIONES.md).

### TLS

`red.tls_conectar(host, puerto, ca)` utiliza el contexto de cliente de la biblioteca
`ssl` de Python, con verificación de CA y nombre del servidor, y TLS mínimo 1.2.
Una cadena vacía en `ca` selecciona el almacén del sistema. Una CA explícita debe
estar dentro del proyecto y requiere `leer_archivos`.

No existe opción pública para desactivar la verificación. No confíes en un
certificado solo para evitar un error. La prueba local crea un certificado efímero,
verifica el caso confiable y rechaza CA desconocida y nombre incorrecto.

No están implementados servidor TLS, certificado cliente/mTLS, DTLS ni APIs para
cifrados obsoletos. TCP sin `tls_conectar` no queda cifrado automáticamente.
Para límites y permisos consulta [TCP/UDP](INTEGRACIONES.md).

### SSH

`red.ssh_conectar(host, puerto, usuario, hosts_conocidos, variable_clave)` necesita
Paramiko, permiso `red`, lectura del archivo `known_hosts` y permiso `leer_entorno`.
Los hosts y nombres de variables deben estar en las listas autorizadas.

Se utiliza `RejectPolicy`: una clave desconocida o distinta se rechaza. Obtén y
verifica la huella por un canal confiable antes de preparar `known_hosts`.
Nunca aceptes automáticamente una clave obtenida de una conexión no verificada.
El comportamiento se apoya en la [API oficial SSHClient](https://docs.paramiko.org/en/latest/api/client.html).

```toml
[permisos]
red = true
leer_archivos = true
leer_entorno = true
[integraciones]
hosts = ["servidor.ejemplo.com"]
entorno = ["SSH_PASSWORD"]
```

En Linux/WSL puedes leer una contraseña local sin incluirla en el historial:

```bash
read -rs SSH_PASSWORD
export SSH_PASSWORD
pyforge run
unset SSH_PASSWORD
```

No guardes esa variable en el repositorio ni imprimas su valor. En Windows ejecuta
el ejemplo desde WSL y define allí la variable. No se heredan automáticamente
variables de PowerShell hacia una sesión WSL.

La autenticación por clave privada y las órdenes remotas no están implementadas.
Se verificaron autenticación y transferencias SFTP con Paramiko en un servidor
de laboratorio loopback y archivos ficticios. Cada despliegue real necesita su
propia configuración y verificación de huellas. Cierra en `finalmente`.

### FTPS y SFTP

SFTP reutiliza el identificador SSH: `sftp_listar`, `sftp_subir`, `sftp_descargar`.
La subida abre el destino remoto en modo exclusivo; la descarga no sobrescribe
el destino local. Las transferencias se limitan a 8 MiB.

`ftps_conectar` utiliza FTP explícito sobre TLS, verifica el certificado y activa
la protección del canal de datos. `ftp_listar`, `ftp_subir` y `ftp_descargar`
requieren ese recurso FTPS. No se ofrece FTP sin cifrar. Los nombres remotos
rechazan CR, LF y NUL. `ftp_subir` utiliza STOU: el servidor puede asignar un nombre
único diferente del solicitado; esta API todavía no devuelve ese nombre.

No están implementados creación/borrado de carpetas remotas ni administración
completa del protocolo. Las rutas remotas las autoriza el servidor: no corresponden
a la raíz local del proyecto. SFTP se probó con un servidor Paramiko local; FTPS sigue pendiente de validación
con un servidor de laboratorio. Consulta [permisos](INTEGRACIONES.md) y [SSH](INTEGRACIONES.md).

### Cifrado

`cifrado.cifrar(texto, clave, asociados)` utiliza AES-256-GCM de `cryptography`.
`clave()` usa el generador del sistema y devuelve 32 bytes en base64. El formato
`ST1.` incluye un nonce aleatorio de 96 bits seguido del texto cifrado y su etiqueta.
La versión, el algoritmo y el contexto suministrado se autentican como datos asociados.
La clave no se almacena en el sobre. Un nonce nunca debe reutilizarse con la misma
clave; el azar reduce la probabilidad de repetición, no prueba que sea imposible.
Rota las claves y limita su uso. Véase la [documentación oficial de AESGCM](https://cryptography.io/en/latest/hazmat/primitives/aead/).

`descifrar` autentica antes de devolver el texto. Una clave incorrecta, un contexto
diferente o una alteración producen error. `hash` admite SHA256/512;
`hmac` usa SHA256; `comparar` compara bytes sin variar según su contenido.
Base64 **no cifra**.

`hash_clave` guarda PBKDF2-HMAC-SHA256 con sal aleatoria de 16 bytes y 600000
iteraciones en `STP1.600000.`. `verificar_clave` exige ese formato y coste fijo.
Es un formato experimental versionado, sin migración ni política automática de
rotación. No utiliza SHA256 directo para contraseñas.

Ejemplos: [texto](../examples/integraciones/cifrado_texto/src/main.st) y
[contraseñas](../examples/integraciones/contrasenas/src/main.st).
No imprimas claves reales. ST/Python mantienen cadenas en memoria: no se garantiza
su borrado físico. Aún no hay cifrado por bloques de archivos grandes, Argon2id,
almacén de claves, recuperación ni rotación automática. Perder la clave hace
irrecuperable el contenido. Permiso requerido: `cifrado = true`.

### VPN

`vpn.detectar()` informa si encuentra `wg`, `wg-quick`, `openvpn`, `ipsec` e
interfaces locales. Devuelve `estado: no_verificado` y
`administracion_implementada: false`. Tener un cliente instalado no demuestra
que exista un túnel activo.

`vpn.validar(ruta, protocolo)` realiza una inspección conservadora de WireGuard u
OpenVPN. No importa secretos ni activa perfiles. WireGuard admite un Interface y
un Peer sin PrivateKey ni hooks; revisa direcciones y tamaño de clave pública.
OpenVPN exige verificación de servidor/nombre y mínimo TLS 1.2, y rechaza opciones
no enumeradas, scripts, plugins y compresión. Esta revisión sintáctica **no**
comprueba que endpoints, certificados o claves funcionen.

No están implementados conexión, desconexión, IPsec operativo, rutas/DNS,
kill switch, split tunneling, medición de fugas ni gestor gráfico de VPN.
No se ejecutó ninguna modificación de la red del equipo. Para implementar esas
capacidades faltan adaptadores por plataforma, previsualización/aprobación,
rollback verificable y pruebas en laboratorios Linux/Windows.

Ejemplo: [diagnóstico](../examples/integraciones/vpn_diagnostico/src/main.st).
Una VPN tampoco garantiza anonimato absoluto.

### Procesos

`comandos.ejecutar(programa; argumentos_json; segundos)` ejecuta una ruta absoluta
incluida en la lista permitida, sin shell. Acepta hasta 64 argumentos y timeout de
1–30 segundos; limita stdout y stderr combinados a 2048 bytes. Devuelve un JSON
con `codigo`, `stdout` y `stderr`; un código no cero no es éxito. Usa PATH y LANG
filtrados, stdin cerrado y la carpeta del proyecto. Timeout o exceso de salida
terminan el grupo de procesos. Autorizar un programa no limita todas sus posibles
acciones: no hay sandbox, elevación, streaming ni procesos públicos en segundo plano.


## Interfaces graficas

### Dos interfaces distintas

El IDE es una aplicación web local: su menú, terminal y manual se modifican en `ide/`. Las ventanas de tus programas ST usan **PySide6/Qt** y se modifican en `src/stdlib/gui.st`, `runtime/integraciones.py` y `runtime/gui.py`. Cambiar CSS del IDE no cambia las ventanas Qt.

### Recorrido de una llamada

1. Un programa importa `gui` y llama a `gui.crear(diseno)`.
2. `src/stdlib/gui.st` valida/codifica JSON y llama a `integraciones.pedir("gui.crear", ...)`.
3. El compilador nativo comunica la petición al auxiliar `runtime/integraciones.py`. Su registro de operaciones dirige la llamada a `gui_create`.
4. `gui_create` comprueba `[permisos] gui = true`, la dependencia PySide6 y el tamaño del diseño. Arranca `runtime/gui.py` en otro proceso.
5. `GuiSession` administra sus canales. Qt recibe el diseño por stdin; `main` y `build` construyen la ventana en el hilo principal.
6. Las señales de controles llaman a `emit`. ST recupera un evento mediante `gui.evento(id)`.
7. `gui.cambiar_texto` envía una orden; `receive` encuentra el control por su id y lo modifica. `gui.cerrar` libera el proceso.

El código Qt se incrusta desde el runtime privado de `src/codegen/llvm_compat.rs` en el auxiliar del ejecutable. Los cambios del runtime requieren reconstruir el compilador y volver a compilar el programa. Los comentarios añadidos describen estas conexiones sin alterar el protocolo.

### Cada función pública

| Función | Entrada y resultado | Dónde continúa |
|---|---|---|
| `gui.crear(diseno)` | Cadena JSON; retorna identificador entero de sesión | `Runtime.gui_create`, `GuiSession`, `gui.main` |
| `gui.evento(ventana)` | Identificador; retorna objeto JSON en una cadena | `Runtime.gui_event`, `GuiSession.event` |
| `gui.cambiar_texto(ventana, control, texto)` | Id de ventana, id de control y texto; booleano de envío | `Runtime.gui_text`, `GuiSession.send`, `gui.receive` |
| `gui.cerrar(ventana)` | Identificador; libera el recurso | `Runtime.close_resource`, `GuiSession.close` |

`verdadero` en cambiar_texto confirma el envío; un control incompatible puede producir después un evento de error. `evento` no bloquea: retorna `ninguno` si no hay novedades. Mantén una pequeña espera con `time.dormir(20)` entre consultas. Utiliza `finalmente` para cerrar recursos cuando falle el programa.

### Dónde personalizar

- **Título:** propiedad `titulo` del JSON; `main` aplica `setWindowTitle`.
- **Tamaño inicial:** `window.resize(800, 600)` en `runtime/gui.py`.
- **Colores Qt:** `application.setStyleSheet` en ese mismo archivo; se aplica con `tema: "oscuro"`.
- **Distribución:** `QVBoxLayout` de la ventana y de cada panel. Para otras distribuciones, añade propiedades validadas y aplica el layout en `build`.
- **Controles:** ramas de `build`: etiqueta, boton, entrada, clave, texto, casilla, opcion, lista, tabla y panel.
- **Eventos:** conexiones `.connect(...)` de cada control y función `emit`.
- **Órdenes posteriores:** `receive`, junto al emisor correspondiente del auxiliar.
- **Permisos y límites:** `gui_create`, `GuiSession` y validaciones de `build`; conservarlos al ampliar.

Los ids deben ser únicos. El máximo actual es 64 controles y seis niveles. Listas y tablas limitan sus datos iniciales. Las entradas de contraseña no emiten su contenido. La tabla y el texto multilínea no sincronizan automáticamente sus ediciones con ST.

### Agregar un control

Por ejemplo, para implementar una barra de progreso:

1. En `build`, añade una rama para `tipo == "progreso"` que cree `QProgressBar`.
2. Valida `minimo`, `maximo` y `valor`; define límites razonables y rechaza valores incompatibles.
3. Conserva el registro común `widgets[name] = widget` y `parent_layout.addWidget(widget)`.
4. Si debe actualizarse después, implementa una orden `progreso` en `receive` comprobando que el id corresponde a esa clase.
5. Añade un método del auxiliar que valide argumentos, compruebe permiso y use `GuiSession.send`; regístralo como `gui.progreso`.
6. Añade su función pública en `src/stdlib/gui.st` utilizando las funciones JSON, nunca concatenación de texto sin codificar.
7. Prueba ids desconocidos, valores inválidos, límites, cierre y actualización correcta; después actualiza ejemplos y documentación.

Esta es una receta de ampliación: el control progreso todavía no está implementado. El IDE descubre las firmas públicas del módulo mediante `src/st/stdlib.rs` y `ide/language.py`; las sugerencias no implementan por sí mismas una operación del runtime.

### Ejemplos existentes y pruebas

Consulta `examples/integraciones/gui/` para una ventana sencilla y `examples/proyectos_avanzados/20_panel_operaciones/` para eventos, botones, estado local y cierre ordenado. Cada carpeta tiene manifiesto y fuentes comentadas.

```bash
python3 -m unittest discover -s tests -p test_integraciones.py -q
python3 scripts/verify-advanced-projects.py --project 20_panel_operaciones --strict
```

La segunda orden necesita el compilador construido y PySide6 en el intérprete seleccionado. Usa `--python /ruta/absoluta/al/python` si está en un entorno virtual. El laboratorio usa Qt sin pantalla física; una prueba manual complementa la comprobación de apariencia e interacción.

### Manual e inicio desde la terminal del IDE

`/manual.md` se presenta como HTML con índice y colores, usando el mismo renderizador del curso. El texto Markdown original sigue disponible en `/manual-fuente.md`. Modifica `docs/MANUAL_ST.md` para actualizar contenido y `ide/manual.css` para cambiar la presentación.

En la cabecera de salida del IDE pulsa **Inicio** o usa **Ctrl+Alt+H** incluso con el foco en la entrada de terminal. El atajo no se envía al programa ST y el navegador conserva el aviso de cambios sin guardar. Volver al menú no detiene el programa; usa Detener antes de cambiar de proyecto.


## Web y foro

El foro oficial, su registro y acceso están en **website/backend**. No necesitas
generar un proyecto con PyForge para tener la comunidad. Consulta
[su guía de ejecución y despliegue](../website/backend/README.md).

`website/` conserva las páginas del curso, ejemplos y descargas. El generador
`scripts/build-site.py` copia únicamente recursos públicos de su raíz. Nunca
incluye `website/backend` en `dist/site` ni en el ZIP estático.

### Crear una aplicación propia

```bash
pyforge new mi_sitio --web
cd mi_sitio
python3 -m venv .venv
.venv/bin/python -m pip install -r web/requirements.txt
export PYFORGE_WEB_PYTHON="$PWD/.venv/bin/python"
pyforge web
```

La plantilla nueva contiene una vista ST, capturas privadas por sesión y culebrita
con reglas calculadas por el motor ST. No crea un foro ni exige cuenta para jugar.
Los proyectos anteriores de notas, inventario y agenda permanecen disponibles en
`examples/web`. La culebrita completa está en `examples/web/culebrita`.

La lógica compilada está en `src/main.st` y `src/vistas.st`; Django Python recibe
HTTP, gestiona las sesiones y SQL. Cambia la lógica ST y reinicia `pyforge web`.
El servidor pasa datos JSON por stdin a un binario conocido: no compila código
recibido de visitantes. Los límites de CPU/memoria no constituyen una sandbox OS.

El puente anterior se conserva como referencia en `runtime/web_legacy`. Los
proyectos creados anteriormente no se modifican. El nuevo crate Rust es una base
separada, descrita en [INTEGRACIONES.md](INTEGRACIONES.md).

### Añadir campos y vistas

Define el modelo en `web/comunidad/models.py`, su formulario en `forms.py` y la
vista en `views.py`. Filtra consultas y escrituras por `request.user`; nunca tomes
el propietario de un campo del navegador. Registra la URL y crea una plantilla.
Genera y revisa migraciones antes de aplicarlas; no borres bases existentes.

No hay garantía de invulnerabilidad. Producción requiere HTTPS, secreto externo,
hosts explícitos, servidor WSGI, límites del proxy, copias privadas y actualizaciones.


## Base Rust pyforge_django

Se conserva el puente Django Python (`pyforge new --web` y `pyforge web`). El nuevo
crate independiente **crates/pyforge_django** usa axum/Tokio, SQLite/sqlx, Tera y
Argon2id. No reemplaza el compilador ni añade una VM: PyForge continúa siendo AOT.

**La API exacta `importar "django"` con vistas como valores y declaraciones
`modelo` todavía no está conectada al compilador.** El crate ofrece su base Rust y
ABI C para hacerlo; no basta añadir una palabra a KEYWORDS. No debe confundirse
esta base con un reemplazo completo de Django ni anunciarse como lista para producción.

### Construir y probar

```bash
cargo test --manifest-path crates/pyforge_django/Cargo.toml --locked
cargo build --manifest-path crates/pyforge_django/Cargo.toml --release --locked
```

Cargo produce biblioteca Rust, biblioteca compartida y biblioteca estática. El
Cargo.lock del crate fija las versiones; su build no cambia las dependencias del
compilador principal. `include/pyforge_django.h` describe la ABI 1.

### Mapa del código

| Archivo | Responsabilidad y punto de ampliación |
| --- | --- |
| `src/lib.rs` | Contexto opaco, despacho JSON, callbacks y propiedad de memoria de C. |
| `src/server.rs` | Rutas literales, petición, respuesta, sesiones CSRF y servidor local. |
| `src/orm.rs` | Registro de esquemas, validación y consultas SQL parametrizadas. |
| `src/templates.rs` | Registro de plantillas confiables y renderizado con autoescape. |
| `src/security.rs` | Tokens aleatorios, comparación constante y contraseñas Argon2id. |
| `tests/security.rs` | Regresiones de CSRF, origen, SQL y ocultación de claves. |

Cada función documenta su cometido. Las funciones `unsafe` explican el contrato de
punteros: Rust no puede validar un puntero arbitrario que entregue otro lenguaje.

### Convenio de llamadas

`pyforge_django_new("sqlite:aplicacion.db?mode=rwc")` devuelve un contexto o NULL.
Solo el anfitrión configura la ruta, nunca una petición HTTP. Los resultados de
`call` y `orm` son JSON: `{"ok":true,"respuesta":...}` o un error sin detalles SQL.
Libera cada resultado exactamente una vez con `pyforge_django_free`.

`pyforge_django_call` admite `responder_json`, `redirigir` y `renderizar`. En esta ABI
base, `renderizar` recibe el texto **confiable** de una plantilla y un contexto JSON;
no coincide todavía con la firma ST que recibe el nombre de un archivo. Rust también
ofrece `Templates::new` y `render` para registrar fuentes una vez y buscar por nombre.
No construyas fuentes de plantillas a partir de entradas HTTP. Tera escapa cualquier
extensión; un desarrollador aún puede desactivar escape expresamente con `safe` en
una plantilla, por lo que no debe aplicarlo a datos del visitante.

`pyforge_django_route` registra ruta, nombre y callback C con su liberador. El
callback recibe `metodo`, `GET`, `POST` y `CSRF_TOKEN` en JSON. Devuelve una respuesta
`tipo=json` con `datos`, `tipo=html` con `cuerpo`, o `tipo=redireccion` con `url` local.
Las vistas C se serializan por contexto; no son funciones ST automáticamente.

`pyforge_django_listen` bloquea hasta Ctrl+C. Escucha solo 127.0.0.1. El origen
público se configura explícitamente y debe coincidir exactamente en POST. Admite
GET y POST JSON, no multipart ni formularios codificados. Un GET establece una
cookie HttpOnly/SameSite=Strict y entrega el token a la vista. Los POST requieren
cookie, encabezado `X-CSRF-Token` y `Origin` correcto; los inválidos devuelven 403
antes de ejecutar la vista. Las cookies usan Secure con origen HTTPS. Las sesiones
caducan a los 30 minutos, viven en memoria y se limitan a 4096. No equivalen a cuentas
ni autorización de usuarios. El cuerpo se limita a 64 KiB y hay 32 peticiones activas.

### ORM sin SQL libre

El anfitrión envía a `pyforge_django_orm` una operación `registrar` y un `esquema`:

```json
{"operacion":"registrar","esquema":{"nombre":"Usuario","campos":{"correo":{"tipo":"cadena","unique":true},"clave":{"tipo":"clave_segura"}}}}
```

Tipos persistentes iniciales: cadena, entero, flotante, booleano y clave_segura.
`crear` recibe `modelo` y `datos`; `filtrar` recibe `modelo` y `filtros`; `obtener`
recibe `modelo` e `id`; `todos` recibe `modelo`. Las consultas devuelven hasta 100
filas ordenadas por id. No hay fragmentos SQL ni operadores arbitrarios en esta API.
Los nombres de modelo/campo se validan y se citan; los valores se enlazan con bind.

`clave_segura` admite 12–256 bytes, genera sal aleatoria y guarda Argon2id; se omite
siempre en consultas y no puede usarse como filtro. `security::verify_password`
verifica hashes dentro de Rust; todavía falta un servicio de autenticación del ORM.
`unique` se aplica en SQLite. Al reabrir solo se admite el mismo esquema; los cambios
requieren migraciones explícitas. Aún no hay motor automático de migraciones,
relaciones, transacciones de aplicación, eliminación ni actualización de filas.

### Conectar la sintaxis ST solicitada

La sintaxis existente ahora admite `;` entre argumentos y parámetros; conserva
comas anteriores por compatibilidad. Los ejemplos nuevos usan punto y coma.
Para la API completa hacen falta estas piezas coordinadas, todavía pendientes:

1. `src/st/lexer.rs` y `parser.rs`: declaraciones `modelo`, `campo`, `clave_segura`,
   opciones de campo y argumentos nombrados; tipo de función como valor.
2. `src/ast.rs` y análisis semántico: símbolos de modelos y funciones, firmas,
   diccionarios heterogéneos de petición/respuesta y métodos ORM generados.
3. IR, lowering y backend: representación de valores, callbacks de vistas y llamada
   C; enlazar la biblioteca con su runtime y establecer la vida de los objetos.
4. `src/st/stdlib.rs`: registrar `django` cuando el enlace real esté disponible.
5. Pruebas completas: compilar `.st`, atender HTTP, guardar datos, validar CSRF y
   rechazar acceso entre usuarios desde el binario generado.

No se han añadido ejemplos `.st` con declaraciones que el compilador rechace.
`examples/blog_st/{modelos,vistas,urls,main}.st` son lógica de blog **compilable**,
pero no constituyen aún el servidor del DSL propuesto. Las aplicaciones web que sí
funcionan con el puente actual están en `examples/web/{notas,inventario,agenda}`.

### Seguridad y despliegue

Esta base no es una auditoría ni una garantía de invulnerabilidad. Faltan autenticación,
autorización por objeto, limitación distribuida, gestión de sesiones persistentes y
migraciones. Las vistas son código confiable y un callback bloqueado puede consumir
capacidad; no hay aislamiento ni cancelación forzada de código nativo. Producción
necesita HTTPS, proxy, límites, permisos de archivos, copias privadas y actualizaciones.

Referencias: [axum](https://docs.rs/axum/latest/axum/),
[sqlx QueryBuilder](https://docs.rs/sqlx/latest/sqlx/struct.QueryBuilder.html),
[Tera](https://docs.rs/tera/latest/tera/struct.Tera.html) y
[Argon2](https://docs.rs/argon2/latest/argon2/).

## API de integraciones

<!-- API-INTEGRACIONES:INICIO -->
Generada desde `src/stdlib/*.st` por `scripts/update-integrations-api.py`.
Las funciones que devuelven JSON lo indican en su comentario; siguen siendo cadenas ST.

### integraciones

| Firma | Descripción |
|---|---|
| `informacion() -> cadena` | Informa de versión, plataforma y dependencias realmente detectadas, como JSON. |
| `permisos() -> cadena` | Enumera únicamente los permisos declarados en PyForge.toml del directorio actual. |

### archivos

| Firma | Descripción |
|---|---|
| `leer(ruta: cadena, codificacion: cadena) -> cadena` | Lee texto dentro del proyecto con codificación explícita. |
| `escribir_texto(ruta: cadena, texto: cadena, sobrescribir: booleano, codificacion: cadena) -> booleano` | Crea texto; sobrescribir exige verdadero explícito. |
| `crear_carpeta(ruta: cadena) -> booleano` | Crea carpetas locales; requiere permiso de escritura. |
| `listar(ruta: cadena) -> cadena` | Devuelve un arreglo JSON de nombres, hasta 128 entradas. |
| `eliminar(ruta: cadena, confirmar: booleano) -> booleano` | Elimina un archivo o carpeta vacía; exige confirmación. |
| `copiar(origen: cadena, destino: cadena) -> booleano` | Copia sin sobrescribir destinos existentes. |
| `mover(origen: cadena, destino: cadena) -> booleano` | Copia y elimina el original tras guardar correctamente. |
| `existe(ruta: cadena) -> booleano` | Consulta existencia bajo los mismos permisos de lectura. |

### db

| Firma | Descripción |
|---|---|
| `conectar(ruta: cadena) -> entero` | Abre SQLite; devuelve un identificador que debe cerrarse. |
| `ejecutar(conexion: entero, sentencia: cadena, parametros: cadena) -> entero` | Ejecuta una sentencia con parámetros JSON separados; devuelve filas afectadas. |
| `consultar(conexion: entero, sentencia: cadena, parametros: cadena) -> cadena` | Devuelve hasta 100 filas JSON; usa LIMIT para paginar. |
| `confirmar(conexion: entero) -> booleano` | Confirma la transacción SQLite pendiente. |
| `revertir(conexion: entero) -> booleano` | Revierte la transacción SQLite pendiente. |
| `cerrar(conexion: entero) -> booleano` | Cierra y revierte cambios no confirmados. |
| `respaldar(conexion: entero, destino: cadena) -> booleano` | Copia SQLite coherente sin sobrescribir: confirmar o revertir antes de llamar. |

### documentos

| Firma | Descripción |
|---|---|
| `crear(ruta: cadena, texto: cadena, sobrescribir: booleano) -> booleano` | Crea TXT, RTF, DOCX o PDF simple; requiere biblioteca del formato. |
| `leer(ruta: cadena) -> cadena` | Extrae texto TXT, DOCX o PDF; no realiza OCR. |
| `sustituir(ruta: cadena, anterior: cadena, nuevo_texto: cadena) -> booleano` | Edita TXT o runs DOCX; no edita posiciones arbitrarias de un PDF. |
| `convertir(origen: cadena, destino: cadena) -> booleano` | Convierte mediante LibreOffice autorizado sin sobrescribir el destino. |
| `unir_pdf(origenes: cadena, destino: cadena) -> booleano` | Une PDF; origenes es un arreglo JSON de rutas. |
| `extraer_pdf(origen: cadena, destino: cadena, pagina: entero) -> booleano` | Extrae una página usando índice desde cero. |

### red

| Firma | Descripción |
|---|---|
| `tcp_conectar(host: cadena, puerto: entero) -> entero` | Conecta TCP a un host autorizado con timeout de cinco segundos. |
| `tcp_escuchar(host: cadena, puerto: entero) -> entero` | Escucha TCP IPv4 con permiso explícito. |
| `tcp_aceptar(servidor: entero) -> entero` | Acepta una conexión con timeout y devuelve otro identificador. |
| `enviar(conexion: entero, texto: cadena) -> entero` | Envía texto UTF-8 completo por TCP/TLS; devuelve bytes enviados. |
| `recibir(conexion: entero) -> cadena` | Recibe hasta 1024 bytes en base64; TCP puede fragmentar mensajes. |
| `cerrar(conexion: entero) -> booleano` | Cierra el recurso de red y prohíbe volver a utilizarlo. |
| `udp_abrir(host: cadena, puerto: entero) -> entero` | Abre UDP IPv4 local; sin garantía de entrega ni cifrado. |
| `udp_enviar(conexion: entero, host: cadena, puerto: entero, texto: cadena) -> entero` | Envía un datagrama al host autorizado. |
| `udp_recibir(conexion: entero) -> cadena` | Devuelve origen y datos base64; rechaza datagramas truncados. |
| `tls_conectar(host: cadena, puerto: entero, ca: cadena) -> entero` | TLS mínimo 1.2 con CA/nombre verificados; ca vacía usa almacén del sistema. |
| `ssh_conectar(host: cadena, puerto: entero, usuario: cadena, hosts_conocidos: cadena, variable_clave: cadena) -> entero` | SSH verifica known_hosts; contraseña desde variable de entorno autorizada. |
| `sftp_listar(conexion: entero, ruta: cadena) -> cadena` | Lista archivos remotos a través de SSH autenticado. |
| `sftp_subir(conexion: entero, local: cadena, remoto: cadena) -> booleano` | Sube un archivo sin reemplazar uno remoto existente. |
| `sftp_descargar(conexion: entero, remoto: cadena, local: cadena) -> booleano` | Descarga como máximo 8 MiB sin sobrescribir el destino. |
| `ftps_conectar(host: cadena, puerto: entero, usuario: cadena, variable_clave: cadena) -> entero` | FTPS explícito con canal de datos cifrado y certificados verificados. |
| `ftp_listar(conexion: entero, ruta: cadena) -> cadena` | Lista sobre una conexión FTPS; rechaza CR/LF en rutas. |
| `ftp_subir(conexion: entero, local: cadena, remoto: cadena) -> booleano` | Sube por FTPS usando STOU; servidor asigna nombre único. |
| `ftp_descargar(conexion: entero, remoto: cadena, local: cadena) -> booleano` | Descarga por FTPS de forma acotada y exclusiva. |
| `direccion(conexion: entero) -> cadena` | Devuelve host/puerto local y si el recurso usa TLS, como JSON. |
| `enviar_bytes(conexion: entero, contenido: cadena) -> entero` | Envía bytes arbitrarios representados mediante base64 estricto. |

### cifrado

| Firma | Descripción |
|---|---|
| `clave() -> cadena` | Genera clave AES-256 base64 con el CSPRNG; no la imprimas ni guardes en el repositorio. |
| `cifrar(texto: cadena, clave_texto: cadena, asociados: cadena) -> cadena` | AES-256-GCM: nonce aleatorio, versión autenticada y datos asociados. |
| `descifrar(sobre: cadena, clave_texto: cadena, asociados: cadena) -> cadena` | Autentica antes de devolver texto; rechaza modificación y clave incorrecta. |
| `hash(texto: cadena, algoritmo: cadena) -> cadena` | SHA256 o SHA512; no sustituye un hash de contraseñas. |
| `hmac(texto: cadena, clave_texto: cadena) -> cadena` | HMAC-SHA256 con clave base64. |
| `comparar(izquierda: cadena, derecha: cadena) -> booleano` | Compara bytes sin diferencias de tiempo dependientes de su contenido. |
| `hash_clave(clave_texto: cadena) -> cadena` | Hash lento PBKDF2-HMAC-SHA256 con sal aleatoria; guarda el resultado, nunca la contraseña. |
| `verificar_clave(clave_texto: cadena, resumen: cadena) -> booleano` | Comprueba contraseña con el coste y formato STP1; la comparación no filtra su contenido. |
| `base64(texto: cadena) -> cadena` | Base64 es codificación, no cifrado. Facilita transporte de texto sobre APIs binarias. |
| `desde_base64(texto: cadena) -> cadena` | Convierte base64 a UTF-8; los datos que no sean texto producen una excepción. |

### comandos

| Firma | Descripción |
|---|---|
| `ejecutar(programa: cadena, argumentos: cadena, segundos: entero) -> cadena` | Ejecuta ruta autorizada sin shell, timeout y salida limitada; devuelve JSON. |

### sistema

| Firma | Descripción |
|---|---|
| `entorno(nombre: cadena) -> cadena` | Lee una variable de entorno autorizada por su nombre exacto. |

### vpn

| Firma | Descripción |
|---|---|
| `detectar() -> cadena` | Detecta herramientas e interfaces locales; no afirma que haya un túnel activo. |
| `validar(ruta: cadena, protocolo: cadena) -> cadena` | Revisión conservadora de perfil SIN claves; no conecta ni modifica la red. |

### gui

| Firma | Descripción |
|---|---|
| `crear(diseno: cadena) -> entero` | Crea una ventana declarativa; devuelve un identificador de sesión. |
| `evento(ventana: entero) -> cadena` | Retorna un objeto JSON con evento: lista, click, texto, cambio, seleccion, cerrada o ninguno. |
| `cambiar_texto(ventana: entero, control: cadena, texto: cadena) -> booleano` | Cambia el texto de etiqueta, botón o entrada; los fallos llegan como eventos. |
| `cerrar(ventana: entero) -> booleano` | Cierra el proceso gráfico propio y libera sus descriptores. |

<!-- API-INTEGRACIONES:FIN -->

## Convivencia con el runtime Rust nativo

`importar "nativo";` añade archivos y buffers binarios mediante una ABI Rust
incrustada. Su [API y contratos](MANUAL_ST.md#32-runtime-rust-nativo-archivos-y-buffers-binarios)
son independientes del auxiliar Python. Los módulos anteriores permanecen disponibles
y pueden utilizarse en el mismo programa: una cadena producida por `nativo.texto`
se puede entregar a `archivos.escribir_texto`.

El módulo nativo no hereda las capacidades del manifiesto de integraciones: como
las operaciones nativas previas de `io` y `os`, usa los permisos del proceso. No
presentes el manifiesto como aislamiento de un programa ST completo. Python sigue
siendo necesario para las integraciones que lo utilizaban; este incremento no
las migra silenciosamente ni elimina sus dependencias.

Para código nuevo que necesite archivos binarios, `importar "recursos";` ofrece
los tipos `archivo_nativo` y `buffer_nativo`. El bloque `usar` genera el cierre y
evita que un préstamo escape. La API manual `nativo` se conserva para compatibilidad.
Ambas usan los permisos del proceso y no quedan aisladas por el manifiesto del
auxiliar. Consulta la sección 33 del manual antes de elegir una de las dos APIs.

## Motor 3D y audio nativos opcionales

`crates/pyforge_engine3d` no utiliza el auxiliar Python. Es un crate Rust separado
que proporciona ejecutables gráficos y APIs internas para recursos, escena, animación
y audio. Separarlo impide que un programa ST de consola enlace obligatoriamente
`wgpu`, `winit` o `eframe`.

La salida de audio transforma la mezcla espacial en PCM S16LE y abre uno de estos
clientes mediante `std::process::Command`, con argumentos fijos y sin shell:

| Backend | Cliente | Selección |
| --- | --- | --- |
| PipeWire | `pw-cat` | Preferido por `AudioBackend::Auto` |
| ALSA | `aplay` | Alternativa automática |

La API admite bloques sucesivos mediante `AudioPlayback::write_stereo` y una ayuda
asíncrona para un clip mediante `play_spatial_async`. “Asíncrona” aquí significa un
hilo de reproducción de audio; no es el streaming asíncrono de modelos o mundos.
Los clientes heredan el dispositivo y la sesión del usuario. No se ejecutan con
permisos elevados y su ausencia produce un error recuperable.

Los importadores OBJ/glTF/PNG/JPEG son analizadores acotados en CPU. Cargar un archivo
no lo incorpora automáticamente al grafo visible. Las animaciones glTF, materiales,
sombras, IBL y el streaming de recursos aún deben conectarse al renderer. Consulta
las secciones 35–37 del manual y los ejemplos de `crates/pyforge_engine3d/examples`.

El ejecutor `pyforge-game3d` integra lógica ST mediante un protocolo numérico acotado.
Los perfiles `portales`, `cristales`, `laberinto` y `guardianes` comparten ventana,
HUD y viewport sólido. Cada acción crea un proceso ST corto; por ello la integración
es apropiada para estos juegos por turnos, pero no constituye todavía una ABI de
tiempo real. Consulta la sección 38 del manual y `examples/games/JUEGOS_3D.md`.
