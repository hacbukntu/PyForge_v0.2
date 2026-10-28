"""Ejemplos de negocio; el foro real vive en website/backend."""
from django.contrib import admin
from django.contrib.auth.views import LogoutView
from django.shortcuts import redirect
from django.urls import path
from comunidad import views
urlpatterns = [
    path('', lambda request: redirect('inventario')),
    path('cuenta/entrar/', views.SignIn.as_view(), name='login'),
    path('cuenta/registro/', views.register, name='register'),
    path('cuenta/salir/', LogoutView.as_view(), name='logout'),
    path('notas/', views.notes, name='notes'),
    path('inventario/', views.records, {'kind': 'inventario'}, name='inventario'),
    path('agenda/', views.records, {'kind': 'agenda'}, name='agenda'),
    path('vista-st/', views.st_view, name='st'),
    path('administracion/', admin.site.urls),
]
