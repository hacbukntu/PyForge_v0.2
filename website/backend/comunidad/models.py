"""Modelo relacional. El ORM parametriza valores; no se concatena SQL con entradas."""
from django.conf import settings
from django.db import models


class Topic(models.Model):
    """Discusión con texto escapado al mostrarlo; ocultable por moderación."""
    author = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    title = models.CharField(max_length=140)
    body = models.TextField(max_length=8000)
    created = models.DateTimeField(auto_now_add=True)
    visible = models.BooleanField(default=True)
    image = models.BinaryField(blank=True, default=bytes)


class Reply(models.Model):
    """Respuesta asociada a un tema y al usuario autenticado real."""
    topic = models.ForeignKey(Topic, on_delete=models.CASCADE, related_name='replies')
    author = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    body = models.TextField(max_length=4000)
    created = models.DateTimeField(auto_now_add=True)
    visible = models.BooleanField(default=True)


class Note(models.Model):
    """Ejemplo SQL privado: todas las consultas filtran por propietario."""
    owner = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    text = models.CharField(max_length=500)


class Quota(models.Model):
    """Contador compartido entre procesos WSGI; no guarda contraseñas ni IP literal."""
    key = models.CharField(max_length=64, primary_key=True)
    minute = models.BigIntegerField()
    count = models.PositiveIntegerField(default=0)
