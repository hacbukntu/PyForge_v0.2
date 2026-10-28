# Web ST sencilla con SQLite

`pyforge web` sirve la vista `/vista-st/`, capturas privadas por sesión en `/notas/`
y la culebrita nativa en `/juego/`. No se generan modelos empresariales ni se exige
login para los ejemplos. El foro oficial permanece en `website/backend`.

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r web/requirements.txt
export PYFORGE_WEB_PYTHON="$PWD/.venv/bin/python"
pyforge web
```

Edita `src/vistas.st` para la respuesta, `src/snake.st` para el juego y
`web/templates`/`web/static` para la presentación. Django conserva sesiones y
capturas en SQLite. Usa migraciones para actualizar bases; nunca borres la base
para resolver un cambio de modelo. Las sesiones anónimas no son cuentas recuperables.

El script del juego es externo, compatible con CSP. Los cambios usan POST+CSRF.
El servidor de desarrollo es local: consulta `docs/README.md` en el
repositorio antes de desplegar con HTTPS y un servidor WSGI.
