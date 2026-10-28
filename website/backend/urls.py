"""Rutas explícitas; nunca se sirven fuentes, bases SQL ni carpetas del proyecto."""
from django.contrib import admin
from django.shortcuts import redirect
from django.urls import path, re_path
from comunidad import views
from pathlib import Path
from django.http import FileResponse, Http404
from django.views.decorators.http import require_safe

@require_safe
def public_page(request, name):
    """Entrega únicamente páginas y recursos públicos generados, con lista de extensiones."""
    root = Path(__file__).resolve().parents[2] / 'dist' / 'site'
    target = (root / name).resolve()
    if not target.is_relative_to(root.resolve()) or target.suffix not in {'.html', '.css', '.js', '.svg', '.png', '.ico', '.json', '.st', '.py', '.md', '.toml', '.deb', '.vsix'} or not target.is_file():
        raise Http404()
    return FileResponse(target.open('rb'))

urlpatterns = [path('', public_page, {'name': 'index.html'}), path('foro/', views.topics, name='topics'), path('foro/tema/<int:pk>/', views.topic, name='topic'), path('foro/captura/<int:pk>/', views.screenshot, name='screenshot'), path('administracion/', admin.site.urls)]

# Rutas de la web institucional; nunca se monta el directorio backend como estático.
urlpatterns += [path(name, public_page, {"name": name}) for name in ("index.html", "curso.html", "ejemplos.html", "vocabulario.html", "foro.html", "descargas.html", "style.css", "app.js", "logo.svg", "curso.json")]

urlpatterns += [path('foro.html', lambda request: redirect('topics')), re_path(r'^(?P<name>(?:ejemplos|docs|descargas)/[A-Za-z0-9_./+%-]+)$', public_page)]
