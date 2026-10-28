"""Pruebas aisladas de permisos, datos, procesos y criptografía; sin red externa."""
import base64
import importlib.util
import json
from pathlib import Path
import socket
import sys
import tempfile
import threading
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('st_integraciones', ROOT / 'runtime/integraciones.py')
bridge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)


class Integrations(unittest.TestCase):
    """Cada caso obtiene recursos temporales; nunca toca documentos del usuario."""
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.runtime = bridge.Runtime(self.root, {'permisos': {p: True for p in bridge.PERMISSIONS},
            'integraciones': {'hosts': ['127.0.0.1'], 'programas': ['/usr/bin/printf', '/usr/bin/sleep', '/usr/bin/yes']}})

    def tearDown(self):
        self.runtime.close()
        self.temp.cleanup()

    def request(self, operation, *args):
        return self.runtime.dispatch({'operacion': operation, 'argumentos': list(args)})

    def test_default_deny(self):
        denied = bridge.Runtime(self.root, {})
        self.assertFalse(denied.dispatch({'operacion': 'archivos.leer', 'argumentos': ['a']})['ok'])
        self.assertTrue(denied.dispatch({'operacion': 'api.informacion', 'argumentos': []})['ok'])

    def test_manifest_rejects_unknown_and_non_boolean(self):
        for config in [{'permisos': {'root': True}}, {'permisos': {'red': 'false'}},
                       {'integraciones': {'hosts': '*'}}, {'integraciones': {'shell': True}}]:
            with self.assertRaises(bridge.IntegrationError):
                bridge.Runtime(self.root, config)

    def test_file_roundtrip_overwrite_and_traversal(self):
        self.assertTrue(self.request('archivos.escribir', 'texto.txt', 'Español ñ')['ok'])
        self.assertFalse(self.request('archivos.escribir', 'texto.txt', 'destruir')['ok'])
        self.assertEqual(self.request('archivos.leer', 'texto.txt')['resultado'], 'Español ñ')
        for path in ['../fuera', '/tmp/fuera', '.', '.git/config', 'a\x00b']:
            self.assertFalse(self.request('archivos.escribir', path, 'x')['ok'], path)
        (self.root / 'enlace').symlink_to('/tmp', target_is_directory=True)
        self.assertFalse(self.request('archivos.leer', 'enlace/archivo')['ok'])

    def test_delete_requires_confirmation_and_never_recursive(self):
        self.request('archivos.escribir', 'texto.txt', 'x')
        self.assertFalse(self.request('archivos.eliminar', 'texto.txt')['ok'])
        self.assertTrue(self.request('archivos.eliminar', 'texto.txt', True)['ok'])
        self.request('archivos.crear_carpeta', 'datos')
        self.request('archivos.escribir', 'datos/a', 'x')
        self.assertFalse(self.request('archivos.eliminar', 'datos', True)['ok'])

    def test_sql_parameters_transactions_and_attach(self):
        handle = self.request('db.abrir', 'prueba.db')['resultado']
        self.assertTrue(self.request('db.ejecutar', handle, 'CREATE TABLE datos(nombre TEXT)', [])['ok'])
        payload = "Robert'); DROP TABLE datos;--"
        self.assertTrue(self.request('db.ejecutar', handle, 'INSERT INTO datos VALUES (?)', [payload])['ok'])
        self.assertEqual(self.request('db.consultar', handle, 'SELECT nombre FROM datos', [])['resultado'], [[payload]])
        self.request('db.revertir', handle)
        self.assertEqual(self.request('db.consultar', handle, 'SELECT nombre FROM datos', [])['resultado'], [])
        self.assertFalse(self.request('db.ejecutar', handle, "ATTACH DATABASE '/tmp/fuera.db' AS fuera", [])['ok'])
        self.request('db.cerrar', handle)
        self.assertFalse(self.request('db.confirmar', handle)['ok'])

    def test_crypto_authentication_and_unique_nonce(self):
        if importlib.util.find_spec('cryptography') is None:
            self.skipTest('cryptography ausente: no se afirma verificar AES-GCM')
        key = self.request('cifrado.clave')['resultado']
        first = self.request('cifrado.cifrar', 'secreto ñ', key, 'contexto')['resultado']
        second = self.request('cifrado.cifrar', 'secreto ñ', key, 'contexto')['resultado']
        self.assertNotEqual(first, second)
        self.assertEqual(self.request('cifrado.descifrar', first, key, 'contexto')['resultado'], 'secreto ñ')
        self.assertFalse(self.request('cifrado.descifrar', first, key, 'otro')['ok'])
        self.assertFalse(self.request('cifrado.descifrar', first, self.request('cifrado.clave')['resultado'], 'contexto')['ok'])
        raw = bytearray(base64.b64decode(first[4:])); raw[-1] ^= 1
        response = self.request('cifrado.descifrar', 'ST1.' + base64.b64encode(raw).decode(), key, 'contexto')
        self.assertFalse(response['ok'])
        self.assertNotIn(key, json.dumps(response))
        self.assertNotIn('secreto ñ', json.dumps(response))

    def test_process_no_shell_timeout_and_output_limit(self):
        result = self.request('comandos.ejecutar', '/usr/bin/printf', ['%s', '$(touch malicioso); echo x'], 3)
        self.assertEqual(result['resultado']['stdout'], '$(touch malicioso); echo x')
        self.assertFalse((self.root / 'malicioso').exists())
        self.assertFalse(self.request('comandos.ejecutar', '/bin/sh', ['-c', 'echo x'], 3)['ok'])
        self.assertFalse(self.request('comandos.ejecutar', '/usr/bin/printf', ['a\x00b'], 3)['ok'])
        self.assertFalse(self.request('comandos.ejecutar', '/usr/bin/sleep', ['3'], 1)['ok'])
        self.assertFalse(self.request('comandos.ejecutar', '/usr/bin/yes', [], 2)['ok'])

    def test_tcp_loopback_and_closed_handle(self):
        server = socket.socket(); server.bind(('127.0.0.1', 0)); server.listen(); server.settimeout(5)
        port = server.getsockname()[1]
        errors = []
        def peer():
            try:
                with server:
                    connection, _ = server.accept()
                    with connection:
                        connection.settimeout(5)
                        self.assertEqual(connection.recv(10), b'hola')
                        connection.sendall('sí'.encode())
            except Exception as error:
                errors.append(error)
        thread = threading.Thread(target=peer); thread.start()
        try:
            handle = self.request('red.tcp_conectar', '127.0.0.1', port)['resultado']
            self.request('red.enviar', handle, 'hola')
            self.assertEqual(base64.b64decode(self.request('red.recibir', handle)['resultado']), 'sí'.encode())
            self.request('red.cerrar', handle)
            self.assertFalse(self.request('red.recibir', handle)['ok'])
        finally:
            thread.join(6)
        self.assertFalse(thread.is_alive()); self.assertEqual(errors, [])

    def test_udp_receive(self):
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); sock.bind(('127.0.0.1', 0)); sock.settimeout(3)
        handle = self.runtime.add('socket', sock)
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sender:
            sender.sendto(b'datagrama', sock.getsockname())
        result = self.request('red.udp_recibir', handle)
        self.assertEqual(base64.b64decode(result['resultado']['datos_base64']), b'datagrama')

    def test_document_txt_rtf_and_missing_dependency(self):
        self.assertTrue(self.request('documentos.crear', 'nota.txt', 'Hola')['ok'])
        self.assertTrue(self.request('documentos.reemplazar', 'nota.txt', 'Hola', 'Adiós')['ok'])
        self.assertEqual(self.request('documentos.leer', 'nota.txt')['resultado'], 'Adiós')
        self.assertTrue(self.request('documentos.crear', 'nota.rtf', 'Español ñ 😀\nLínea')['ok'])
        self.assertIn(b'\\u', (self.root / 'nota.rtf').read_bytes())
        self.assertFalse(self.request('documentos.crear', 'antiguo.doc', 'Hola')['ok'])

    def test_protocol_redaction_and_size(self):
        self.assertFalse(self.runtime.dispatch({'operacion': '__import__', 'argumentos': []})['ok'])
        self.assertFalse(self.runtime.dispatch([])['ok'])
        result = self.request('db.consultar', 998, 'secreto', [])
        self.assertNotIn('secreto', json.dumps(result))
        self.request('archivos.escribir', 'grande.txt', 'x' * 5000)
        self.assertFalse(self.request('archivos.leer', 'grande.txt')['ok'])

    def test_password_hash_and_base64(self):
        encoded = self.request('cifrado.hash_clave', 'contraseña temporal')['resultado']
        self.assertTrue(self.request('cifrado.verificar_clave', 'contraseña temporal', encoded)['resultado'])
        self.assertFalse(self.request('cifrado.verificar_clave', 'otra', encoded)['resultado'])
        self.assertFalse(self.request('cifrado.verificar_clave', 'otra', 'STP1.999999999.' + encoded[12:])['ok'])
        self.assertEqual(self.request('cifrado.desde_base64', self.request('cifrado.base64', 'ñ')['resultado'])['resultado'], 'ñ')

    def test_tls_certificate_and_hostname(self):
        import ssl
        import subprocess
        import shutil
        if not shutil.which('openssl'):
            self.skipTest('openssl ausente: no se verifican certificados TLS')
        key, cert = self.root / 'prueba.key', self.root / 'ca.pem'
        subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
                        '-keyout', str(key), '-out', str(cert), '-subj', '/CN=localhost',
                        '-addext', 'subjectAltName=DNS:localhost'], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(cert, key)
        self.runtime.options['hosts'].append('localhost')
        for host, ca, expected in [('localhost', 'ca.pem', True), ('localhost', '', False), ('127.0.0.1', 'ca.pem', False)]:
            listener = socket.socket(); listener.bind(('127.0.0.1', 0)); listener.listen(); listener.settimeout(5)
            port = listener.getsockname()[1]
            def peer():
                with listener:
                    raw, _ = listener.accept()
                    try:
                        with context.wrap_socket(raw, server_side=True) as tls:
                            tls.sendall(b'TLS')
                    except ssl.SSLError:
                        raw.close()  # rechazo esperado del cliente con CA/nombre incorrectos
            thread = threading.Thread(target=peer); thread.start()
            response = self.request('red.tls_conectar', host, port, ca)
            self.assertEqual(response['ok'], expected, response)
            if expected:
                self.assertEqual(base64.b64decode(self.request('red.recibir', response['resultado'])['resultado']), b'TLS')
                self.request('red.cerrar', response['resultado'])
            thread.join(6)
            self.assertFalse(thread.is_alive())

    def test_sql_backup(self):
        handle = self.request('db.abrir', 'origen.db')['resultado']
        self.request('db.ejecutar', handle, 'CREATE TABLE datos(valor INTEGER)', [])
        self.request('db.ejecutar', handle, 'INSERT INTO datos VALUES (?)', [42])
        self.assertFalse(self.request('db.respaldar', handle, 'copia.db')['ok'])
        self.request('db.confirmar', handle)
        self.assertTrue(self.request('db.respaldar', handle, 'copia.db')['ok'])
        other = self.request('db.abrir', 'copia.db')['resultado']
        self.assertEqual(self.request('db.consultar', other, 'SELECT valor FROM datos', [])['resultado'], [[42]])

    def test_vpn_rejects_commands_and_private_keys(self):
        config = 'client\nremote-cert-tls server\nverify-x509-name vpn.ejemplo.com name\ntls-version-min 1.2\n'
        self.request('archivos.escribir', 'ejemplo.ovpn', config)
        result = self.request('vpn.validar', 'ejemplo.ovpn', 'openvpn')
        self.assertTrue(result['ok'])
        self.assertFalse(result['resultado']['activacion_disponible'])
        self.request('archivos.escribir', 'ejemplo.ovpn', config + 'up /tmp/malicioso\n', True)
        self.assertFalse(self.request('vpn.validar', 'ejemplo.ovpn', 'openvpn')['ok'])


    def test_optional_docx_and_pdf(self):
        if any(importlib.util.find_spec(name) is None for name in ['docx', 'pypdf', 'reportlab']):
            self.skipTest('Dependencias DOCX/PDF ausentes: formatos no verificados aquí')
        self.assertTrue(self.request('documentos.crear', 'informe.docx', 'Informe español')['ok'])
        self.assertTrue(self.request('documentos.reemplazar', 'informe.docx', 'Informe', 'Documento')['ok'])
        self.assertEqual(self.request('documentos.leer', 'informe.docx')['resultado'], 'Documento español')
        for name, text in [('uno.pdf', 'Primera pagina'), ('dos.pdf', 'Segunda pagina')]:
            self.assertTrue(self.request('documentos.crear', name, text)['ok'])
            self.assertIn(text, self.request('documentos.leer', name)['resultado'])
        self.assertTrue(self.request('documentos.unir_pdf', ['uno.pdf', 'dos.pdf'], 'unido.pdf')['ok'])
        self.assertTrue(self.request('documentos.extraer_pdf', 'unido.pdf', 'pagina.pdf', 1)['ok'])
        self.assertIn('Segunda pagina', self.request('documentos.leer', 'pagina.pdf')['resultado'])
        self.assertFalse(self.request('documentos.extraer_pdf', 'unido.pdf', 'fuera.pdf', 999)['ok'])
        (self.root / 'corrupto.docx').write_bytes(b'no es docx')
        self.assertFalse(self.request('documentos.leer', 'corrupto.docx')['ok'])

    def test_optional_gui_headless_events_and_close(self):
        import time
        from unittest.mock import patch
        if importlib.util.find_spec('PySide6') is None:
            self.skipTest('PySide6 ausente: GUI no verificada aquí')
        source = (ROOT / 'runtime/gui.py').read_text()
        # Automatiza un clic real de QPushButton; no reemplaza Qt con un mock.
        source = source.replace('    window.show()', '    window.show()\n    QtCore.QTimer.singleShot(100, widgets["boton"].click)')
        with patch.dict('os.environ', {'QT_QPA_PLATFORM': 'offscreen'}), patch.object(bridge, 'GUI_SOURCE', source):
            response = self.request('gui.crear', {'titulo': 'Prueba ST', 'controles': [
                {'tipo': 'etiqueta', 'id': 'saludo', 'texto': 'Antes'},
                {'tipo': 'boton', 'id': 'boton', 'texto': 'Prueba'}]})
        self.assertTrue(response['ok'], response)
        handle = response['resultado']
        events = []
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline and 'click' not in events:
            event = self.request('gui.evento', handle)
            self.assertTrue(event['ok'], event)
            events.append(event['resultado']['evento'])
            time.sleep(0.02)
        self.assertIn('lista', events)
        self.assertIn('click', events)
        self.assertTrue(self.request('gui.texto', handle, 'saludo', 'Después')['ok'])
        self.assertTrue(self.request('gui.cerrar', handle)['ok'])
        self.assertFalse(self.request('gui.evento', handle)['ok'])

    def test_optional_ssh_known_host(self):
        import os
        import time
        from unittest.mock import patch
        if importlib.util.find_spec('paramiko') is None:
            self.skipTest('Paramiko ausente: SSH no verificado aquí')
        import paramiko
        host_key = paramiko.RSAKey.generate(2048)
        class Server(paramiko.ServerInterface):
            def check_auth_password(self, username, password):
                return paramiko.AUTH_SUCCESSFUL if username == 'prueba' and password == 'ficticia-temporal' else paramiko.AUTH_FAILED
            def get_allowed_auths(self, username):
                return 'password'
            def check_channel_request(self, kind, channel_id):
                return paramiko.OPEN_SUCCEEDED if kind == 'session' else paramiko.OPEN_FAILED_ADMINISTRATIVELY_PROHIBITED
        remote_root = self.root / 'servidor'
        remote_root.mkdir()
        (remote_root / 'origen.txt').write_text('archivo remoto de prueba')
        class Files(paramiko.SFTPServerInterface):
            """Servidor de laboratorio: solo nombres simples dentro de su carpeta temporal."""
            def list_folder(self, path):
                result = []
                for file in remote_root.iterdir():
                    item = paramiko.SFTPAttributes.from_stat(file.stat())
                    item.filename = file.name
                    result.append(item)
                return result
            def open(self, path, flags, attributes):
                if '/' in path or path in {'.', '..'}:
                    return paramiko.SFTP_PERMISSION_DENIED
                try:
                    descriptor = os.open(remote_root / path, flags, 0o600)
                    mode = 'wb' if flags & os.O_WRONLY else 'rb'
                    stream = os.fdopen(descriptor, mode)
                    handle = paramiko.SFTPHandle(flags)
                    if mode == 'wb':
                        handle.writefile = stream
                    else:
                        handle.readfile = stream
                    return handle
                except OSError as error:
                    return paramiko.SFTPServer.convert_errno(error.errno)
        listener = socket.socket(); listener.bind(('127.0.0.1', 0)); listener.listen(); listener.settimeout(5)
        port = listener.getsockname()[1]
        known = self.root / 'known_hosts'
        known.write_text(f'[127.0.0.1]:{port} {host_key.get_name()} {host_key.get_base64()}\n')
        self.runtime.options['entorno'] = ['ST_TEST_PASSWORD']
        errors = []
        def peer():
            try:
                with listener:
                    client, _ = listener.accept()
                    with paramiko.Transport(client) as transport:
                        transport.add_server_key(host_key)
                        transport.set_subsystem_handler('sftp', paramiko.SFTPServer, Files)
                        transport.start_server(server=Server())
                        deadline = time.monotonic() + 5
                        while transport.is_active() and time.monotonic() < deadline:
                            time.sleep(.02)
            except Exception as error:
                errors.append(type(error).__name__)
        thread = threading.Thread(target=peer); thread.start()
        try:
            with patch.dict(os.environ, {'ST_TEST_PASSWORD': 'ficticia-temporal'}):
                result = self.request('red.ssh_conectar', '127.0.0.1', port, 'prueba', 'known_hosts', 'ST_TEST_PASSWORD')
            self.assertTrue(result['ok'], result)
            handle = result['resultado']
            self.assertIn('origen.txt', self.request('red.sftp_listar', handle, '.')['resultado'])
            (self.root / 'subida.txt').write_text('subida temporal')
            self.assertTrue(self.request('red.sftp_subir', handle, 'subida.txt', 'nuevo.txt')['ok'])
            self.assertFalse(self.request('red.sftp_subir', handle, 'subida.txt', 'nuevo.txt')['ok'])
            self.assertTrue(self.request('red.sftp_descargar', handle, 'origen.txt', 'descarga.txt')['ok'])
            self.assertEqual((self.root / 'descarga.txt').read_text(), 'archivo remoto de prueba')
            self.request('red.cerrar', handle)
        finally:
            thread.join(6)
        self.assertFalse(thread.is_alive())
        self.assertEqual(errors, [])


if __name__ == '__main__':
    unittest.main()
