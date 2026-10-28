"""Configuración del inicio: datos editables, sin HTML ni comandos ejecutables."""
import json
import os
from pathlib import Path

ACTIONS = {'editor', 'abrir', 'crear', 'manual', 'apoyar'}


def config_path():
    """Permite personalizar la instalación sin escribir en /usr/share ni usar sudo."""
    return Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home() / '.config'))) / 'pyforge/menu.json'


def validate(data):
    """Acepta únicamente títulos y acciones conocidas para evitar inyección de comandos."""
    if not isinstance(data, dict):
        raise ValueError('El menú debe ser un objeto JSON.')
    for key in ('titulo', 'mensaje'):
        if not isinstance(data.get(key), str) or len(data[key]) > 500:
            raise ValueError(f'El campo {key} debe contener texto de hasta 500 caracteres.')
    options = data.get('opciones')
    if not isinstance(options, list) or not 1 <= len(options) <= 5:
        raise ValueError('El menú admite entre una y cinco opciones.')
    seen = set()
    for option in options:
        if not isinstance(option, dict) or option.get('accion') not in ACTIONS:
            raise ValueError('Acción de menú desconocida.')
        if option['accion'] in seen:
            raise ValueError('No se permiten acciones repetidas.')
        seen.add(option['accion'])
        if not isinstance(option.get('texto'), str) or not 1 <= len(option['texto']) <= 100:
            raise ValueError('Cada opción necesita un texto de 1 a 100 caracteres.')
    return data


def load():
    """Recarga la personalización en cada visita y recupera el menú original si falla."""
    default = Path(__file__).with_name('menu.json')
    custom = config_path()
    try:
        if custom.exists():
            if custom.stat().st_size > 16384:
                raise ValueError('Configuración demasiado grande.')
            return validate(json.loads(custom.read_text('utf-8')))
    except (OSError, ValueError):
        pass
    return validate(json.loads(default.read_text('utf-8')))
