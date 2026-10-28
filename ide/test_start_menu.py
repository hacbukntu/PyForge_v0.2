"""Valida personalización y actualización del menú sin modificar configuración del usuario."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import start_menu


class StartMenuTests(unittest.TestCase):
    """Las opciones son datos y el archivo inválido no bloquea el acceso al editor."""
    def test_update_and_fallback(self):
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {'XDG_CONFIG_HOME': directory}):
            default = start_menu.load()
            self.assertEqual(len(default['opciones']), 5)
            changed = {**default, 'titulo': 'Mi entorno ST'}
            source = Path(directory) / 'opciones.json'
            source.write_text(json.dumps(changed))
            script = Path(__file__).resolve().parents[1] / 'scripts/update-ide-menu.py'
            subprocess.run(['python3', str(script), str(source)], check=True, capture_output=True)
            self.assertEqual(start_menu.load()['titulo'], 'Mi entorno ST')
            start_menu.config_path().write_text('{roto')
            self.assertEqual(start_menu.load(), default)

    def test_reject_unknown_actions(self):
        with self.assertRaises(ValueError):
            start_menu.validate({'titulo':'ST', 'mensaje':'Texto', 'opciones':[{'accion':'shell', 'texto':'Ejecutar'}]})
