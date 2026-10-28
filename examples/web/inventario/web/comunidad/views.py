"""Vistas HTTP: autenticación, CSRF, límites y propietario preceden a cualquier escritura."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from django.conf import settings
from django.contrib.auth import login
from django.contrib.auth.decorators import login_required
from django.contrib.auth.forms import UserCreationForm
from django.contrib.auth.views import LoginView
from django.core.paginator import Paginator
from django.http import HttpResponse
from django.shortcuts import get_object_or_404, redirect, render
from django.views.decorators.http import require_http_methods
from .forms import NoteForm, ProductForm, AppointmentForm
from .models import Note, Product, Appointment
from security import limited


class SignIn(LoginView):
    """El framework verifica hashes y rota la sesión al entrar; cuota antes del hash."""
    template_name = 'form.html'

    def post(self, request, *args, **kwargs):
        if limited(request, 'login', 8):
            return HttpResponse('Espera un minuto antes de volver a intentarlo.', status=429)
        return super().post(request, *args, **kwargs)


@require_http_methods(['GET','POST'])
def register(request):
    """Registro con validadores Django; no expone correo ni contraseñas de usuarios."""
    if request.method == 'POST' and limited(request, 'register', 4):
        return HttpResponse('Demasiados intentos.', status=429)
    form = UserCreationForm(request.POST or None)
    if request.method == 'POST' and form.is_valid():
        user = form.save()
        login(request, user)
        return redirect('notes')
    return render(request, 'form.html', {'form':form, 'title':'Crear cuenta'})


@login_required
@require_http_methods(['GET','POST'])
def notes(request):
    """Ejemplo SQL privado; la sesión determina el propietario, nunca un campo del cliente."""
    form = NoteForm(request.POST or None)
    if request.method == 'POST':
        if limited(request, 'notes', 10):
            return HttpResponse('Demasiadas peticiones.',status=429)
        if form.is_valid():
            if Note.objects.filter(owner=request.user).count() >= 100:
                form.add_error(None,'Máximo 100 notas por usuario.')
            else:
                Note.objects.create(owner=request.user, text=form.cleaned_data['text'])
                return redirect('notes')
    return render(request,'notes.html',{'form':form,'notes':Note.objects.filter(owner=request.user).order_by('-pk')[:100]})


@login_required
@require_http_methods(['GET','POST'])
def st_view(request):
    """La lógica ST recibe JSON por stdin: no compila código ni recibe órdenes del visitante."""
    answer = ''
    if request.method == 'POST':
        if limited(request, 'st', 10):
            return HttpResponse('Demasiadas peticiones.',status=429)
        name = request.POST.get('nombre','')[:100]
        binary = os.environ.get('ST_WEB_BINARY','')
        if not binary or not Path(binary).is_absolute() or not Path(binary).is_file():
            return HttpResponse('Compila el proyecto y configura ST_WEB_BINARY.',status=503)
        try:
            with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
                result = subprocess.run([sys.executable,'-I',str(settings.BASE_DIR/'st_worker.py'),binary], input=json.dumps({'nombre':name}).encode()+b'\n', stdout=output, stderr=errors, timeout=4, cwd=settings.BASE_DIR.parent)
                output.seek(0)
                if result.returncode:
                    raise ValueError('ST falló')
                answer = str(json.loads(output.read(65536))['mensaje'])[:1000]
        except (OSError, ValueError, KeyError, subprocess.TimeoutExpired):
            return HttpResponse('No se pudo completar la operación ST.',status=502)
    return render(request,'st.html',{'answer':answer})



@login_required
@require_http_methods(['GET', 'POST'])
def records(request, kind):
    """Selecciona un modelo fijo, valida el formulario y filtra siempre por sesión."""
    model, form_type, title = {'inventario': (Product, ProductForm, 'Inventario'), 'agenda': (Appointment, AppointmentForm, 'Agenda')}[kind]
    form = form_type(request.POST or None)
    if request.method == 'POST':
        if limited(request, kind, 10):
            return HttpResponse('Demasiadas peticiones.', status=429)
        if form.is_valid():
            if model.objects.filter(owner=request.user).count() >= 100:
                form.add_error(None, 'Máximo 100 registros por usuario.')
            else:
                model.objects.create(owner=request.user, **form.cleaned_data)
                return redirect(kind)
    return render(request, 'pages.html', {'title': title, 'form': form, 'records': model.objects.filter(owner=request.user).order_by('-pk')[:100]})
