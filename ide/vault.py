"""Proyecto cifrado en reposo: AES-256-GCM, scrypt y copia de trabajo exclusivamente en tmpfs.

La memoria puede acabar en swap: para proteger un equipo perdido, usa también
cifrado de disco/swap. Un proceso del mismo usuario puede leer una sesión abierta.
"""
import base64
import hmac
import json
import os
from pathlib import Path
import secrets
import tempfile
import threading
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.scrypt import Scrypt
from cryptography.exceptions import InvalidTag

MAGIC = b'STVAULT1'
MAXIMUM = 32 * 1024 * 1024


def derive(password, salt):
    """Deriva una clave; no guarda ni devuelve la contraseña original."""
    if not isinstance(password,str) or not 12 <= len(password) <= 256:
        raise ValueError('Usa una contraseña de 12 a 256 caracteres.')
    return Scrypt(salt=salt,length=32,n=2**15,r=8,p=1).derive(password.encode('utf-8'))


class Vault:
    """Guarda un único contenedor; cada guardado usa nonce nuevo y reemplazo atómico."""
    def __init__(self, path, password, create=False):
        self.path = Path(path).resolve()
        self.lock = threading.RLock()
        self.temporary = None
        if create:
            if self.path.exists():
                raise ValueError('La bóveda ya existe; no se sobrescribe.')
            self.salt = secrets.token_bytes(16)
            self.key = derive(password,self.salt)
            files = {'src/main.st':base64.b64encode(b'// Proyecto cifrado ST.\nmostrar("Hola desde tu proyecto privado");\n').decode(), 'PyForge.toml':base64.b64encode(b'[package]\nname="privado"\nversion="0.1.0"\n[build]\nentry="src/main.st"\n').decode()}
            # Reserva el destino sin sobrescribir: solo se escribirá texto cifrado.
            with self.path.open('xb') as file:
                os.chmod(self.path,0o600)
                file.write(self.encode(files))
        else:
            if self.path.stat().st_size > MAXIMUM * 2:
                raise ValueError('Bóveda demasiado grande.')
            data = self.path.read_bytes()
            if not data.startswith(MAGIC) or len(data) < 52:
                raise ValueError('Formato de bóveda inválido.')
            self.salt = data[8:24]
            self.key = derive(password,self.salt)
            try:
                files = json.loads(AESGCM(self.key).decrypt(data[24:36],data[36:],MAGIC+self.salt))
            except (InvalidTag, ValueError):
                raise ValueError('Contraseña incorrecta o archivo alterado.') from None
        # Sin fallback a /tmp: los scripts abiertos no deben escribirse en un disco normal.
        if not Path('/dev/shm').is_dir():
            raise ValueError('El modo cifrado requiere /dev/shm (Linux/WSL).')
        self.temporary = tempfile.TemporaryDirectory(prefix='st-vault-',dir='/dev/shm')
        self.root = Path(self.temporary.name)
        try:
            if not isinstance(files,dict) or len(files) > 2000:
                raise ValueError('Contenido de bóveda inválido.')
            total = 0
            for name, value in files.items():
                relative = Path(name)
                if relative.is_absolute() or '..' in relative.parts or not relative.parts:
                    raise ValueError('Ruta inválida en bóveda.')
                data = base64.b64decode(value,validate=True)
                total += len(data)
                if total > MAXIMUM:
                    raise ValueError('Contenido de bóveda demasiado grande.')
                target = self.root/relative
                target.parent.mkdir(parents=True,exist_ok=True)
                target.write_bytes(data)
        except Exception:
            self.close()
            raise

    def encode(self, files):
        """Autentica formato y sal; el nonce aleatorio nunca se reutiliza intencionadamente."""
        nonce = secrets.token_bytes(12)
        raw = json.dumps(files,separators=(',',':')).encode()
        return MAGIC+self.salt+nonce+AESGCM(self.key).encrypt(nonce,raw,MAGIC+self.salt)

    def verify(self, password):
        """Compara claves derivadas en tiempo constante para desbloquear el navegador."""
        try:
            key = derive(password,self.salt)
            if self.key is not None:
                return hmac.compare_digest(key,self.key)
            if self.path.stat().st_size > MAXIMUM * 2:
                return False
            data = self.path.read_bytes()
            AESGCM(key).decrypt(data[24:36],data[36:],MAGIC+self.salt)
            return True
        except (ValueError,InvalidTag,OSError):
            return False

    def save(self):
        """Excluye artefactos de compilación; rechaza enlaces en lugar de seguirlos."""
        with self.lock:
            files, total = {}, 0
            for parent, dirs, names in os.walk(self.root,followlinks=False):
                dirs[:] = [d for d in dirs if not d.startswith('st-ide-') and d not in {'target','dist','.git','.venv','__pycache__','.st-ide-trash','.build'}]
                for name in names:
                    path = Path(parent)/name
                    if path.is_symlink() or not path.is_file():
                        raise ValueError('La bóveda no admite enlaces ni archivos especiales.')
                    size = path.stat().st_size
                    total += size
                    if total > MAXIMUM or len(files) >= 2000:
                        raise ValueError('Límite de bóveda: 32 MB y 2000 archivos.')
                    files[str(path.relative_to(self.root))] = base64.b64encode(path.read_bytes()).decode()
            raw = self.encode(files)
            with tempfile.NamedTemporaryFile(dir=self.path.parent,prefix='.st-vault-',delete=False) as output:
                temporary = Path(output.name)
                output.write(raw); output.flush(); os.fsync(output.fileno())
            try:
                temporary.replace(self.path)
            finally:
                temporary.unlink(missing_ok=True)

    def close(self):
        """Elimina la copia temporal de RAM. No promete borrado físico ni protección de swap."""
        if self.temporary:
            self.temporary.cleanup()
            self.temporary = None
            self.key = None
