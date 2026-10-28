#!/usr/bin/env python3
"""IDE local de ST. Sirve archivos del proyecto y ejecuta PyForge sin shell."""
import argparse
import codecs
from contextlib import nullcontext
import json
import os
from pathlib import Path
import secrets
import signal
import subprocess
import tempfile
import threading
import tomllib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse
from language import vocabulary
import python_language
import database
import start_menu
import manual
import folder_picker
import privacy

ASSETS = Path(__file__).resolve().parent
REPO = ASSETS.parent
EXCLUDED = {'.git', 'target', 'dist', '.venv', 'node_modules', '__pycache__'}


class Workspace:
    """Restringe las operaciones al proyecto abierto y administra una ejecución."""
    def __init__(self, root, compiler):
        self.root = root.resolve()
        self.compiler = compiler.resolve()
        self.token = secrets.token_urlsafe(32)
        self.lock = threading.RLock()
        self.picker_lock = threading.Lock()
        self.process = None
        self.running = False
        self.input_ready = False
        self.cancelled = False
        self.output = ''
        self.initial_file = None
        self.vault = None
        self.worker_thread = None

    def path(self, relative):
        """Resuelve una ruta sin permitir escapes, incluso mediante enlaces simbólicos."""
        path = (self.root / relative).resolve()
        if not path.is_relative_to(self.root):
            raise ValueError('La ruta está fuera del proyecto.')
        return path

    def source_path(self, relative):
        """Crea rutas dentro de src; acepta nombre.st o src/nombre.st sin duplicarlo.

        Rechaza rutas absolutas, ascensos y enlaces para que la creación nunca salga
        de src, incluso si una subcarpeta apunta a otro lugar del proyecto.
        """
        route = Path(relative)
        if route.is_absolute() or '..' in route.parts:
            raise ValueError('Usa una ruta relativa dentro de src.')
        parts = route.parts[1:] if route.parts and route.parts[0] == 'src' else route.parts
        target = self.root / 'src'
        for part in ('', *parts):
            target = target / part
            if target.is_symlink():
                raise ValueError('No se permite crear archivos mediante enlaces simbólicos.')
        return self.path(str(target.relative_to(self.root)))

    def remove(self, relative, original):
        """Retira un archivo a una papelera local, comprobando contenido y ejecución.

        La copia conserva su contenido y nombre en un directorio único. No elimina
        carpetas ni sigue enlaces; evita sobrescribir elementos de la papelera.
        """
        route = Path(relative)
        if route.is_absolute() or '..' in route.parts or any(part.startswith('.') or part in EXCLUDED for part in route.parts):
            raise ValueError('No se permite eliminar esa ruta.')
        candidate = self.root
        for part in route.parts:
            candidate /= part
            if candidate.is_symlink():
                raise ValueError('No se permite eliminar mediante enlaces simbólicos.')
        path = self.path(relative)
        if not path.is_file() or path.suffix not in {'.st', '.py', '.md', '.toml', '.txt', '.rs', '.json', '.html', '.css', '.js'}:
            raise ValueError('Selecciona un archivo editable existente.')
        with self.lock:
            if self.running:
                raise ValueError('Detén la ejecución antes de eliminar archivos.')
            if path.stat().st_size > 1_000_000 or path.read_text() != original:
                raise ValueError('El archivo cambió fuera del IDE. Vuelve a abrirlo antes de eliminar.')
            trash = self.root / '.st-ide-trash'
            if trash.is_symlink():
                raise ValueError('La papelera no puede ser un enlace simbólico.')
            trash.mkdir(exist_ok=True)
            destination = Path(tempfile.mkdtemp(prefix='archivo-', dir=trash)) / path.name
            path.rename(destination)
            return str(destination.relative_to(self.root))

    def tree(self):
        """Enumera archivos editables con un límite para proyectos grandes."""
        result = []
        for parent, dirs, files in os.walk(self.root, followlinks=False):
            dirs[:] = sorted(d for d in dirs if d not in EXCLUDED and not d.startswith('.'))
            for name in sorted(files):
                path = Path(parent) / name
                if path.suffix.lower() in {'.st', '.py', '.md', '.toml', '.txt', '.rs', '.json', '.db', '.html', '.css', '.js'} and not path.is_symlink():
                    result.append(str(path.relative_to(self.root)))
                    if len(result) >= 1500:
                        return result
        return result

    def append(self, text):
        """Conserva el último megabyte de salida para limitar memoria."""
        with self.lock:
            self.output = (self.output + text)[-1_000_000:]

    def start(self, relative, stdin_text=None):
        """Valida una entrada ST/Python e inicia el trabajo sin bloquear HTTP.

        Ambos formatos pasan por pyforgec: la extensión selecciona el analizador
        en driver::compile_file. Python conserva las limitaciones del compilador.
        """
        if self.vault and stdin_text is not None:
            raise ValueError('En proyectos cifrados utiliza la entrada interactiva de la terminal.')
        if stdin_text is not None and (not isinstance(stdin_text, str) or len(stdin_text.encode("utf-8")) > 1_000_000):
            raise ValueError("La entrada del programa debe ser texto de hasta 1 MB.")
        source = self.path(relative)
        if source.suffix not in {'.st', '.py'} or not source.is_file():
            raise ValueError('Selecciona un archivo .st o .py existente.')
        if not self.compiler.is_file():
            raise ValueError('No se encontró pyforgec. Ejecuta cargo build --release o usa --compiler.')
        with self.lock:
            if self.vault and self.vault.temporary is None:
                raise ValueError('El proyecto está bloqueado.')
            if self.running:
                raise ValueError('Ya hay una ejecución activa.')
            self.running = True
            self.cancelled = False
            self.output = f'> Compilando {relative}\n'
        self.worker_thread = threading.Thread(target=self.worker, args=(source, stdin_text), daemon=True)
        self.worker_thread.start()

    def check(self, relative):
        """Analiza el archivo abierto sin construir ni ejecutar un programa.

        La lista de argumentos evita el shell. El límite de tiempo y de salida
        mantiene receptiva la interfaz incluso ante una fuente problemática.
        """
        source = self.path(relative)
        if source.suffix not in {'.st', '.py'} or not source.is_file():
            raise ValueError('Selecciona un archivo .st o .py existente.')
        if not self.compiler.is_file():
            raise ValueError('No se encontró pyforgec.')
        with self.lock:
            if self.running:
                raise ValueError('Detén la ejecución antes de comprobar el archivo.')
        result = subprocess.run([str(self.compiler), 'check', str(source)], cwd=self.root,
                                capture_output=True, text=True, timeout=20)
        output = (result.stdout + result.stderr)[-200_000:]
        return {'ok': result.returncode == 0, 'code': result.returncode,
                'output': output or ('Sin errores.\n' if result.returncode == 0 else 'Comprobación fallida.\n')}

    def command(self, args, input_file=None, interactive=False, pass_fd=None):
        """Ejecuta argumentos directos y recoge salida; crea un grupo para Detener."""
        with self.lock:
            if self.cancelled:
                return -1
            self.process = subprocess.Popen(args, cwd=self.root, stdout=subprocess.PIPE,
                                            stderr=subprocess.STDOUT, stdin=input_file if input_file is not None else subprocess.PIPE if interactive else subprocess.DEVNULL, start_new_session=True, pass_fds=(() if pass_fd is None else (pass_fd,)), env=({**os.environ, 'TMPDIR':str(self.root)} if self.vault else None))
            process = self.process
            self.input_ready = interactive
            if interactive:
                # Las escrituras HTTP nunca deben bloquear si el programa no lee.
                os.set_blocking(process.stdin.fileno(), False)
        try:
            decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
            while chunk := process.stdout.read1(4096):
                self.append(decoder.decode(chunk))
            self.append(decoder.decode(b"", final=True))
            return process.wait()
        finally:
            process.stdout.close()
            with self.lock:
                self.input_ready = False
                if process.stdin and not process.stdin.closed:
                    process.stdin.close()
                self.process = None

    def worker(self, source, stdin_text=None):
        """Compila a un directorio temporal y ejecuta solo si la compilación termina bien."""
        try:
            with tempfile.TemporaryDirectory(prefix='st-ide-', dir=self.root if self.vault else None) as temp:
                binary = Path(temp) / 'programa'
                status = self.command([str(self.compiler), 'build', str(source), '-o', str(binary)])
                if status == 0:
                    with self.lock:
                        cancelled = self.cancelled
                    if not cancelled:
                        self.append('> Ejecutando\n')
                        if stdin_text is None:
                            if self.vault:
                                # /dev/shm puede ser noexec: ejecuta una copia anónima en memoria.
                                descriptor = os.memfd_create('st-programa', os.MFD_CLOEXEC)
                                try:
                                    with os.fdopen(os.dup(descriptor), 'wb') as memory:
                                        memory.write(binary.read_bytes())
                                    status = self.command([f'/proc/self/fd/{descriptor}'], interactive=True, pass_fd=descriptor)
                                finally:
                                    os.close(descriptor)
                            else:
                                status = self.command([str(binary)], interactive=True)
                        else:
                            # Compatibilidad con clientes anteriores que envían entrada preparada.
                            with tempfile.TemporaryFile() as input_file:
                                input_file.write(stdin_text.encode('utf-8'))
                                input_file.seek(0)
                                status = self.command([str(binary)], input_file)
                self.append(f'\n> Proceso terminado (código {status}).\n')
        except Exception as error:
            self.append(f'\n> Error: {error}\n')
        finally:
            with self.lock:
                self.running = False
                if self.vault:
                    try:
                        self.vault.save()
                    except Exception as error:
                        self.append('No se pudo guardar la bóveda: ' + str(error))

    def send_input(self, text="", eof=False):
        """Entrega una línea al programa activo, nunca a un intérprete de comandos.

        Escribe como máximo PIPE_BUF bytes para evitar líneas parciales y bloqueos.
        El usuario puede enviar EOF explícito para terminar una lectura sin más datos.
        """
        if not isinstance(text, str) or not isinstance(eof, bool):
            raise ValueError('La entrada debe ser texto y EOF debe ser booleano.')
        data = (text + '\n').encode('utf-8')
        if len(data) > 4096:
            raise ValueError('Cada envío admite hasta 4095 bytes de texto UTF-8.')
        with self.lock:
            process = self.process
            if not self.input_ready or not process or process.poll() is not None:
                raise ValueError('No hay un programa activo que acepte entrada.')
            try:
                if eof:
                    process.stdin.close()
                    self.input_ready = False
                else:
                    os.write(process.stdin.fileno(), data)
                    self.append(text + '\n')
            except BlockingIOError:
                raise ValueError('El programa aún no ha leído la entrada anterior; espera y reintenta.') from None
            except (BrokenPipeError, OSError):
                self.input_ready = False
                raise ValueError('El programa ha cerrado su entrada.') from None

    def stop(self):
        """Cancela la ejecución y mata el grupo, incluyendo el ensamblador/enlazador."""
        with self.lock:
            self.cancelled = True
            if self.process and self.process.poll() is None:
                try:
                    os.killpg(self.process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass


class Handler(BaseHTTPRequestHandler):
    """API limitada a localhost con token por sesión y sin acceso a rutas arbitrarias."""
    def reply(self, status, value, mime='application/json; charset=utf-8'):
        data = json.dumps(value, ensure_ascii=False).encode() if isinstance(value, (dict, list)) else value
        self.send_response(status)
        self.send_header('Content-Type', mime)
        self.send_header('Content-Length', str(len(data)))
        self.send_header('Cache-Control', 'no-store')
        self.send_header('X-Content-Type-Options', 'nosniff')
        self.send_header('X-Frame-Options', 'DENY')
        self.send_header('Referrer-Policy', 'no-referrer')
        if getattr(self,'session_cookie',None):
            self.send_header('Set-Cookie', self.session_cookie)
        self.end_headers()
        self.wfile.write(data)

    def local(self):
        """Rechaza encabezados Host ajenos para evitar acceso por DNS rebinding."""
        return self.headers.get('Host') == f'127.0.0.1:{self.server.server_port}'

    def do_GET(self):
        if not self.local():
            self.reply(403, {'error': 'Host no permitido.'})
            return
        route = urlparse(self.path).path
        access = getattr(self.server,'access',None)
        if access and not access.valid(self.headers) and route not in {'/style.css','/st.svg','/lock.js'}:
            self.reply(200, (ASSETS/'lock.html').read_bytes(), 'text/html; charset=utf-8')
            return
        if route in {'/', '/inicio.html'}:
            data = (ASSETS / ('inicio.html' if route == '/inicio.html' else 'index.html')).read_text().replace('__TOKEN__', self.server.workspace.token)
            self.reply(200, data.encode(), 'text/html; charset=utf-8')
        elif route == '/manual.md':
            # Sirve el documento vigente como HTML; no abre ni ejecuta ejemplos.
            document = manual.page((REPO / 'docs/MANUAL_ST.md').read_text('utf-8'))
            self.reply(200, document.encode('utf-8'), 'text/html; charset=utf-8')
        elif route == '/manual-fuente.md':
            course = manual.course_source((REPO / 'docs/MANUAL_ST.md').read_text('utf-8'))
            self.reply(200, course.encode('utf-8'), 'text/plain; charset=utf-8')
        elif route in ['/preferences.js', '/lock.js', '/manual.css', '/inicio.js', '/apoyar.html', '/formats.js', '/web_languages.js', '/app.js', '/autopairs.js', '/folding.js', '/search.js', '/style.css', '/st.svg', '/python.svg']:
            types = {'.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml'}
            path = ASSETS / route[1:]
            self.reply(200, path.read_bytes(), types[path.suffix] + '; charset=utf-8')
        else:
            self.reply(404, {'error': 'No encontrado.'})

    def do_POST(self):
        access = getattr(self.server,'access',None)
        if urlparse(self.path).path == '/api/unlock':
            # JSON + comprobación Origin bloquean envíos de formularios desde otro origen.
            origin = self.headers.get('Origin')
            if not self.local() or not access or self.headers.get('Content-Type') != 'application/json' or (origin and origin != f'http://127.0.0.1:{self.server.server_port}'):
                self.reply(403, {'error':'Acceso no permitido.'}); return
            try:
                size = int(self.headers.get('Content-Length',0))
                if not 0 < size < 2048: raise ValueError('Solicitud inválida.')
                password = json.loads(self.rfile.read(size))['password']
                token = access.login(password)
                if access.vault.temporary is None:
                    from vault import Vault
                    protected = Vault(access.vault.path,password)
                    replacement = Workspace(protected.root,self.server.workspace.compiler)
                    replacement.vault = protected
                    replacement.initial_file = 'src/main.st'
                    self.server.workspace = replacement
                    access.vault = protected
                self.session_cookie = f'st-private={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=900'
                self.reply(200, {'ok':True})
            except (ValueError,KeyError,OSError):
                self.reply(403, {'error':'No se pudo desbloquear. Comprueba la contraseña o espera un minuto.'})
            return
        if access and not access.valid(self.headers):
            self.reply(401, {'error':'Proyecto bloqueado. Recarga para desbloquear.'}); return
        if not self.local() or self.headers.get('X-ST-Token') != self.server.workspace.token:
            self.reply(403, {'error': 'Sesión no válida.'})
            return
        # Una operación ya autorizada no debe recrear archivos después de bloquear.
        # Serializa únicamente las bóvedas; el selector normal no congela las consultas.
        workspace = self.server.workspace
        with workspace.lock if workspace.vault else nullcontext():
            latest = getattr(self.server,'access',None)
            if latest and not latest.valid(self.headers):
                self.reply(401, {'error':'Proyecto bloqueado.'}); return
            self.authenticated_post()

    def authenticated_post(self):
        """Despacha operaciones tras comprobar token y, en bóvedas, sesión bajo bloqueo."""
        try:
            size = int(self.headers.get('Content-Length', 0))
            if not 0 < size <= 2_000_000:
                raise ValueError('Petición demasiado grande o vacía.')
            body = json.loads(self.rfile.read(size))
            ws = self.server.workspace
            route = urlparse(self.path).path
            if route == '/api/vault-open':
                if ws.running or ws.vault:
                    raise ValueError('Cierra la sesión protegida o detén el programa antes de abrir una bóveda.')
                password = body.get('password','')
                if not isinstance(password,str) or not 12 <= len(password) <= 256:
                    raise ValueError('Usa una contraseña de 12 a 256 caracteres.')
                if not ws.picker_lock.acquire(blocking=False):
                    raise ValueError('Ya hay un selector abierto.')
                try:
                    selected = folder_picker.choose(ws.root, 'vault-create' if body.get('create') is True else 'vault-open')
                    if selected is None:
                        self.reply(200, {'cancelled':True}); return
                    from vault import Vault
                    protected = Vault(selected,password,body.get('create') is True)
                    with ws.lock:
                        if ws.running or self.server.workspace is not ws:
                            protected.close()
                            raise ValueError('La sesión cambió durante la selección.')
                        replacement = Workspace(protected.root,ws.compiler)
                        replacement.vault = protected
                        replacement.initial_file = 'src/main.st'
                        self.server.workspace = replacement
                        self.server.access = privacy.Access(protected)
                        token = self.server.access.login(password)
                        self.session_cookie = f'st-private={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=900'
                finally:
                    ws.picker_lock.release()
                result = {'ok':True}
            elif route == '/api/lock':
                access = getattr(self.server,'access',None)
                if not access:
                    raise ValueError('El proyecto actual no está cifrado. Crea o abre una bóveda desde Inicio.')
                if ws.running:
                    raise ValueError('Detén el programa antes de bloquear.')
                ws.vault.save()
                access.lock_all()
                ws.vault.close()
                ws.output = ''
                ws.token = secrets.token_urlsafe(32)
                result = {'ok':True}
            elif route == '/api/start-menu':
                result = {**start_menu.load(), 'project': ws.vault.path.name + ' · cifrado' if ws.vault else str(ws.root)}
            elif route in {'/api/project-open', '/api/project-select'}:
                if ws.vault:
                    raise ValueError('Cierra esta aplicación para cambiar de bóveda o volver al modo sin contraseña.')
                # Cambiar de raíz requiere una acción explícita autenticada; no sigue rutas de URLs.
                if route == '/api/project-select':
                    # Solo una petición autenticada abre un diálogo del escritorio local.
                    if ws.running:
                        raise ValueError('Detén el programa antes de cambiar de proyecto.')
                    if not ws.picker_lock.acquire(blocking=False):
                        raise ValueError('Ya hay un selector de carpetas abierto.')
                    try:
                        path = folder_picker.choose(ws.root, body.get('create') is True)
                    finally:
                        ws.picker_lock.release()
                    if path is None:
                        self.reply(200, {'cancelled': True})
                        return
                else:
                    path = Path(body['path']).expanduser()
                if not path.is_absolute() or '..' in path.parts:
                    raise ValueError('Indica una ruta absoluta sin ascensos.')
                with ws.lock:
                    if self.server.workspace is not ws:
                        raise ValueError('El proyecto cambió; recarga el menú.')
                    if ws.running:
                        raise ValueError('Detén el programa antes de cambiar de proyecto.')
                    if body.get('create') is True:
                        if path.exists() or not path.parent.is_dir():
                            raise ValueError('La carpeta nueva no debe existir y su carpeta padre debe existir.')
                        # Reutiliza el generador del compilador, sin shell ni plantillas duplicadas.
                        result = subprocess.run([str(ws.compiler.with_name('pyforge')), 'new', path.name], cwd=path.parent,
                                                capture_output=True, text=True, timeout=15)
                        if result.returncode:
                            raise ValueError(result.stderr or 'No se pudo crear el proyecto.')
                    if not path.is_dir():
                        raise ValueError('Selecciona una carpeta existente.')
                    replacement = Workspace(path, ws.compiler)
                    replacement.initial_file = 'src/main.st' if body.get('create') is True else None
                    self.server.workspace = replacement
                result = {'ok': True}
            elif route == '/api/language':
                result = vocabulary()
            elif route == '/api/python-language':
                result = python_language.vocabulary()
            elif route == '/api/python-highlight':
                # Solo análisis de texto: nunca se ejecuta el contenido del editor.
                source = body['text']
                if not isinstance(source, str) or len(source) > 1_000_000:
                    raise ValueError('Texto Python no válido o demasiado grande.')
                result = python_language.highlight(source)
            elif route == '/api/permissions':
                # Solo muestra capacidades; nunca concede permisos ni revela otras tablas.
                manifest = ws.path('PyForge.toml')
                if manifest.exists() and manifest.stat().st_size > 65536:
                    raise ValueError('Manifiesto demasiado grande para mostrar permisos.')
                config = tomllib.loads(manifest.read_text('utf-8')) if manifest.exists() else {}
                declared = config.get('permisos', {})
                if not isinstance(declared, dict) or any(type(v) is not bool for v in declared.values()):
                    raise ValueError('Los permisos deben ser valores booleanos.')
                result = {'permisos': declared, 'sandbox': False}
            elif route == '/api/tree':
                result = {'name': ws.root.name, 'files': ws.tree(), 'initial_file': ws.initial_file, 'protected': bool(ws.vault)}
            elif route == '/api/read':
                path = ws.path(body['path'])
                if path.suffix.lower() == '.db':
                    result = database.preview(path)
                else:
                    if path.stat().st_size > 1_000_000:
                        raise ValueError('El editor admite archivos de hasta 1 MB.')
                    result = {'text': path.read_text()}
            elif route == '/api/save':
                path = ws.path(body['path'])
                if path.suffix.lower() == '.db':
                    raise ValueError('Las bases SQLite se muestran en modo de solo lectura.')
                if not path.is_file():
                    raise ValueError('El archivo ya no existe.')
                # Detecta modificaciones externas antes de sobrescribirlas.
                if path.read_text() != body['original']:
                    raise ValueError('El archivo cambió fuera del IDE. Copia tus cambios y vuelve a abrirlo.')
                path.write_text(body['text'])
                result = {'ok': True}
            elif route == '/api/create':
                path = ws.source_path(body['path'])
                if path.suffix != '.st':
                    raise ValueError('Los archivos nuevos deben terminar en .st.')
                path.parent.mkdir(parents=True, exist_ok=True)
                words = vocabulary()['keywords']
                show = next((item['word'] for item in words if item['kind'] == 'Mostrar'), None)
                template = f'{show}("Hola desde ST");\n' if show else '// Programa ST\n'
                with path.open('x') as file:
                    file.write(template)
                result = {'ok': True, 'path': str(path.relative_to(ws.root))}
            elif route == '/api/delete':
                result = {'ok': True, 'trash': ws.remove(body['path'], body['original'])}
            elif route == '/api/run':
                ws.start(body['path'], body.get('stdin'))
                result = {'ok': True}
            elif route == '/api/check':
                result = ws.check(body['path'])
            elif route == '/api/input':
                ws.send_input(body.get('text', ''), body.get('eof', False))
                result = {'ok': True}
            elif route == '/api/stop':
                ws.stop()
                result = {'ok': True}
            elif route == '/api/status':
                with ws.lock:
                    result = {'running': ws.running, 'input_ready': ws.input_ready, 'output': ws.output}
            else:
                self.reply(404, {'error': 'No encontrado.'})
                return
            if ws.vault and route in {'/api/save','/api/create','/api/delete'}:
                ws.vault.save()
            self.reply(200, result)
        except (ValueError, KeyError, OSError, subprocess.TimeoutExpired) as error:
            self.reply(400, {'error': str(error)})

    def log_message(self, *_):
        """Mantiene la consola de arranque limpia; los errores se muestran en la interfaz."""


def opening_target(argument):
    """Acepta carpeta o fuente; para un archivo busca su manifiesto más cercano."""
    target = Path(argument).resolve()
    if target.is_dir():
        return target, None
    if not target.is_file() or target.suffix not in {'.st', '.py'}:
        raise ValueError('Selecciona una carpeta existente o un archivo .st/.py.')
    root = next((parent for parent in target.parents if (parent / 'PyForge.toml').is_file()), target.parent)
    return root, target.relative_to(root).as_posix()


def main():
    """Abre un servidor local para el proyecto indicado; Ctrl+C detiene sus procesos."""
    parser = argparse.ArgumentParser(description='ST IDE — editor local de proyectos ST')
    parser.add_argument('project', nargs='?', default=os.environ.get('PYFORGE_PROJECT', str(Path.cwd())))
    parser.add_argument('--open-browser', action='store_true', help='Abre la interfaz en el navegador sin ejecutar el archivo.')
    parser.add_argument('--port', type=int, default=8765)
    parser.add_argument('--compiler', default=str(REPO / 'target/release/pyforgec'))
    args = parser.parse_args()
    try:
        root, initial_file = opening_target(args.project)
    except ValueError as error:
        parser.error(str(error))
    server = ThreadingHTTPServer(('127.0.0.1', args.port), Handler)
    server.workspace = Workspace(root, Path(args.compiler))
    server.workspace.initial_file = initial_file
    # Abrir una fuente conserva el acceso directo; el lanzador sin archivo muestra el menú.
    start_url = f'http://127.0.0.1:{server.server_port}' + ('/inicio.html' if args.open_browser and not initial_file else '/')
    # En WSL el puente Windows abre su navegador; no intenta lanzar uno Linux.
    if args.open_browser and os.environ.get('PYFORGE_WINDOWS_BROWSER') != '1':
        import webbrowser
        opener = threading.Timer(0.2, lambda: webbrowser.open(start_url))
        opener.daemon = True
        opener.start()
    print(f'ST IDE: {start_url}\nProyecto: {root}\nCtrl+C para cerrar.', flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.workspace.stop()
        if server.workspace.worker_thread:
            server.workspace.worker_thread.join()
        if server.workspace.vault and server.workspace.vault.temporary:
            # Guardar antes de limpiar; un error conserva el contenedor anterior.
            try:
                server.workspace.vault.save()
            finally:
                server.workspace.vault.close()
        server.server_close()


if __name__ == '__main__':
    main()
