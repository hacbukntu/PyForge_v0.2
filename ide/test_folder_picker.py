"""Prueba el protocolo del selector sin abrir ventanas en la suite automática."""
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import folder_picker


class FolderPickerTests(unittest.TestCase):
    """Cancelación y errores no se confunden con una ruta elegida."""
    def test_selection_and_cancel(self):
        with patch.object(folder_picker.subprocess, 'run', return_value=SimpleNamespace(returncode=0, stdout='"/tmp/Mi proyecto"')) as run:
            self.assertEqual(folder_picker.choose(Path('/tmp')), Path('/tmp/Mi proyecto'))
            self.assertNotIn('shell', run.call_args.kwargs)
        with patch.object(folder_picker.subprocess, 'run', return_value=SimpleNamespace(returncode=0, stdout='null')):
            self.assertIsNone(folder_picker.choose(Path('/tmp'), True))

    def test_missing_display(self):
        with patch.object(folder_picker.subprocess, 'run', return_value=SimpleNamespace(returncode=1, stdout='')):
            with self.assertRaisesRegex(ValueError, 'sesión gráfica'):
                folder_picker.choose(Path('/tmp'))
