"""Auxiliar opcional de ST: JSON local, permisos denegados por defecto.

El compilador incrusta esta fuente. No escucha puertos, no instala paquetes y no
interpreta código recibido. Cada operación debe estar en la tabla de despacho.
No constituye un sandbox para código ST ni para procesos externos autorizados.
"""
import base64
import contextlib
import hashlib
import hmac
import importlib
import importlib.metadata
import io
import json
import os
from pathlib import Path
import platform
import secrets
import shutil
import socket
import sqlite3
import ssl
import subprocess
import sys
import tempfile
import time
import tomllib

GUI_SOURCE = None  # El compilador sustituye este valor por runtime/gui.py incrustado.
VERSION = '1.0-experimental'
MAX_FRAME = 4096
MAX_FILE = 8 * 1024 * 1024
PERMISSIONS = {'leer_archivos', 'escribir_archivos', 'eliminar_archivos',
               'red', 'escuchar_red', 'cifrado', 'base_datos', 'documentos',
               'ejecutar_procesos', 'gui', 'consultar_red', 'leer_entorno'}
DEPENDENCIES = {'docx': ('python-docx', 'python3-docx'),
                'pypdf': ('pypdf', 'python3-pypdf'),
                'reportlab': ('reportlab', 'python3-reportlab'),
                'cryptography': ('cryptography', 'python3-cryptography'),
                'paramiko': ('paramiko', 'python3-paramiko'),
                'PySide6': ('PySide6', 'python3-pyside6.qtwidgets')}


class IntegrationError(Exception):
    """Error público sin contenido de documentos, contraseñas ni salida externa."""
    def __init__(self, code, message):
        self.code = code
        super().__init__(message)


def fail(code, message):
    """Interrumpe una operación con diagnóstico estable que ST puede capturar."""
    raise IntegrationError(code, message)


def dependency(name):
    """Importa solo al usar la capacidad; nunca descarga dependencias."""
    try:
        return importlib.import_module(name)
    except ImportError:
        pip, apt = DEPENDENCIES[name]
        fail('DEPENDENCIA_AUSENTE', f'{pip}: Linux/WSL: sudo apt install {apt}; '
             f'entorno Python Windows: py -m pip install {pip}. ST ejecuta en WSL.')


class GuiSession:
    """Conecta el auxiliar ST con Qt: send escribe órdenes y event lee respuestas.

    Es propietario del proceso gráfico y sus pipes; close solo termina ese hijo.
    Para añadir órdenes conserva la trama JSON por línea y los límites existentes.
    """
    def __init__(self, process):
        """Conserva el proceso y activa canales no bloqueantes para no detener ST."""
        self.process = process
        self.pending = bytearray()
        os.set_blocking(process.stdout.fileno(), False)
        os.set_blocking(process.stdin.fileno(), False)

    def send(self, request):
        """Una trama menor de PIPE_BUF se escribe entera o falla sin bloquear."""
        payload = json.dumps(request, ensure_ascii=True).encode('ascii') + b'\n'
        if len(payload) > 4096:
            fail('LIMITE', 'Orden GUI mayor de 4096 bytes')
        count = os.write(self.process.stdin.fileno(), payload)
        if count != len(payload):
            fail('GUI_CANAL', 'Canal GUI incompleto; cierra la sesión')

    def event(self):
        """Lee hasta una trama; protege memoria y conserva el resto para la próxima consulta."""
        if b'\n' not in self.pending:
            with contextlib.suppress(BlockingIOError):
                self.pending.extend(os.read(self.process.stdout.fileno(), 4096))
        if len(self.pending) > 8192:
            self.close()
            fail('LIMITE', 'Respuesta GUI demasiado grande')
        if b'\n' in self.pending:
            line, _, rest = self.pending.partition(b'\n')
            self.pending[:] = rest
            return json.loads(line)
        return {'evento': 'cerrada' if self.process.poll() is not None else 'ninguno'}

    def close(self):
        """Solicita cierre y termina el proceso propio si no responde en dos segundos."""
        with contextlib.suppress(Exception):
            self.send({'operacion': 'cerrar'})
        try:
            self.process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        self.process.stdin.close()
        self.process.stdout.close()


class Runtime:
    """Una sesión por ejecutable: posee recursos y los cierra al recibir EOF."""
    def __init__(self, root=None, config=None):
        self.root = Path(root or Path.cwd()).resolve()
        if config is None:
            manifest = self.root / 'PyForge.toml'
            config = tomllib.loads(manifest.read_text('utf-8')) if manifest.exists() else {}
        self.permissions = config.get('permisos', {})
        self.options = config.get('integraciones', {})
        if not isinstance(self.permissions, dict) or not isinstance(self.options, dict):
            fail('CONFIGURACION', 'permisos e integraciones deben ser tablas TOML')
        if any(k not in PERMISSIONS or type(v) is not bool for k, v in self.permissions.items()):
            fail('CONFIGURACION', 'Permiso desconocido o valor que no es booleano')
        allowed = {'hosts', 'programas', 'entorno'}
        if any(k not in allowed or not isinstance(v, list) or any(not isinstance(x, str) for x in v)
               for k, v in self.options.items()):
            fail('CONFIGURACION', 'integraciones admite listas hosts, programas y entorno')
        self.resources = {}
        self.counter = 0
        self.operations = {
            'api.informacion': self.info, 'api.permisos': lambda: self.permissions,
            'archivos.leer': self.read, 'archivos.escribir': self.write,
            'archivos.crear_carpeta': self.mkdir, 'archivos.listar': self.listdir,
            'archivos.eliminar': self.remove, 'archivos.copiar': self.copy,
            'archivos.mover': self.move, 'archivos.existe': self.exists,
            'sistema.entorno': self.environment,
            'db.abrir': self.db_open, 'db.ejecutar': self.db_execute,
            'db.consultar': self.db_query, 'db.confirmar': self.db_commit,
            'db.revertir': self.db_rollback, 'db.cerrar': self.close_resource,
            'documentos.crear': self.document_create, 'documentos.leer': self.document_read,
            'documentos.reemplazar': self.document_replace, 'documentos.convertir': self.document_convert,
            'documentos.unir_pdf': self.pdf_merge, 'documentos.extraer_pdf': self.pdf_extract,
            'red.tcp_conectar': self.tcp_connect, 'red.tcp_escuchar': self.tcp_listen,
            'red.tcp_aceptar': self.tcp_accept, 'red.enviar': self.net_send,
            'red.recibir': self.net_recv, 'red.cerrar': self.close_resource,
            'red.udp_abrir': self.udp_open, 'red.udp_enviar': self.udp_send,
            'red.udp_recibir': self.udp_recv, 'red.tls_conectar': self.tls_connect,
            'red.ssh_conectar': self.ssh_connect, 'red.sftp_listar': self.sftp_list,
            'red.sftp_subir': self.sftp_put, 'red.sftp_descargar': self.sftp_get,
            'red.ftps_conectar': self.ftps_connect, 'red.ftp_listar': self.ftp_list,
            'red.ftp_subir': self.ftp_put, 'red.ftp_descargar': self.ftp_get,
            'cifrado.clave': self.crypto_key, 'cifrado.cifrar': self.encrypt,
            'cifrado.descifrar': self.decrypt, 'cifrado.hash': self.digest,
            'cifrado.hmac': self.mac, 'cifrado.comparar': self.compare,
            'comandos.ejecutar': self.process_run,
            'vpn.detectar': self.vpn_detect, 'vpn.validar': self.vpn_validate,
            'cifrado.hash_clave': self.password_hash, 'cifrado.verificar_clave': self.password_verify,
            'cifrado.base64': self.encode_base64, 'cifrado.desde_base64': self.decode_base64,
            'red.direccion': self.socket_info, 'red.enviar_bytes': self.send_bytes,
            'db.respaldar': self.db_backup,
            # Registro público: debe coincidir con las cadenas de src/stdlib/gui.st.
            'gui.crear': self.gui_create, 'gui.evento': self.gui_event,
            'gui.texto': self.gui_text, 'gui.cerrar': self.close_resource,
        }

    def require(self, permission):
        """Cada punto de entrada exige una capacidad declarada, sin elevar privilegios."""
        if not self.permissions.get(permission, False):
            fail('PERMISO_DENEGADO', f'Declara [permisos] {permission} = true tras revisar el programa')

    def path(self, value, permission):
        """Rechaza rutas absolutas, padres y enlaces: raíz local, no aislamiento OS."""
        self.require(permission)
        if not isinstance(value, str) or not value or '\x00' in value:
            fail('RUTA_INVALIDA', 'Se requiere una ruta relativa no vacía')
        path = Path(value)
        if path.is_absolute() or '..' in path.parts:
            fail('RUTA_DENEGADA', 'Usa una ruta relativa dentro del proyecto')
        current = self.root
        for part in path.parts:
            current = current / part
            if current.is_symlink():
                fail('RUTA_DENEGADA', 'No se admiten enlaces simbólicos')
        resolved = current.resolve()
        if resolved == self.root or not resolved.is_relative_to(self.root):
            fail('RUTA_DENEGADA', 'No se puede operar sobre la raíz del proyecto')
        if (any(p in {'.git', '.ssh', '.gnupg', '.codex'} for p in path.parts) or
                path.name.startswith('.env') or path.name in {'id_rsa', 'id_ed25519'} or
                path.suffix.lower() in {'.key', '.p12', '.pfx'}):
            fail('RUTA_DENEGADA', 'Directorio protegido')
        return resolved

    def read_bytes(self, path):
        """Lee con límite antes de devolver datos; rechaza dispositivos y tuberías."""
        if not path.is_file():
            fail('ARCHIVO_INVALIDO', 'Se requiere un archivo regular existente')
        with path.open('rb') as source:
            data = source.read(MAX_FILE + 1)
        if len(data) > MAX_FILE:
            fail('LIMITE', 'Archivo mayor de 8 MiB')
        return data

    def save_bytes(self, path, data, overwrite=False):
        """Publica sin sobrescribir por defecto; el reemplazo autorizado es atómico."""
        if len(data) > MAX_FILE:
            fail('LIMITE', 'Contenido mayor de 8 MiB')
        with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as tmp:
            temp = Path(tmp.name)
            try:
                tmp.write(data)
                tmp.flush()
                os.fsync(tmp.fileno())
                if overwrite:
                    os.replace(temp, path)
                else:
                    os.link(temp, path)  # falla si ya existe: no hay carrera check/write
            finally:
                temp.unlink(missing_ok=True)

    def add(self, kind, value):
        """Devuelve un identificador opaco; ST nunca recibe punteros Python."""
        if len(self.resources) >= 64:
            value.close()
            fail('LIMITE', 'Máximo 64 recursos abiertos; cierra los que no uses')
        self.counter += 1
        self.resources[self.counter] = (kind, value)
        return self.counter

    def resource(self, handle, kinds):
        """Comprueba tipo y vida del recurso para evitar uso tras cierre."""
        item = self.resources.get(handle)
        if item is None or item[0] not in kinds:
            fail('RECURSO_INVALIDO', 'Identificador cerrado, inexistente o de otro tipo')
        return item[1]

    def close_resource(self, handle):
        """Cierra una vez; cerrar una base sin confirmar revierte su transacción."""
        self.resource(handle, {'db', 'socket', 'ssh', 'ftps', 'gui'})
        _, value = self.resources.pop(handle)
        value.close()
        return True

    def close(self):
        """Libera recursos al terminar incluso después de una solicitud inválida."""
        for handle in list(self.resources):
            with contextlib.suppress(Exception):
                self.close_resource(handle)

    def info(self):
        """Describe dependencias presentes sin importar módulos ni consultar la red."""
        packages = {}
        for name, (package, _) in DEPENDENCIES.items():
            try:
                packages[name] = importlib.metadata.version(package)
            except importlib.metadata.PackageNotFoundError:
                packages[name] = None
        return {'version': VERSION, 'plataforma': platform.system(),
                'python': platform.python_version(), 'dependencias': packages,
                'libreoffice': bool(shutil.which('libreoffice')),
                'sandbox': False}

    def read(self, name, encoding='utf-8'):
        """Lee texto con codificación explícita y errores estrictos."""
        return self.read_bytes(self.path(name, 'leer_archivos')).decode(encoding)

    def write(self, name, text, overwrite=False, encoding='utf-8'):
        """Crea texto; sobrescribir requiere el argumento explícito verdadero."""
        self.save_bytes(self.path(name, 'escribir_archivos'), text.encode(encoding), overwrite)
        return True

    def mkdir(self, name):
        """Crea directorios locales con permisos sujetos a umask."""
        self.path(name, 'escribir_archivos').mkdir(mode=0o700, parents=True, exist_ok=True)
        return True

    def listdir(self, name):
        """Lista nombres, nunca contenido; limita la respuesta antes del transporte."""
        path = self.path(name, 'leer_archivos')
        result = sorted(p.name for p in path.iterdir())
        if len(result) > 128:
            fail('LIMITE', 'Máximo 128 entradas por listado')
        return result

    def exists(self, name):
        """Consulta existencia solo dentro de las rutas permitidas."""
        return self.path(name, 'leer_archivos').exists()

    def remove(self, name, confirm=False):
        """Elimina archivos o carpetas vacías; nunca borra árboles recursivamente."""
        if confirm is not True:
            fail('CONFIRMACION', 'La eliminación requiere confirmar = verdadero')
        path = self.path(name, 'eliminar_archivos')
        path.rmdir() if path.is_dir() else path.unlink()
        return True

    def copy(self, source, destination):
        """Copia un archivo regular sin reemplazar el destino."""
        self.save_bytes(self.path(destination, 'escribir_archivos'),
                        self.read_bytes(self.path(source, 'leer_archivos')))
        return True

    def move(self, source, destination):
        """Copia y elimina el origen únicamente tras publicar el destino."""
        original = self.path(source, 'eliminar_archivos')
        self.copy(source, destination)
        original.unlink()
        return True

    def environment(self, name):
        """Lee solo nombres autorizados explícitamente; no enumera ni registra secretos."""
        self.require('leer_entorno')
        if name not in self.options.get('entorno', []):
            fail('PERMISO_DENEGADO', 'Variable de entorno fuera de la lista autorizada')
        return os.environ.get(name, '')

    def db_open(self, name):
        """Abre SQLite local con espera acotada y claves foráneas activadas."""
        self.require('base_datos')
        path = self.path(name, 'escribir_archivos')
        self.path(name, 'leer_archivos')
        if path.exists() and not path.is_file():
            fail('ARCHIVO_INVALIDO', 'SQLite requiere un archivo regular')
        db = sqlite3.connect(path, timeout=3)
        db.execute('PRAGMA foreign_keys = ON')
        db.setlimit(sqlite3.SQLITE_LIMIT_LENGTH, MAX_FILE)
        db.setlimit(sqlite3.SQLITE_LIMIT_SQL_LENGTH, MAX_FRAME)
        db.setlimit(sqlite3.SQLITE_LIMIT_ATTACHED, 0)
        # Prohíbe adjuntar/escribir otras rutas y cargar extensiones desde SQL.
        def authorize(action, a, b, database, trigger):
            if action in {sqlite3.SQLITE_ATTACH, sqlite3.SQLITE_DETACH, sqlite3.SQLITE_PRAGMA}:
                return sqlite3.SQLITE_DENY
            if action == sqlite3.SQLITE_FUNCTION and b and b.lower() in {'load_extension', 'writefile', 'readfile'}:
                return sqlite3.SQLITE_DENY
            return sqlite3.SQLITE_OK
        db.set_authorizer(authorize)
        return self.add('db', db)

    def db_cursor(self, handle, sql, parameters):
        """Ejecuta una sentencia parametrizada con presupuesto de instrucciones."""
        self.require('base_datos')
        if not isinstance(parameters, list) or len(parameters) > 128:
            fail('ARGUMENTO', 'Parámetros SQL: arreglo JSON de hasta 128 valores')
        db = self.resource(handle, {'db'})
        deadline = time.monotonic() + 3
        db.set_progress_handler(lambda: int(time.monotonic() > deadline), 1000)
        return db.execute(sql, parameters)

    def db_execute(self, handle, sql, parameters):
        """Devuelve filas afectadas; el programa debe confirmar explícitamente."""
        return self.db_cursor(handle, sql, parameters).rowcount

    def db_query(self, handle, sql, parameters):
        """Devuelve hasta 100 filas JSON; no materializa consultas ilimitadas."""
        cursor = self.db_cursor(handle, sql, parameters)
        rows = cursor.fetchmany(101)
        if len(rows) > 100:
            fail('LIMITE', 'Consulta mayor de 100 filas; utiliza LIMIT y OFFSET')
        return [list(row) for row in rows]

    def db_commit(self, handle):
        """Confirma los cambios de la transacción actual."""
        self.resource(handle, {'db'}).commit()
        return True

    def db_rollback(self, handle):
        """Revierte los cambios todavía no confirmados."""
        self.resource(handle, {'db'}).rollback()
        return True

    def document_create(self, name, text, overwrite=False):
        """Crea documentos de texto sencillos; formato elegido por extensión."""
        self.require('documentos')
        path = self.path(name, 'escribir_archivos')
        extension = path.suffix.lower()
        data = io.BytesIO()
        if extension == '.txt':
            data.write(text.encode('utf-8'))
        elif extension == '.rtf':
            escaped = ''.join('\\' + c if c in '\\{}' else ('\\par\n' if c == '\n' else
                              (c if ord(c) < 128 else ''.join('\\u' + str(int.from_bytes(unit, 'little', signed=True)) + '?'
                               for unit in [c.encode('utf-16-le')[i:i+2] for i in range(0, len(c.encode('utf-16-le')), 2)]))) for c in text)
            data.write(('{\\rtf1\\ansi\\uc1 ' + escaped + '}').encode('ascii'))
        elif extension == '.docx':
            module = dependency('docx')
            doc = module.Document()
            for paragraph in text.split('\n'):
                doc.add_paragraph(paragraph)
            doc.save(data)
        elif extension == '.pdf':
            dependency('reportlab')
            from reportlab.pdfgen import canvas
            pdf = canvas.Canvas(data)
            y = 800
            for line in text.split('\n'):
                if y < 50:
                    pdf.showPage()
                    y = 800
                pdf.drawString(50, y, line)
                y -= 16
            pdf.save()
        elif extension == '.doc':
            fail('CONVERSION_REQUERIDA', 'Crea DOCX y usa documentos.convertir con LibreOffice')
        else:
            fail('FORMATO', 'Creación admitida: txt, rtf, docx, pdf; DB se gestiona con db')
        self.save_bytes(path, data.getvalue(), overwrite)
        return True

    def document_read(self, name):
        """Extrae texto; no promete conservar estilos ni OCR de PDF escaneado."""
        self.require('documentos')
        path = self.path(name, 'leer_archivos')
        data = io.BytesIO(self.read_bytes(path))
        if path.suffix.lower() == '.txt':
            return data.getvalue().decode('utf-8')
        if path.suffix.lower() == '.docx':
            doc = dependency('docx').Document(data)
            return '\n'.join(p.text for p in doc.paragraphs)
        if path.suffix.lower() == '.pdf':
            pdf = dependency('pypdf').PdfReader(data, strict=True)
            return '\n'.join(page.extract_text() or '' for page in pdf.pages)
        fail('FORMATO', 'Lectura directa: txt, docx, pdf. Convierte DOC/RTF con LibreOffice')

    def document_replace(self, name, old, new):
        """Reemplaza TXT o texto contenido en un run DOCX; guarda atómicamente."""
        self.require('documentos')
        path = self.path(name, 'leer_archivos')
        self.path(name, 'escribir_archivos')
        raw = self.read_bytes(path)
        if path.suffix.lower() == '.txt':
            self.save_bytes(path, raw.decode('utf-8').replace(old, new).encode('utf-8'), True)
        elif path.suffix.lower() == '.docx':
            doc = dependency('docx').Document(io.BytesIO(raw))
            for paragraph in doc.paragraphs:
                for run in paragraph.runs:
                    run.text = run.text.replace(old, new)
            data = io.BytesIO()
            doc.save(data)
            self.save_bytes(path, data.getvalue(), True)
        else:
            fail('EDICION_LIMITADA', 'Reemplazo: TXT y runs DOCX. PDF no es un procesador de texto')
        return True

    def document_convert(self, source, destination):
        """Usa LibreOffice en un perfil temporal aislado y sin shell; no descarga nada."""
        self.require('documentos')
        self.require('ejecutar_procesos')
        program = shutil.which('libreoffice')
        if not program:
            fail('DEPENDENCIA_AUSENTE', 'LibreOffice: Linux/WSL sudo apt install libreoffice')
        src = self.path(source, 'leer_archivos')
        dst = self.path(destination, 'escribir_archivos')
        extension = dst.suffix.lower().lstrip('.')
        if extension not in {'doc', 'docx', 'pdf', 'rtf', 'txt'}:
            fail('FORMATO', 'Destino de conversión no admitido')
        with tempfile.TemporaryDirectory(prefix='st-documento-') as folder:
            root = Path(folder)
            local = root / ('entrada' + src.suffix)
            local.write_bytes(self.read_bytes(src))
            output = root / 'salida'
            output.mkdir()
            result = subprocess.run([program, '-env:UserInstallation=' + (root / 'perfil').as_uri(),
                                     '--headless', '--convert-to', extension, '--outdir', str(output), str(local)],
                                    stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                    stderr=subprocess.DEVNULL, timeout=30, env={'PATH': '/usr/bin:/bin'})
            expected = output / ('entrada.' + extension)
            if result.returncode or not expected.is_file():
                fail('CONVERSION', 'LibreOffice no generó el archivo solicitado')
            self.save_bytes(dst, self.read_bytes(expected))
        return True

    def pdf_merge(self, sources, destination):
        """Une páginas de archivos PDF dentro del proyecto sin sobrescribir salida."""
        self.require('documentos')
        module = dependency('pypdf')
        writer = module.PdfWriter()
        if not isinstance(sources, list) or not 1 <= len(sources) <= 20:
            fail('ARGUMENTO', 'Se requieren entre 1 y 20 PDF')
        for source in sources:
            writer.append(io.BytesIO(self.read_bytes(self.path(source, 'leer_archivos'))))
        data = io.BytesIO()
        writer.write(data)
        self.save_bytes(self.path(destination, 'escribir_archivos'), data.getvalue())
        return True

    def pdf_extract(self, source, destination, page):
        """Extrae una página usando índice cero, con validación de límites."""
        self.require('documentos')
        module = dependency('pypdf')
        reader = module.PdfReader(io.BytesIO(self.read_bytes(self.path(source, 'leer_archivos'))))
        if type(page) is not int or not 0 <= page < len(reader.pages):
            fail('INDICE', 'Página PDF fuera de rango')
        writer = module.PdfWriter()
        writer.add_page(reader.pages[page])
        data = io.BytesIO()
        writer.write(data)
        self.save_bytes(self.path(destination, 'escribir_archivos'), data.getvalue())
        return True

    def endpoint(self, host, port, listening=False):
        """Autoriza host exacto; no interpreta URL ni comodines de destinos."""
        self.require('escuchar_red' if listening else 'red')
        if host not in self.options.get('hosts', []):
            fail('DESTINO_DENEGADO', 'Host fuera de integraciones.hosts')
        if type(port) is not int or not 1 <= port <= 65535:
            fail('PUERTO', 'Puerto permitido: 1 a 65535')
        return host, port

    def tcp_connect(self, host, port):
        """TCP cliente con tiempo máximo de cinco segundos por operación."""
        return self.add('socket', socket.create_connection(self.endpoint(host, port), timeout=5))

    def tcp_listen(self, host, port):
        """Servidor IPv4 explícito; aceptar también tiene timeout."""
        endpoint = self.endpoint(host, port, True)
        sock = socket.socket()
        try:
            sock.settimeout(5)
            sock.bind(endpoint)
            sock.listen(8)
            return self.add('socket', sock)
        except Exception:
            sock.close()
            raise

    def tcp_accept(self, handle):
        """Acepta una conexión sin bloquear indefinidamente."""
        self.require('escuchar_red')
        client, _ = self.resource(handle, {'socket'}).accept()
        client.settimeout(5)
        return self.add('socket', client)

    def net_send(self, handle, text):
        """Envía todo el texto UTF-8 por TCP/TLS; no agrega delimitadores."""
        self.resource(handle, {'socket'}).sendall(text.encode('utf-8'))
        return len(text.encode('utf-8'))

    def net_recv(self, handle):
        """Devuelve bytes en base64: respeta fragmentación TCP y UTF-8 incompleto."""
        return base64.b64encode(self.resource(handle, {'socket'}).recv(1024)).decode('ascii')

    def udp_open(self, host, port):
        """Abre UDP IPv4 local; UDP no garantiza entrega, orden ni cifrado."""
        address = self.endpoint(host, port, True)
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        try:
            sock.settimeout(5)
            sock.bind(address)
            return self.add('socket', sock)
        except Exception:
            sock.close()
            raise

    def udp_send(self, handle, host, port, text):
        """Envía un datagrama a un destino autorizado."""
        return self.resource(handle, {'socket'}).sendto(text.encode('utf-8'), self.endpoint(host, port))

    def udp_recv(self, handle):
        """Incluye origen y datos base64; detecta truncamiento en Linux."""
        sock = self.resource(handle, {'socket'})
        data, _, flags, address = sock.recvmsg(1024)
        if flags & socket.MSG_TRUNC:
            fail('LIMITE', 'Datagrama mayor de 1024 bytes: descartado')
        return {'datos_base64': base64.b64encode(data).decode('ascii'), 'host': address[0], 'puerto': address[1]}

    def tls_connect(self, host, port, ca=''):
        """TLS cliente: CA y nombre obligatoriamente verificados, mínimo TLS 1.2."""
        address = self.endpoint(host, port)
        context = ssl.create_default_context(cafile=str(self.path(ca, 'leer_archivos')) if ca else None)
        context.minimum_version = ssl.TLSVersion.TLSv1_2
        raw = socket.create_connection(address, timeout=5)
        try:
            secured = context.wrap_socket(raw, server_hostname=host)
            return self.add('socket', secured)
        except Exception:
            raw.close()
            raise

    def ssh_connect(self, host, port, user, known_hosts, password_env):
        """SSH rechaza claves desconocidas; la contraseña procede del entorno autorizado."""
        self.endpoint(host, port)
        module = dependency('paramiko')
        client = module.SSHClient()
        client.load_host_keys(str(self.path(known_hosts, 'leer_archivos')))
        client.set_missing_host_key_policy(module.RejectPolicy())
        try:
            client.connect(host, port=port, username=user, password=self.environment(password_env),
                           timeout=5, banner_timeout=5, auth_timeout=5, look_for_keys=False, allow_agent=False)
            return self.add('ssh', client)
        except Exception:
            client.close()
            raise

    def sftp_list(self, handle, remote):
        """Lista una carpeta remota autenticada; no acepta hosts automáticamente."""
        with self.resource(handle, {'ssh'}).open_sftp() as sftp:
            sftp.get_channel().settimeout(5)
            entries = sftp.listdir(remote)
            if len(entries) > 128:
                fail('LIMITE', 'Listado SFTP mayor de 128 entradas')
            return entries

    def sftp_put(self, handle, local, remote):
        """Sube sin sobrescribir el archivo remoto existente."""
        data = self.read_bytes(self.path(local, 'leer_archivos'))
        with self.resource(handle, {'ssh'}).open_sftp() as sftp:
            sftp.get_channel().settimeout(5)
            with sftp.file(remote, 'wx') as stream:
                stream.write(data)
        return True

    def sftp_get(self, handle, remote, local):
        """Descarga con límite y publicación local exclusiva."""
        target = self.path(local, 'escribir_archivos')
        with self.resource(handle, {'ssh'}).open_sftp() as sftp:
            sftp.get_channel().settimeout(5)
            with sftp.file(remote, 'rb') as stream:
                data = stream.read(MAX_FILE + 1)
        self.save_bytes(target, data)
        return True

    def ftps_connect(self, host, port, user, password_env):
        """FTPS explícito verifica TLS y protege el canal de datos (PROT P)."""
        import ftplib
        self.endpoint(host, port)
        context = ssl.create_default_context()
        context.minimum_version = ssl.TLSVersion.TLSv1_2
        client = ftplib.FTP_TLS(context=context, timeout=5)
        try:
            client.connect(host, port)
            client.login(user, self.environment(password_env))
            client.prot_p()
            return self.add('ftps', client)
        except Exception:
            client.close()
            raise

    def ftp_path(self, name):
        """Evita inyección de órdenes de control FTP por CR/LF."""
        if not isinstance(name, str) or any(c in name for c in '\r\n\x00'):
            fail('RUTA_INVALIDA', 'Nombre FTP inválido')
        return name

    def ftp_list(self, handle, remote):
        """Lista FTPS mediante canal de datos cifrado."""
        result = []
        def collect(line):
            if len(result) >= 128:
                fail('LIMITE', 'Listado FTP demasiado grande')
            result.append(line)
        self.resource(handle, {'ftps'}).retrlines('NLST ' + self.ftp_path(remote), collect)
        return result

    def ftp_put(self, handle, local, remote):
        """Sube con nombre único asignado por el servidor mediante STOU."""
        data = self.read_bytes(self.path(local, 'leer_archivos'))
        self.resource(handle, {'ftps'}).storbinary('STOU ' + self.ftp_path(remote), io.BytesIO(data))
        return True

    def ftp_get(self, handle, remote, local):
        """Descarga FTPS con límite y sin sobrescribir destinos locales."""
        target = self.path(local, 'escribir_archivos')
        output = io.BytesIO()
        def collect(block):
            if output.tell() + len(block) > MAX_FILE:
                fail('LIMITE', 'Descarga mayor de 8 MiB')
            output.write(block)
        self.resource(handle, {'ftps'}).retrbinary('RETR ' + self.ftp_path(remote), collect)
        self.save_bytes(target, output.getvalue())
        return True

    def crypto_key(self):
        """Genera 256 bits del CSPRNG del sistema; no escribe ni registra la clave."""
        self.require('cifrado')
        return base64.b64encode(secrets.token_bytes(32)).decode('ascii')

    def key_bytes(self, key):
        """Valida una clave AES-256 codificada en base64 estricto."""
        raw = base64.b64decode(key, validate=True)
        if len(raw) != 32:
            fail('CLAVE', 'AES-256 necesita 32 bytes en base64')
        return raw

    def encrypt(self, text, key, associated=''):
        """AES-256-GCM de cryptography; nonce aleatorio de 96 bits y versión autenticada."""
        self.require('cifrado')
        dependency('cryptography')
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM
        nonce = secrets.token_bytes(12)
        ad = b'ST1:AES256GCM:' + associated.encode('utf-8')
        result = AESGCM(self.key_bytes(key)).encrypt(nonce, text.encode('utf-8'), ad)
        return 'ST1.' + base64.b64encode(nonce + result).decode('ascii')

    def decrypt(self, envelope, key, associated=''):
        """Autentica antes de devolver texto: clave incorrecta o alteración se rechazan."""
        self.require('cifrado')
        dependency('cryptography')
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM
        if not envelope.startswith('ST1.'):
            fail('FORMATO_CIFRADO', 'Versión de cifrado desconocida')
        raw = base64.b64decode(envelope[4:], validate=True)
        if len(raw) < 28:
            fail('FORMATO_CIFRADO', 'Sobre cifrado incompleto')
        try:
            result = AESGCM(self.key_bytes(key)).decrypt(raw[:12], raw[12:], b'ST1:AES256GCM:' + associated.encode('utf-8'))
        except Exception:
            fail('AUTENTICACION', 'Clave incorrecta, datos asociados distintos o contenido alterado')
        return result.decode('utf-8')

    def digest(self, text, algorithm='sha256'):
        """Hash de integridad; no debe usarse como almacenamiento de contraseñas."""
        self.require('cifrado')
        if algorithm not in {'sha256', 'sha512'}:
            fail('ALGORITMO', 'Solo sha256 y sha512')
        return hashlib.new(algorithm, text.encode('utf-8')).hexdigest()

    def mac(self, text, key):
        """HMAC-SHA256 con clave base64 de 256 bits."""
        self.require('cifrado')
        return hmac.new(self.key_bytes(key), text.encode('utf-8'), hashlib.sha256).hexdigest()

    def compare(self, left, right):
        """Comparación de bytes constante respecto al contenido, no a la longitud."""
        self.require('cifrado')
        return hmac.compare_digest(left.encode('utf-8'), right.encode('utf-8'))

    def process_run(self, program, arguments, seconds=5):
        """Ejecuta sin shell, con entorno filtrado y salida acotada mediante selectores."""
        import selectors
        self.require('ejecutar_procesos')
        path = Path(program)
        if not path.is_absolute() or str(path) not in self.options.get('programas', []):
            fail('PROGRAMA_DENEGADO', 'Usa ruta absoluta incluida en integraciones.programas')
        if (not isinstance(arguments, list) or len(arguments) > 64 or
                any(not isinstance(a, str) or '\x00' in a for a in arguments)):
            fail('ARGUMENTO', 'Argumentos: arreglo de hasta 64 cadenas sin NUL')
        if type(seconds) is not int or not 1 <= seconds <= 30:
            fail('TIEMPO', 'Tiempo máximo permitido: 1 a 30 segundos')
        env = {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8'}
        process = subprocess.Popen([str(path), *arguments], cwd=self.root, env=env,
                                   stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, start_new_session=True)
        output = {'stdout': bytearray(), 'stderr': bytearray()}
        deadline = time.monotonic() + seconds
        try:
            with selectors.DefaultSelector() as selector:
                for stream, name in [(process.stdout, 'stdout'), (process.stderr, 'stderr')]:
                    os.set_blocking(stream.fileno(), False)
                    selector.register(stream, selectors.EVENT_READ, name)
                while selector.get_map():
                    if time.monotonic() > deadline:
                        fail('TIEMPO_AGOTADO', 'Proceso cancelado por tiempo máximo')
                    for event, _ in selector.select(0.05):
                        block = os.read(event.fileobj.fileno(), 1024)
                        if not block:
                            selector.unregister(event.fileobj)
                        else:
                            output[event.data].extend(block)
                            if sum(map(len, output.values())) > 2048:
                                fail('LIMITE', 'Proceso cancelado: salida mayor de 2048 bytes')
                code = process.wait(timeout=max(0.01, deadline - time.monotonic()))
            return {'codigo': code, **{k: bytes(v).decode('utf-8', 'replace') for k, v in output.items()}}
        finally:
            # El grupo es creado por esta llamada; nunca termina procesos ajenos.
            import signal
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            process.stdout.close()
            process.stderr.close()

    def vpn_detect(self):
        """Detección local solamente: no toca DNS, rutas, interfaces ni firewall."""
        self.require('consultar_red')
        return {'herramientas': {name: bool(shutil.which(name)) for name in ['wg', 'wg-quick', 'openvpn', 'ipsec']},
                'interfaces': [name for _, name in socket.if_nameindex()],
                'estado': 'no_verificado', 'administracion_implementada': False}

    def password_hash(self, password):
        """PBKDF2-HMAC-SHA256, sal aleatoria y coste explícito; no usa un hash rápido."""
        self.require('cifrado')
        salt = secrets.token_bytes(16)
        value = hashlib.pbkdf2_hmac('sha256', password.encode('utf-8'), salt, 600000)
        return 'STP1.600000.' + base64.b64encode(salt + value).decode('ascii')

    def password_verify(self, password, encoded):
        """Valida formato y coste fijo antes de comparar; evita costes inyectados."""
        self.require('cifrado')
        if not encoded.startswith('STP1.600000.'):
            fail('FORMATO', 'Hash de contraseña no admitido')
        data = base64.b64decode(encoded[12:], validate=True)
        if len(data) != 48:
            fail('FORMATO', 'Hash de contraseña incompleto')
        value = hashlib.pbkdf2_hmac('sha256', password.encode('utf-8'), data[:16], 600000)
        return hmac.compare_digest(value, data[16:])

    def encode_base64(self, text):
        """Codificación reversible: no proporciona cifrado ni integridad."""
        return base64.b64encode(text.encode('utf-8')).decode('ascii')

    def decode_base64(self, text):
        """Decodifica base64 estricto y UTF-8; falla con bytes que no sean texto."""
        return base64.b64decode(text, validate=True).decode('utf-8')

    def socket_info(self, handle):
        """Obtiene dirección local; no interpreta una conexión como garantía de seguridad."""
        sock = self.resource(handle, {'socket'})
        local = sock.getsockname()
        return {'host': local[0], 'puerto': local[1], 'tls': isinstance(sock, ssl.SSLSocket)}

    def send_bytes(self, handle, encoded):
        """Envía bytes base64 por TCP/TLS para no forzar UTF-8 en protocolos binarios."""
        data = base64.b64decode(encoded, validate=True)
        self.resource(handle, {'socket'}).sendall(data)
        return len(data)

    def db_backup(self, handle, destination):
        """Copia coherente con la API SQLite; requiere confirmar primero la transacción."""
        self.require('base_datos')
        db = self.resource(handle, {'db'})
        if db.in_transaction:
            fail('TRANSACCION', 'Confirma o revierte antes de crear una copia SQLite')
        target = self.path(destination, 'escribir_archivos')
        with tempfile.TemporaryDirectory(dir=target.parent) as folder:
            path = Path(folder) / 'copia.db'
            backup = sqlite3.connect(path)
            try:
                deadline = time.monotonic() + 5
                def progress(status, remaining, total):
                    if time.monotonic() > deadline:
                        fail('TIEMPO_AGOTADO', 'Copia SQLite excedió cinco segundos')
                db.backup(backup, pages=64, progress=progress)
            finally:
                backup.close()
            self.save_bytes(target, self.read_bytes(path))
        return True

    def vpn_validate(self, name, kind):
        """Inspección conservadora: rechaza claves embebidas y órdenes ejecutables.

        No activa el perfil ni afirma validez de credenciales o conectividad.
        La administración real de interfaces/rutas/firewall permanece pendiente.
        """
        self.require('consultar_red')
        text = self.read(name)
        if kind == 'wireguard':
            import configparser
            config = configparser.ConfigParser(interpolation=None, strict=True)
            config.read_string(text)
            allowed = {'Interface': {'address', 'dns', 'mtu', 'listenport'},
                       'Peer': {'publickey', 'endpoint', 'allowedips', 'persistentkeepalive'}}
            if set(config.sections()) != set(allowed):
                fail('PERFIL', 'Se requiere Interface y un Peer; múltiples peers no implementados')
            if any(key not in allowed[section] for section in config for key in config[section]):
                fail('PERFIL', 'Perfil contiene claves privadas, hooks u opciones no admitidas')
            import ipaddress
            for value in config['Interface'].get('Address', '').split(','):
                ipaddress.ip_interface(value.strip())
            for value in config['Peer'].get('AllowedIPs', '').split(','):
                ipaddress.ip_network(value.strip(), strict=False)
            key = base64.b64decode(config['Peer'].get('PublicKey', ''), validate=True)
            if len(key) != 32:
                fail('PERFIL', 'Clave pública WireGuard inválida')
        elif kind == 'openvpn':
            import shlex
            allowed = {'client', 'dev', 'proto', 'remote', 'nobind', 'persist-key', 'persist-tun',
                       'remote-cert-tls', 'verify-x509-name', 'tls-version-min', 'ca', 'verb',
                       'auth-nocache', 'data-ciphers', 'resolv-retry'}
            entries = {}
            for line in text.splitlines():
                parts = shlex.split(line, comments=True)
                if not parts:
                    continue
                if parts[0] not in allowed or parts[0] in entries:
                    fail('PERFIL', 'OpenVPN: opción no admitida, duplicada o potencialmente ejecutable')
                entries[parts[0]] = parts[1:]
            if (entries.get('remote-cert-tls') != ['server'] or
                    not entries.get('verify-x509-name') or
                    entries.get('tls-version-min') not in [['1.2'], ['1.3']]):
                fail('PERFIL', 'Exige remote-cert-tls server, verify-x509-name y TLS >= 1.2')
        else:
            fail('PERFIL', 'Inspección disponible para wireguard u openvpn')
        return {'revision_sintactica': True, 'conectividad_verificada': False,
                'activacion_disponible': False, 'secretos_importados': False}

    def gui_create(self, specification):
        """Inicia Qt en un proceso dedicado: el hilo gráfico nunca realiza E/S ST."""
        self.require('gui')
        dependency('PySide6')
        if not isinstance(specification, dict):
            fail('GUI_CONFIGURACION', 'Se requiere un objeto JSON declarativo')
        payload = json.dumps(specification, ensure_ascii=True).encode('ascii') + b'\n'
        if len(payload) > 4096:
            fail('LIMITE', 'Definición GUI mayor de 4096 bytes')
        # En ejecutables se usa la copia incrustada al compilar, no un archivo del usuario.
        # Tras cambiar runtime/gui.py hay que recompilar compilador y programa ST.
        source = GUI_SOURCE
        if source is None:
            source = Path(__file__).with_name('gui.py').read_text('utf-8')
        # Solo pasan variables de sesión gráfica; no se heredan contraseñas de servicios.
        env = {key: value for key, value in os.environ.items() if key in {
            'DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'DBUS_SESSION_BUS_ADDRESS',
            'HOME', 'LANG', 'XAUTHORITY'}}
        # Modo de pruebas sin pantalla: únicamente plataformas Qt conocidas.
        if os.environ.get('QT_QPA_PLATFORM') in {'offscreen', 'minimal'}:
            env['QT_QPA_PLATFORM'] = os.environ['QT_QPA_PLATFORM']
        process = subprocess.Popen([sys.executable, '-I', '-c', source],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.DEVNULL, env=env, start_new_session=True)
        session = GuiSession(process)
        try:
            process.stdin.write(payload)
            process.stdin.flush()
            return self.add('gui', session)
        except Exception:
            session.close()
            raise

    def gui_event(self, handle):
        """Consulta eventos sin esperar; ST puede dormir brevemente entre consultas."""
        self.require('gui')
        return self.resource(handle, {'gui'}).event()

    def gui_text(self, handle, name, text):
        """Encola una modificación; los errores del control llegan como eventos GUI."""
        self.require('gui')
        self.resource(handle, {'gui'}).send({'operacion': 'texto', 'id': name, 'texto': text})
        return True

    def dispatch(self, request):
        """Valida sobre cerrado y convierte excepciones en errores públicos redactados."""
        operation = 'desconocida'
        try:
            if not isinstance(request, dict) or set(request) != {'operacion', 'argumentos'}:
                fail('PROTOCOLO', 'Se requieren operacion y argumentos')
            candidate = request['operacion']
            arguments = request['argumentos']
            if not isinstance(candidate, str) or candidate not in self.operations:
                fail('OPERACION', 'Operación no implementada; consulta docs/INTEGRACIONES.md')
            operation = candidate
            if not isinstance(arguments, list) or len(arguments) > 8:
                fail('ARGUMENTO', 'Se requiere un arreglo de hasta ocho argumentos')
            value = self.operations[operation](*arguments)
            response = {'ok': True, 'resultado': value}
            if len(json.dumps(response, ensure_ascii=True).encode()) > MAX_FRAME - 1:
                fail('LIMITE', 'Respuesta mayor de 4095 bytes; divide los datos')
            return response
        except IntegrationError as error:
            return {'ok': False, 'error': f'{error.code}: {error}', 'operacion': operation}
        except Exception as error:
            # No imprime str(error): bibliotecas externas pueden incluir secretos o SQL.
            return {'ok': False, 'error': 'OPERACION_FALLIDA: ' + type(error).__name__ +
                    '; revisa argumentos, permisos, dependencia y recurso', 'operacion': operation}


def serve():
    """Lee tramas locales acotadas hasta EOF; stdout queda reservado al protocolo."""
    runtime = None
    try:
        try:
            runtime = Runtime()
        except Exception:
            startup = {'ok': False, 'error': 'CONFIGURACION: revisa PyForge.toml y sus permisos'}
        while True:
            line = sys.stdin.buffer.readline(MAX_FRAME + 1)
            if not line:
                break
            if len(line) > MAX_FRAME or not line.endswith(b'\n'):
                break  # no continúa un canal desincronizado
            try:
                request = json.loads(line)
                response = runtime.dispatch(request) if runtime else startup
            except Exception:
                response = {'ok': False, 'error': 'PROTOCOLO: JSON inválido'}
            sys.stdout.write(json.dumps(response, ensure_ascii=True, separators=(',', ':')) + '\n')
            sys.stdout.flush()
    finally:
        if runtime:
            runtime.close()


if __name__ == '__main__':
    serve()
