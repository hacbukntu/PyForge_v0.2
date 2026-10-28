"""Registra los tres modelos para el administrador creado expresamente por el propietario."""
from django.contrib import admin
from .models import Note, Product, Appointment
admin.site.register([Note, Product, Appointment])
