# Web ST + Django

`src/main.st` y `src/vistas.st` son lógica ST compilada; `web/` usa Django para HTTP,
cuentas, CSRF, sesiones, plantillas y SQL. No es un servidor HTTP escrito íntegramente en ST.

Desde la carpeta del proyecto:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r web/requirements.txt
export PYFORGE_WEB_PYTHON="$PWD/.venv/bin/python"
pyforge web
```

Abre http://127.0.0.1:8000/foro/. Registra una cuenta con contraseña de al menos
12 caracteres. El foro admite texto, respuestas y capturas PNG/JPEG; los miembros
pueden leerlas. Mis notas SQL solo muestra las notas del propietario. Vista ST
consulta el ejecutable nativo; cambia `src/vistas.st` y reinicia `pyforge web`.

`pyforge web --port 8080` cambia el puerto local. `pyforge check/build/run/clean`
siguen disponibles; `run` ejecuta la entrada ST por consola, no el servidor web.

Moderación: con ST_WEB_LOCAL=1, ejecuta `.venv/bin/python web/manage.py createsuperuser`.
No hay claves predeterminadas. El administrador puede ocultar temas/respuestas en
`/administracion/`. No publiques claves ni información personal en capturas.

Producción requiere HTTPS, secreto externo, hosts explícitos, servidor WSGI y
límites del proxy para cuerpos/conexiones. Nunca publiques el directorio completo
del proyecto ni utilices runserver públicamente. Consulta docs/INTEGRACIONES.md en
PyForge. No hay verificación de correo ni recuperación por correo configuradas.
