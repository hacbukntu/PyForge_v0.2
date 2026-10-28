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
from .forms import TopicForm, ReplyForm, NoteForm
from .models import Topic, Reply, Note
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
        return redirect('topics')
    return render(request, 'form.html', {'form':form, 'title':'Crear cuenta'})


@login_required
@require_http_methods(['GET','POST'])
def topics(request):
    """Lista paginada y alta de discusiones: capturas solo para miembros autenticados."""
    form = TopicForm(request.POST or None, request.FILES or None)
    if request.method == 'POST':
        if limited(request, 'topic', 5):
            return HttpResponse('Espera antes de publicar de nuevo.', status=429)
        if form.is_valid():
            item = Topic.objects.create(author=request.user, title=form.cleaned_data['title'], body=form.cleaned_data['body'], image=form.cleaned_data['screenshot'])
            return redirect('topic', pk=item.pk)
    page = Paginator(Topic.objects.filter(visible=True).select_related('author').defer('image').order_by('-created'),20).get_page(request.GET.get('page'))
    return render(request, 'topics.html', {'form':form, 'page':page})


@login_required
@require_http_methods(['GET','POST'])
def topic(request, pk):
    """Ocultar el tema también impide consultar sus respuestas y su captura."""
    item = get_object_or_404(Topic.objects.select_related('author').defer('image'), pk=pk, visible=True)
    form = ReplyForm(request.POST or None)
    if request.method == 'POST':
        if limited(request, 'reply', 10):
            return HttpResponse('Espera antes de responder de nuevo.', status=429)
        if form.is_valid():
            Reply.objects.create(topic=item, author=request.user, body=form.cleaned_data['body'])
            return redirect('topic', pk=pk)
    page = Paginator(item.replies.filter(visible=True).select_related('author').order_by('created'),30).get_page(request.GET.get('page'))
    return render(request, 'topic.html', {'topic':item, 'form':form, 'page':page, 'has_image': bool(item.image)})


@login_required
def screenshot(request, pk):
    """Sirve únicamente PNG recodificado, sin rutas proporcionadas por el usuario."""
    item = get_object_or_404(Topic, pk=pk, visible=True)
    if not item.image:
        return HttpResponse(status=404)
    return HttpResponse(bytes(item.image), content_type='image/png')


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
@require_http_methods(['POST'])
def remove_topic(request, pk):
    """El autor o un moderador retira el tema y borra su captura de la fila SQL."""
    item = get_object_or_404(Topic,pk=pk)
    if request.user.pk != item.author_id and not request.user.is_staff:
        return HttpResponse(status=403)
    item.visible=False
    item.image=b''
    item.save(update_fields=['visible','image'])
    return redirect('topics')
