#!/usr/bin/env python3
"""Servidor SFTP de laboratorio: una conexión, loopback y raíz temporal explícita.

Solo lo lanza el verificador. Genera credenciales efímeras y comunica la contraseña
por un pipe privado; no escribe claves privadas ni contraseñas en el repositorio.
"""
import json
import os
from pathlib import Path
import secrets
import socket
import sys
import time
import paramiko


def main(root):
    """Sirve únicamente nombres simples y termina al cerrar el cliente o vencer el plazo."""
    directory = root / 'servidor-remoto'; directory.mkdir()
    password = secrets.token_urlsafe(24)
    key = paramiko.RSAKey.generate(2048)

    class Authentication(paramiko.ServerInterface):
        """Autoriza una cuenta efímera solo para canales de sesión SFTP."""
        def check_auth_password(self, username, supplied):
            """Comprueba la cuenta efímera sin registrar la contraseña."""
            return paramiko.AUTH_SUCCESSFUL if username == 'prueba' and secrets.compare_digest(supplied, password) else paramiko.AUTH_FAILED
        def get_allowed_auths(self, username):
            """Anuncia el único mecanismo habilitado en este laboratorio."""
            return 'password'
        def check_channel_request(self, kind, channel_id):
            """Limita la conexión al canal necesario para SFTP."""
            return paramiko.OPEN_SUCCEEDED if kind == 'session' else paramiko.OPEN_FAILED_ADMINISTRATIVELY_PROHIBITED

    class Files(paramiko.SFTPServerInterface):
        """Expone una carpeta temporal sin ascensos ni enlaces remotos."""
        def list_folder(self, path):
            """Devuelve la lista inicial vacía que consulta el cliente de prueba."""
            return []
        def open(self, name, flags, attributes):
            """Abre un nombre simple sin seguir enlaces y conserva creación exclusiva."""
            if '/' in name or name in {'', '.', '..'}:
                return paramiko.SFTP_PERMISSION_DENIED
            try:
                descriptor = os.open(directory / name, flags | os.O_NOFOLLOW, 0o600)
                stream = os.fdopen(descriptor, 'wb' if flags & os.O_WRONLY else 'rb')
                handle = paramiko.SFTPHandle(flags)
                if flags & os.O_WRONLY: handle.writefile = stream
                else: handle.readfile = stream
                return handle
            except OSError as error:
                return paramiko.SFTPServer.convert_errno(error.errno)

    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0)); listener.listen(); listener.settimeout(15)
        port = listener.getsockname()[1]
        (root / 'datos/known_hosts').write_text(f'[localhost]:{port} {key.get_name()} {key.get_base64()}\n')
        print(json.dumps({'port': port, 'password': password}), flush=True)
        client, _ = listener.accept()
        with paramiko.Transport(client) as transport:
            transport.add_server_key(key)
            transport.set_subsystem_handler('sftp', paramiko.SFTPServer, Files)
            transport.start_server(server=Authentication())
            deadline = time.monotonic() + 18
            while transport.is_active() and time.monotonic() < deadline:
                time.sleep(.02)


if __name__ == '__main__':
    main(Path(sys.argv[1]))
