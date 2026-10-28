"""Revisa el auditor con secretos sintéticos y un índice Git temporal."""
import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('publication', Path(__file__).resolve().parents[1] / 'scripts/check-publication.py')
publication = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(publication)


class PublicationTests(unittest.TestCase):
    """El índice real manda: limpiar solo el disco no limpia lo que se va a publicar."""
    def test_index_and_redaction(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            token = 'ghp_' + 'a' * 40
            file = root / 'ejemplo.txt'
            file.write_text(token)
            subprocess.run(['git', '-C', str(root), 'add', 'ejemplo.txt'], check=True)
            file.write_text('texto público')
            output = io.StringIO()
            with patch.object(publication, 'ROOT', root), patch('sys.argv', ['check', '--staged']), contextlib.redirect_stdout(output):
                self.assertTrue(publication.main())
            self.assertIn('token GitHub', output.getvalue())
            self.assertNotIn(token, output.getvalue())

    def test_private_names_and_templates(self):
        for name in ['.env', 'ide/firefox-test-demo/logins.json', '.st-ide-trash/notas.txt', 'datos.db', 'secrets.toml']:
            self.assertTrue(publication.private_name(name))
        self.assertFalse(publication.private_name('.env.example'))
        self.assertFalse(publication.private_name('secrets.example.toml'))
