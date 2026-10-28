# Culebrita: ST nativo + Django

Desde esta carpeta:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install -r web/requirements.txt
export PYFORGE_WEB_PYTHON="$PWD/.venv/bin/python"
pyforge web
```

Abre `http://127.0.0.1:8000/juego/`, pulsa iniciar y usa flechas o WASD dentro del
tablero. Puedes pausar o reiniciar. Chocar termina la partida; comer da 10 puntos.
El límite del ejemplo es 100 segmentos y cuatro pasos por segundo.

- `src/main.st`: elige la operación solicitada.
- `src/snake.st`: lee JSON, llama al motor y devuelve estado.
- Módulo `juego`: movimiento, colisiones y selección de comida libre.
- `web/comunidad/views.py`: CSRF, cuota y estado por sesión, sin aceptar puntuaciones del cliente.
- `web/static/snake.js`: dibuja y envía teclas; no calcula reglas del juego.

Django guarda la sesión en SQLite; cambiar de navegador crea otra partida.
Al cerrar y volver a abrir la página se ofrece reiniciar; la API mantiene el último
estado de sesión para consultas posteriores. No es multijugador ni un motor 3D.
No publiques runserver: producción requiere el despliegue HTTPS descrito en la guía.
