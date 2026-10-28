# Agenda de citas

Ejemplo independiente: inicia en `/agenda/`.


Esta plantilla contiene **notas privadas**, **inventario** y **agenda**. El foro de
PyForge está separado, en `website/backend` del repositorio.

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r web/requirements.txt
export PYFORGE_WEB_PYTHON="$PWD/.venv/bin/python"
pyforge web
```

Abre http://127.0.0.1:8000/. Crea una cuenta local; después puedes guardar notas,
productos y citas. Cada consulta filtra por el usuario autenticado. No hay cuentas
ni contraseñas predeterminadas. `/vista-st/` ejecuta `src/main.st`, compilado por
PyForge; modifica `src/vistas.st` y reinicia para cambiar la respuesta nativa.

- `web/comunidad/models.py`: define los tres modelos SQL.
- `web/comunidad/forms.py`: valida tamaños, fechas y unidades no negativas.
- `web/comunidad/views.py`: conecta petición, propietario, modelo y plantilla.
- `web/templates/`, `web/static/`: presentación HTML, CSS y JS.
- `src/`: lógica ST; Django Python atiende HTTP. El crate Rust experimental está separado.

Para cambiar modelos existentes crea migraciones con `manage.py makemigrations`,
revísalas y aplica `migrate`; no borres bases de datos para actualizar.
`pyforge web` es local. Producción necesita HTTPS, secreto externo, hosts explícitos,
servidor WSGI y un proxy con límites de conexiones y tamaño. Consulta `docs/INTEGRACIONES.md`.
