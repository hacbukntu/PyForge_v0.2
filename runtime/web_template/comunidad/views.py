"""Vistas HTTP: la app web se mantiene ligera y sin login obligatorio, con la lógica ST como motor principal."""
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from django.conf import settings
from django.http import HttpResponse, JsonResponse
from django.shortcuts import render
from django.views.decorators.http import require_http_methods

from security import limited
from .forms import CaptureForm
from .models import SavedEntry


def run_st_json(payload):
    """Ejecuta el binario ST compilado y devuelve un JSON con la respuesta del proyecto."""
    binary = os.environ.get('ST_WEB_BINARY', '')
    if not binary or not Path(binary).is_absolute() or not Path(binary).is_file():
        raise RuntimeError('Compila el proyecto y configura ST_WEB_BINARY.')
    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
        result = subprocess.run(
            [sys.executable, '-I', str(settings.BASE_DIR / 'st_worker.py'), binary],
            input=json.dumps(payload).encode() + b'\n',
            stdout=output,
            stderr=errors,
            timeout=4,
            cwd=settings.BASE_DIR.parent,
        )
        output.seek(0)
        if result.returncode:
            raise ValueError('ST falló')
        data = json.loads(output.read(65536))
        return data


@require_http_methods(['GET', 'POST'])
def dashboard(request):
    """Panel de capturas y notas sin login obligatorio."""
    form = CaptureForm(request.POST or None)
    if not request.session.session_key:
        request.session.create()
    scope = {'owner': request.user} if request.user.is_authenticated else {'owner__isnull': True, 'session_key': request.session.session_key}
    entries = SavedEntry.objects.filter(**scope).order_by('-updated_at')[:100]
    if request.method == 'POST':
        if limited(request, 'notes', 10):
            return HttpResponse('Demasiadas peticiones.', status=429)
        if form.is_valid():
            SavedEntry.objects.create(
                owner=request.user if hasattr(request, 'user') and getattr(request.user, 'is_authenticated', False) else None,
                session_key=request.session.session_key,
                title=form.cleaned_data['title'],
                kind=form.cleaned_data['kind'],
                content=form.cleaned_data['content'],
            )
            return render(request, 'notes.html', {'form': form, 'notes': entries, 'entries': entries, 'title': 'Capturas'})
    return render(request, 'notes.html', {'form': form, 'notes': entries, 'entries': entries, 'title': 'Capturas'})


@require_http_methods(['GET', 'POST'])
def st_view(request):
    """Consulta la lógica ST sin requerir autenticación ni rutas de cuenta."""
    answer = ''
    if request.method == 'POST':
        if limited(request, 'st', 10):
            return HttpResponse('Demasiadas peticiones.', status=429)
        name = request.POST.get('nombre', '')[:100]
        try:
            payload = {'nombre': name, 'tipo': 'respuesta'}
            response = run_st_json(payload)
            answer = str(response.get('mensaje', ''))[:1000]
        except (OSError, ValueError, KeyError, TypeError, RuntimeError, subprocess.TimeoutExpired):
            return HttpResponse('No se pudo completar la operación ST.', status=502)
    return render(request, 'st.html', {'answer': answer, 'title': 'Vista ST'})


@require_http_methods(['GET'])
def snake_game(request):
    """Página del juego con el estado inicial servido por ST, sin login."""
    return render(request, 'snake.html', {'title': 'Culebrita ST'})


@require_http_methods(['POST'])
def snake_state(request):
    """Avanza exclusivamente con el estado de la sesión; CSRF protege cada paso."""
    if limited(request, 'snake', 300):
        return JsonResponse({'error': 'Demasiados pasos; espera un minuto.'}, status=429)
    try:
        data = json.loads(request.body)
        action = data.get('accion')
        if action not in {'reiniciar', 'paso'}:
            return JsonResponse({'error': 'Acción inválida'}, status=400)
        state = request.session.get('snake')
        if not state and action == 'paso':
            return JsonResponse({'error': 'Primero inicia la partida.'}, status=409)
        direction = data.get('direccion', 0)
        if type(direction) is not int or direction not in range(4):
            return JsonResponse({'error': 'Dirección inválida'}, status=400)
        response = run_st_json({'tipo': 'snake', 'accion': action, 'estado': state, 'direccion': direction})
        if not isinstance(response, dict) or not isinstance(response.get('cuerpo'), list):
            raise ValueError('Contrato ST inválido')
        request.session['snake'] = response
        return JsonResponse(response)
    except (OSError, ValueError, KeyError, TypeError, AttributeError, RuntimeError, subprocess.TimeoutExpired):
        return JsonResponse({'error': 'No se pudo completar el paso ST.'}, status=502)


# Compatibilidad de nombres internos; el registro de rutas sigue siendo explícito.
notes = dashboard
records = dashboard
