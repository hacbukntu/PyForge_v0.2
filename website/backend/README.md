# Comunidad de la web oficial de PyForge

Este es el backend **real de `/website`**, independiente de `pyforge new --web`.
Sirve la web institucional y las conversaciones públicas del foro en modo de
solo lectura. No hay registro, acceso de miembros, publicaciones ni respuestas;
el inicio de sesión de `/administracion/` se reserva para personal autorizado.
La presentación institucional procede de `dist/site`, generado desde `/website`.

Desde la raíz del repositorio:

```bash
python3 scripts/build-site.py
python3 -m venv target/foro-env
target/foro-env/bin/python -m pip install -r website/backend/requirements.txt
ST_WEB_LOCAL=1 target/foro-env/bin/python website/backend/manage.py migrate
ST_WEB_LOCAL=1 target/foro-env/bin/python website/backend/manage.py runserver 127.0.0.1:8000 --insecure
```

Abre http://127.0.0.1:8000/foro/. El foro es público y de solo lectura. Para
gestionar conversaciones o moderar, el personal autorizado entra en
`/administracion/`:

```bash
ST_WEB_LOCAL=1 target/foro-env/bin/python website/backend/manage.py createsuperuser
```

`--insecure` solo permite servir CSS local con DEBUG desactivado. No utilices
runserver en Internet. Para producción usa gunicorn desde este directorio,
ST_WEB_SECRET aleatorio externo de al menos 50 caracteres, ST_WEB_HOSTS explícitos,
HTTPS y un proxy inverso. No establezcas ST_WEB_LOCAL en producción.

El proxy debe servir exclusivamente los recursos generados de `dist/site` y
`static-build`, pasar `/foro/` y `/administracion/` a gunicorn en loopback y nunca
publicar `backend/`, `.env`, bases SQL ni fuentes del proyecto. Ejecuta collectstatic
con la configuración de producción. Si activas ST_TRUST_PROXY=1, el proxy debe
sobrescribir X-Forwarded-Proto y X-Real-IP, no confiar en los que envíe el visitante.
Los ZIP de la web son estáticos: no incluyen el servicio ni sustituyen su despliegue.

Las consultas usan el ORM. Las capturas existentes se recodifican, se eliminan
sus metadatos y se guardan como datos SQL, nunca como ejecutables. Las
conversaciones y capturas visibles son públicas. La protección no sustituye
actualizaciones, copias de seguridad privadas ni límites del proxy.

Pruebas: `ST_WEB_LOCAL=1 target/foro-env/bin/python website/backend/manage.py test comunidad`.
