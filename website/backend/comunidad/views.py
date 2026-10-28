"""Vistas públicas de la comunidad; solo permiten consultar contenido."""
from django.core.paginator import Paginator
from django.http import HttpResponse
from django.shortcuts import get_object_or_404, render
from django.views.decorators.http import require_safe
from .models import Topic


@require_safe
def topics(request):
    """Lista discusiones públicas sin exponer controles de escritura."""
    page = Paginator(
        Topic.objects.filter(visible=True).select_related('author').defer('image').order_by('-created'),
        20,
    ).get_page(request.GET.get('page'))
    return render(request, 'topics.html', {'page': page})


@require_safe
def topic(request, pk):
    """Muestra un tema visible y sus respuestas publicadas."""
    item = get_object_or_404(Topic.objects.select_related('author').defer('image'), pk=pk, visible=True)
    page = Paginator(item.replies.filter(visible=True).select_related('author').order_by('created'), 30).get_page(request.GET.get('page'))
    return render(request, 'topic.html', {'topic': item, 'page': page, 'has_image': bool(item.image)})


@require_safe
def screenshot(request, pk):
    """Sirve únicamente la imagen PNG recodificada del tema público."""
    item = get_object_or_404(Topic, pk=pk, visible=True)
    if not item.image:
        return HttpResponse(status=404)
    return HttpResponse(bytes(item.image), content_type='image/png')
