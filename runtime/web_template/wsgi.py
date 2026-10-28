"""Entrada de producción para un servidor WSGI detrás de HTTPS."""
import os
os.environ.setdefault('DJANGO_SETTINGS_MODULE','settings')
from django.core.wsgi import get_wsgi_application
application = get_wsgi_application()
