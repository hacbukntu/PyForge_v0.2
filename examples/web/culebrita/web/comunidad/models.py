"""Modelo mínimo y seguro para datos persistentes de la web ST.

PyForge ya no genera un conjunto de modelos empresariales por defecto.
El dato que se persiste es una sola entidad de "capturas" del usuario,
reutilizable para notas, paneles, leads o pruebas de prototipos.
"""
from django.conf import settings
from django.db import models


class SavedEntry(models.Model):
    """Entrada persistente local. La aplicación puede funcionar sin autenticación."""
    owner = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.SET_NULL, null=True, blank=True)
    session_key = models.CharField(max_length=64, blank=True, default="", db_index=True)
    title = models.CharField(max_length=120)
    kind = models.CharField(max_length=32, default='general')
    content = models.TextField()
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ['-updated_at']


class Quota(models.Model):
    """Contador SQL de peticiones compartido por los procesos del servidor."""
    key = models.CharField(max_length=64, primary_key=True)
    minute = models.BigIntegerField()
    count = models.PositiveIntegerField(default=0)
