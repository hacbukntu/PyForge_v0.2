"""Registra el modelo mínimo para el administrador del proyecto."""
from django.contrib import admin
from .models import SavedEntry

admin.site.register([SavedEntry])
