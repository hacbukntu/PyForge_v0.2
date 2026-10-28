"""Moderación mediante cuentas staff; no se generan usuarios ni claves predeterminados."""
from django.contrib import admin
from .models import Topic, Reply

@admin.register(Topic)
class TopicAdmin(admin.ModelAdmin):
    list_display = ['title','author','created','visible']
    list_filter = ['visible']
    exclude = ['image']

@admin.register(Reply)
class ReplyAdmin(admin.ModelAdmin):
    list_display = ['topic','author','created','visible']
    list_filter = ['visible']
