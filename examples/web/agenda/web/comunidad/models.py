"""Tres modelos de ejemplo independientes del foro institucional."""
from django.conf import settings
from django.db import models

class Note(models.Model):
    """Notas privadas del usuario autenticado."""
    owner = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    text = models.CharField(max_length=500)

class Product(models.Model):
    """Inventario personal; las unidades se validan como no negativas."""
    owner = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    name = models.CharField(max_length=120)
    units = models.PositiveIntegerField(default=0)

class Appointment(models.Model):
    """Agenda personal con fecha y descripción, sin datos de otros usuarios."""
    owner = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    title = models.CharField(max_length=120)
    date = models.DateField()

class Quota(models.Model):
    """Contador SQL de peticiones compartido por los procesos del servidor."""
    key = models.CharField(max_length=64, primary_key=True)
    minute = models.BigIntegerField()
    count = models.PositiveIntegerField(default=0)
