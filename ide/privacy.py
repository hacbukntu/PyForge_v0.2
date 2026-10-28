"""Sesiones locales de proyectos cifrados; cookies aleatorias, caducidad y límite de intentos."""
from http.cookies import SimpleCookie, CookieError
import secrets
import threading
import time


class Access:
    """No almacena contraseñas: verifica mediante la clave derivada de la bóveda."""
    def __init__(self, vault):
        self.vault = vault
        self.sessions = {}
        self.failures = []
        self.lock = threading.Lock()

    def valid(self, headers):
        """Solo una cookie HttpOnly del servidor desbloquea páginas y API."""
        cookie = SimpleCookie()
        try:
            cookie.load(headers.get('Cookie',''))
            token = cookie['st-private'].value
        except (KeyError, ValueError, CookieError):
            return False
        with self.lock:
            expiry = self.sessions.get(token,0)
            if expiry < time.monotonic():
                self.sessions.pop(token,None)
                return False
            return True

    def login(self, password):
        """Caduca tras 15 minutos; máximo cinco intentos por minuto entre clientes."""
        with self.lock:
            now = time.monotonic()
            self.failures = [t for t in self.failures if now-t < 60]
            if len(self.failures) >= 5:
                raise ValueError('Demasiados intentos. Espera un minuto.')
            self.failures.append(now)
            if not self.vault.verify(password):
                raise ValueError('Contraseña incorrecta.')
            self.sessions = {t:e for t,e in self.sessions.items() if e>now}
            if len(self.sessions) >= 8:
                self.sessions.clear()
            token = secrets.token_urlsafe(32)
            self.sessions[token] = now+900
            return token

    def lock_all(self):
        """Revoca todas las cookies: conocer un token API anterior ya no basta."""
        with self.lock:
            self.sessions.clear()
