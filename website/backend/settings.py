"""Valores conservadores: producción exige clave externa y nombres de host explícitos."""
import os
from pathlib import Path
BASE_DIR = Path(__file__).resolve().parent
LOCAL = os.environ.get('ST_WEB_LOCAL') == '1'
DEBUG = False
SECRET_KEY = os.environ.get('ST_WEB_SECRET', '')
if not SECRET_KEY:
    if not LOCAL:
        raise RuntimeError('Configura ST_WEB_SECRET fuera del repositorio.')
    import secrets
    SECRET_KEY = secrets.token_urlsafe(48)  # Solo sesión local; se renueva al reiniciar.
ALLOWED_HOSTS = ['127.0.0.1', 'localhost'] if LOCAL else os.environ.get('ST_WEB_HOSTS', '').split(',')
if not LOCAL and (len(SECRET_KEY) < 50 or not all(ALLOWED_HOSTS) or '*' in ALLOWED_HOSTS):
    raise RuntimeError('Producción requiere una clave de al menos 50 caracteres y ST_WEB_HOSTS sin comodines.')
INSTALLED_APPS = ['django.contrib.auth', 'django.contrib.contenttypes', 'django.contrib.sessions', 'django.contrib.messages', 'django.contrib.staticfiles', 'django.contrib.admin', 'comunidad']
MIDDLEWARE = ['django.middleware.security.SecurityMiddleware', 'security.Headers', 'django.contrib.sessions.middleware.SessionMiddleware', 'django.middleware.common.CommonMiddleware', 'django.middleware.csrf.CsrfViewMiddleware', 'django.contrib.auth.middleware.AuthenticationMiddleware', 'django.contrib.messages.middleware.MessageMiddleware', 'django.middleware.clickjacking.XFrameOptionsMiddleware']
ROOT_URLCONF = 'urls'
WSGI_APPLICATION = 'wsgi.application'
DATABASES = {'default': {'ENGINE':'django.db.backends.sqlite3', 'NAME': BASE_DIR / 'data' / 'sitio.sqlite3', 'OPTIONS': {'timeout': 10}}}
# PostgreSQL opcional: instala psycopg, y define ST_DB_* fuera del código.
if os.environ.get('ST_DB_NAME'):
    DATABASES['default'] = {'ENGINE':'django.db.backends.postgresql', 'NAME':os.environ['ST_DB_NAME'], 'USER':os.environ['ST_DB_USER'], 'PASSWORD':os.environ['ST_DB_PASSWORD'], 'HOST':os.environ.get('ST_DB_HOST','127.0.0.1'), 'PORT':os.environ.get('ST_DB_PORT','5432')}
(BASE_DIR / 'data').mkdir(mode=0o700, exist_ok=True)
TEMPLATES = [{'BACKEND':'django.template.backends.django.DjangoTemplates', 'DIRS':[BASE_DIR/'templates'], 'APP_DIRS':True, 'OPTIONS':{'context_processors':['django.template.context_processors.request','django.contrib.auth.context_processors.auth','django.contrib.messages.context_processors.messages']}}]
AUTH_PASSWORD_VALIDATORS = [{'NAME':'django.contrib.auth.password_validation.UserAttributeSimilarityValidator'}, {'NAME':'django.contrib.auth.password_validation.MinimumLengthValidator', 'OPTIONS':{'min_length':12}}, {'NAME':'django.contrib.auth.password_validation.CommonPasswordValidator'}, {'NAME':'django.contrib.auth.password_validation.NumericPasswordValidator'}]
PASSWORD_HASHERS = ['django.contrib.auth.hashers.ScryptPasswordHasher', 'django.contrib.auth.hashers.PBKDF2PasswordHasher']
LANGUAGE_CODE = 'es'
TIME_ZONE = 'UTC'
USE_TZ = True
STATIC_URL = '/static/'
STATICFILES_DIRS = [BASE_DIR / 'static']
STATIC_ROOT = BASE_DIR / 'static-build'
LOGIN_URL = '/administracion/login/'
LOGIN_REDIRECT_URL = '/administracion/'
LOGOUT_REDIRECT_URL = '/administracion/login/'
SESSION_COOKIE_HTTPONLY = True
SESSION_COOKIE_SAMESITE = 'Lax'
SESSION_COOKIE_SECURE = not LOCAL
SESSION_COOKIE_AGE = 3600
CSRF_COOKIE_SECURE = not LOCAL
SECURE_SSL_REDIRECT = not LOCAL
SECURE_HSTS_SECONDS = 31536000 if not LOCAL else 0
SECURE_HSTS_INCLUDE_SUBDOMAINS = not LOCAL
SECURE_HSTS_PRELOAD = not LOCAL
SECURE_CONTENT_TYPE_NOSNIFF = True
SECURE_REFERRER_POLICY = 'same-origin'
X_FRAME_OPTIONS = 'DENY'
DATA_UPLOAD_MAX_MEMORY_SIZE = 3 * 1024 * 1024
FILE_UPLOAD_MAX_MEMORY_SIZE = 2 * 1024 * 1024
DATA_UPLOAD_MAX_NUMBER_FIELDS = 20
DEFAULT_AUTO_FIELD = 'django.db.models.BigAutoField'
# Solo activar detrás de un proxy propio que quite el encabezado recibido del cliente.
ST_TRUST_PROXY = os.environ.get('ST_TRUST_PROXY') == '1'
if ST_TRUST_PROXY:
    SECURE_PROXY_SSL_HEADER = ('HTTP_X_FORWARDED_PROTO', 'https')
