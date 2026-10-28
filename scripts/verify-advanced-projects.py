#!/usr/bin/env python3
"""Verifica los 20 proyectos en copias temporales: resultados, archivos y laboratorios locales.

No instala dependencias ni contacta servidores externos. Informa omisiones y ofrece
--strict para que una omisión haga fallar la ejecución de verificación.
"""
import argparse
import base64
import contextlib
import importlib.util
import json
import os
from pathlib import Path
import secrets
import selectors
import shutil
import socket
import ssl
import subprocess
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'examples/proyectos_avanzados'


def available(python, names):
    """Comprueba paquetes en el intérprete que se incrustará realmente en el programa."""
    code = 'import importlib.util,sys;sys.exit(not all(importlib.util.find_spec(n) for n in sys.argv[1:]))'
    return subprocess.run([python, '-I', '-c', code, *names], capture_output=True).returncode == 0


def port(kind=socket.SOCK_STREAM):
    """Reserva un puerto efímero de loopback para el arranque inmediato del programa ST."""
    with socket.socket(socket.AF_INET, kind) as sock:
        sock.bind(('127.0.0.1', 0))
        return sock.getsockname()[1]


def compile_source(compiler, source, destination, python, optimize):
    """Compila sin shell y con límite temporal; conserva stderr para diagnosticar fallos."""
    args = [str(compiler), 'build', str(source), '-o', str(destination)]
    if not optimize:
        args.append('--no-opt')
    result = subprocess.run(args, env={**os.environ, 'PYFORGE_PYTHON': python}, capture_output=True, text=True, timeout=30)
    if result.returncode:
        raise RuntimeError(result.stderr)


def run_process(binary, root, stdin, env):
    """Ejecuta solo la copia temporal; entrada y salida se mantienen acotadas."""
    # El ejecutable y su auxiliar son nuestros hijos; la sesión propia permite limpiar timeouts.
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen([str(binary)], cwd=root, env=env, stdin=subprocess.PIPE,
                                   stdout=stdout, stderr=stderr, start_new_session=True)
        try:
            process.communicate(stdin.encode(), timeout=20)
            stdout.seek(0); stderr.seek(0)
            output = stdout.read(65537); errors = stderr.read(65537)
            if process.returncode or errors:
                raise RuntimeError(f'Código {process.returncode}: {errors.decode("utf-8", "replace")}')
            if len(output) > 65536:
                raise RuntimeError('La salida excede 64 KiB')
            return output.decode('utf-8')
        finally:
            import signal
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()


def assert_result(project, root, output):
    """Contrasta requisitos del escenario y archivos generados, no solo el código de salida."""
    expected = json.loads((root / project['stdout']).read_text())
    for fragment in expected['contains']:
        if fragment not in output:
            raise AssertionError(f'Falta {fragment!r} en {output!r}')
    for name, requirement in expected['artifacts'].items():
        path = root / name
        if not path.is_file() or path.stat().st_size == 0:
            raise AssertionError('No se generó ' + name)
        if 'equals' in requirement and path.read_text() != requirement['equals']:
            raise AssertionError('Contenido distinto: ' + name)
        if 'prefix' in requirement and not path.read_text().startswith(requirement['prefix']):
            raise AssertionError('Formato incorrecto: ' + name)


def execute(project, root, binaries, python):
    """Selecciona dependencias/laboratorio y cierra servidores aunque el programa falle."""
    mode = project['mode']
    required = {'documentos': ['docx', 'reportlab'], 'gui': ['PySide6'], 'remoto': ['paramiko']}.get(mode, [])
    if project['folder'].startswith('05_'):
        required = ['cryptography']
    if required and not available(python, required):
        return None, 'Faltan dependencias en el intérprete seleccionado: ' + ', '.join(required)
    env = dict(os.environ)
    env['ST_DIARIO_CLAVE'] = base64.b64encode(secrets.token_bytes(32)).decode()
    env['ST_GUI_PRUEBA'] = '1'
    env['QT_QPA_PLATFORM'] = 'offscreen'
    stdin = (root / project['stdin']).read_text()
    errors = []
    thread = None
    listener = None
    server = None
    try:
        if mode == 'tcp':
            env['ST_PORT'] = str(port())
            server = subprocess.Popen([str(binaries['servidor.st'])], cwd=root, env=env,
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            with selectors.DefaultSelector() as selector:
                selector.register(server.stdout, selectors.EVENT_READ)
                if not selector.select(8) or server.stdout.readline() != b'LISTO\n':
                    raise RuntimeError('El servidor ST no quedó preparado')
        elif mode == 'udp':
            listener = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            listener.bind(('127.0.0.1', 0)); listener.settimeout(10)
            env['ST_PORT'] = str(listener.getsockname()[1])
            env['ST_LOCAL_PORT'] = str(port(socket.SOCK_DGRAM))
            def peer():
                """Confirma solo el mensaje de telemetría con esquema y valores esperados."""
                try:
                    data, address = listener.recvfrom(2048)
                    assert json.loads(data) == {'id': 7, 'sensor': 'temperatura', 'valor': 24}
                    listener.sendto(b'{"ack":7}', address)
                except Exception as error:
                    errors.append(error)
            thread = threading.Thread(target=peer, daemon=True); thread.start()
        elif mode == 'tls':
            if not shutil.which('openssl'):
                return None, 'Falta openssl para el certificado temporal del laboratorio'
            (root / 'datos').mkdir(exist_ok=True)
            key = root / 'servidor.key'; cert = root / 'datos/ca.pem'
            subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
                            '-keyout', str(key), '-out', str(cert), '-subj', '/CN=localhost',
                            '-addext', 'subjectAltName=DNS:localhost'], check=True,
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=20)
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); context.load_cert_chain(cert, key)
            listener = socket.socket(); listener.bind(('127.0.0.1', 0)); listener.listen(); listener.settimeout(10)
            env['ST_PORT'] = str(listener.getsockname()[1])
            def peer():
                """Responde HTTP solo después de un handshake TLS real del programa ST."""
                try:
                    client, _ = listener.accept(); client.settimeout(10)
                    with context.wrap_socket(client, server_side=True) as connection:
                        request = bytearray()
                        while not request.endswith(b'\r\n\r\n') and len(request) < 1024:
                            data = connection.recv(128)
                            if not data: break
                            request.extend(data)
                        assert request.startswith(b'GET /estado HTTP/1.1\r\n')
                        connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK')
                except Exception as error:
                    errors.append(error)
            thread = threading.Thread(target=peer, daemon=True); thread.start()
        elif mode == 'remoto':
            # El servidor usa el mismo intérprete seleccionado, en un proceso separado.
            # Así una instalación Paramiko del entorno virtual basta para ambos extremos.
            server = subprocess.Popen([python, str(ROOT / 'scripts/advanced-sftp-lab.py'), str(root)],
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            with selectors.DefaultSelector() as selector:
                selector.register(server.stdout, selectors.EVENT_READ)
                if not selector.select(10): raise RuntimeError('El laboratorio SFTP no respondió')
                ready = json.loads(server.stdout.readline())
            env.update({'ST_PORT': str(ready['port']), 'ST_USER': 'prueba', 'ST_PASSWORD': ready['password']})
        output = run_process(binaries['main.st'], root, stdin, env)
        if thread:
            thread.join(11)
            if thread.is_alive(): raise RuntimeError('Servidor local no terminó')
            if errors: raise RuntimeError(f'Laboratorio {mode}: {type(errors[0]).__name__}')
        if server:
            server.wait(timeout=10)
            if server.returncode: raise RuntimeError('El servidor de laboratorio terminó con error')
        assert_result(project, root, output)
        return output, None
    finally:
        if listener: listener.close()
        if server:
            if server.poll() is None:
                server.terminate()
                try: server.wait(timeout=2)
                except subprocess.TimeoutExpired: server.kill(); server.wait()
            server.stdout.close(); server.stderr.close()


def main():
    """Publica un informe JSON de resultados y nunca instala bibliotecas automáticamente."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=ROOT / 'target/release/pyforgec')
    parser.add_argument('--python', default='/usr/bin/python3')
    parser.add_argument('--project', default='')
    parser.add_argument('--strict', action='store_true')
    parser.add_argument('--no-opt', action='store_true')
    parser.add_argument('--report', type=Path, default=ROOT / 'target/proyectos-avanzados-resultados.json')
    args = parser.parse_args()
    projects = [p for p in json.loads((BASE / 'catalogo.json').read_text()) if p['folder'].startswith(args.project)]
    if not projects: parser.error('No coincide ningún proyecto')
    results = []
    for project in projects:
        item = {'proyecto': project['folder'], 'estado': 'fallo'}
        try:
            with tempfile.TemporaryDirectory(prefix='st-proyecto-avanzado-') as folder:
                root = Path(folder); original = BASE / project['folder']
                shutil.copytree(original / 'src', root / 'src')
                for name in ['PyForge.toml', project['stdin'], project['stdout'], *project['fixtures']]:
                    target = root / name; target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(original / name, target)
                binaries = {}
                for source in sorted((root / 'src').glob('*.st')):
                    binary = root / (source.stem + '.bin')
                    compile_source(args.compiler.resolve(), source, binary, args.python, not args.no_opt)
                    binaries[source.name] = binary
                _, reason = execute(project, root, binaries, args.python)
                item = {'proyecto': project['folder'], 'estado': 'omitido' if reason else 'correcto', 'detalle': reason or 'Compilación, ejecución y comprobaciones correctas'}
        except Exception as error:
            item['detalle'] = str(error)
        results.append(item)
        print(item['proyecto'] + ': ' + item['estado'] + ' — ' + item['detalle'], flush=True)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
    failed = any(r['estado'] == 'fallo' or (args.strict and r['estado'] == 'omitido') for r in results)
    raise SystemExit(1 if failed else 0)


if __name__ == '__main__':
    main()
