"""Rutas web del proyecto PyForge: sencillas, sin login obligatorio y con la lógica ST en el centro."""
from django.contrib import admin
from django.urls import path
from comunidad import views

urlpatterns = [
    path('', views.st_view, name='home'),
    path('vista-st/', views.st_view, name='st'),
    path('juego/', views.snake_game, name='snake'),
    path('api/snake/', views.snake_state, name='snake_state'),
    path('notas/', views.dashboard, name='notes'),
    path('administracion/', admin.site.urls),
]
