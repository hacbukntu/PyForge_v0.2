"""Prueba copia, bloqueo y detección de modificaciones sin red ni código externo."""
import pathlib
import subprocess
import sys
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[1] / 'scripts/bibliotecas.py'


class Bibliotecas(unittest.TestCase):
    def test_copia_y_verificacion(self):
        """Una dependencia copiada se verifica; una edición posterior falla explícitamente."""
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            project = root / 'proyecto'
            project.mkdir()
            (project / 'PyForge.toml').write_text('[package]\nname="prueba"\n')
            source = root / 'original/src'
            source.mkdir(parents=True)
            (source / 'lib.st').write_text('funcion valor()->entero { retornar 1; }')
            def run(*args):
                return subprocess.run([sys.executable, str(SCRIPT), *args, '--proyecto', str(project)], capture_output=True)
            self.assertEqual(run('agregar', 'demo', str(source.parent)).returncode, 0)
            self.assertEqual(run('verificar').returncode, 0)
            self.assertNotEqual(run('agregar', 'demo', str(source.parent)).returncode, 0)
            (project / 'bibliotecas/demo/src/lib.st').write_text('modificado')
            self.assertNotEqual(run('verificar').returncode, 0)
