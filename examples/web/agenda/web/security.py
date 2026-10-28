"""Cabeceras de navegador y cuota persistente por IP/usuario, sin guardar la IP original."""
import hashlib
import hmac
import time
from django.conf import settings
from django.db import transaction
from django.http import HttpResponse
from comunidad.models import Quota


class Headers:
    """Prohíbe scripts inline, enmarcado y carga de recursos de terceros."""
    def __init__(self, get_response):
        self.get_response = get_response

    def __call__(self, request):
        try:
            size = int(request.META.get('CONTENT_LENGTH') or 0)
        except (ValueError, TypeError):
            return HttpResponse('Petición inválida.',status=400)
        if size > 3 * 1024 * 1024:
            return HttpResponse('Petición demasiado grande.', status=413)
        if request.method == 'POST' and request.path == '/administracion/login/' and limited(request,'admin-login',8):
            return HttpResponse('Espera antes de volver a intentarlo.',status=429)
        response = self.get_response(request)
        response['Content-Security-Policy'] = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
        response['Permissions-Policy'] = 'camera=(), microphone=(), geolocation=()'
        response['Cache-Control'] = 'no-store'
        return response


def limited(request, scope, maximum=10):
    """Limita por minuto con SQL transaccional; el proxy debe limitar también tráfico total."""
    address = request.META.get('REMOTE_ADDR', '')  # No confiar en X-Forwarded-For del cliente.
    if getattr(settings,'ST_TRUST_PROXY',False):
        # Solo cuando el proxy elimina el valor del visitante y establece X-Real-IP.
        import ipaddress
        try:
            address = str(ipaddress.ip_address(request.META.get('HTTP_X_REAL_IP','')))
        except ValueError:
            pass
    identity = str(request.user.pk) if getattr(request, 'user', None) and request.user.is_authenticated else address
    digest = hmac.new(settings.SECRET_KEY.encode(), (scope + ':' + identity).encode(), hashlib.sha256).hexdigest()
    minute = int(time.time() // 60)
    with transaction.atomic():
        row, _ = Quota.objects.get_or_create(key=digest, defaults={'minute':minute, 'count':0})
        row = Quota.objects.select_for_update().get(pk=row.pk)
        if row.minute != minute:
            row.minute, row.count = minute, 0
        row.count += 1
        row.save(update_fields=['minute','count'])
        Quota.objects.filter(minute__lt=minute-60).delete()
        return row.count > maximum
