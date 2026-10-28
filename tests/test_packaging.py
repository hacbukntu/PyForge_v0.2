"""Pruebas de selección y vigilancia; no instalan paquetes en el sistema."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('package', Path(__file__).resolve().parents[1] / 'scripts/package.py')
package = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(package)


class PackagingTests(unittest.TestCase):
    """Comprueba que una edición se detecta sin incorporar datos privados al paquete."""
    def test_resources_and_exclusions(self):
        """KEYWORDS y todos los recursos necesarios viajan, pero no perfiles ni cachés."""
        files, digest = package.snapshot()
        for name in ('src/st/lexer.rs', 'src/stdlib/juego.st', 'runtime/web_template/snake.st', 'runtime/web_template/static/snake.js', 'runtime/web_template/comunidad/migrations/0002_session_key.py', 'extensions/vscode-st/package.json', 'extensions/vscode-st/vocabulary.json', 'ide/server.py', 'ide/app.js', 'ide/vault.py', 'ide/privacy.py', 'ide/preferences.js', 'ide/lock.html', 'runtime/web_template/requirements.txt', 'runtime/web_template/templates/st.html', 'runtime/web_template/static/site.css', 'runtime/web_template/comunidad/migrations/0001_initial.py', 'ide/inicio.html', 'ide/inicio.js', 'ide/menu.json', 'ide/start_menu.py', 'ide/apoyar.html', 'ide/formats.js', 'ide/database.py', 'scripts/update-ide-menu.py', 'ide/st.svg', 'website/logo.svg', 'packaging/linux/st.xml', 'packaging/linux/pyforge-ide.desktop', 'packaging/windows/associations.ps1', 'runtime/integraciones.py', 'runtime/gui.py', 'ide/autopairs.js', 'packaging/windows/installer.nsi'):
            self.assertIn(name, files)
        self.assertEqual(len(digest), 64)
        self.assertFalse(any('firefox-' in name or '__pycache__' in name or name.startswith('.git/') for name in files))

    def test_changes_additions_deletions(self):
        """La huella cambia al editar, añadir o eliminar fuentes y permanece estable sin cambios."""
        original = package.ROOT
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in package.inputs():
                relative = path.relative_to(original)
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(path.read_bytes())
            package.ROOT = root
            try:
                _, first = package.snapshot()
                self.assertEqual(first, package.snapshot()[1])
                lexer = root / 'src/st/lexer.rs'
                lexer.write_bytes(lexer.read_bytes() + b'\n// cambio de prueba\n')
                second = package.snapshot()[1]
                self.assertNotEqual(first, second)
                added = root / 'src/nuevo.rs'
                added.write_text('// modulo nuevo\n')
                self.assertNotEqual(second, package.snapshot()[1])
                added.unlink()
                self.assertEqual(second, package.snapshot()[1])
            finally:
                package.ROOT = original


if __name__ == '__main__':
    unittest.main()
