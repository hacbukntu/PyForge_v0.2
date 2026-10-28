"""Verifica generación pública sin ejecutar programas ST ni instalar paquetes."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('site_builder', Path(__file__).resolve().parents[1] / 'scripts/build-site.py')
site = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(site)


class WebsiteTests(unittest.TestCase):
    """Asegura que la documentación se publica como texto y las descargas corresponden al manifiesto."""
    def test_markdown_escapes_html(self):
        rendered = site.markdown('### Texto\n<script>alert(1)</script>\n\n```st\nmostrar("<img>");\n```\n')
        self.assertNotIn('<script>', rendered)
        self.assertNotIn('<img>', rendered)
        self.assertIn('&lt;script&gt;', rendered)
        self.assertIn('<pre><code>', rendered)

    def test_generation_and_updates(self):
        original = site.ROOT
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ('website', 'examples', 'docs', 'src/st', 'ide', 'dist/packages', 'scripts'):
                (root/name).mkdir(parents=True, exist_ok=True)
            # El generador real y language.py se conservan para usar el mismo vocabulario.
            (root/'ide/language.py').write_bytes((original/'ide/language.py').read_bytes())
            (root/'src/st/lexer.rs').write_bytes((original/'src/st/lexer.rs').read_bytes())
            (root/'website/backend').mkdir()
            (root/'website/backend/settings.py').write_text('privado')
            (root/'website/backend/sitio.sqlite3').write_bytes(b'privado')
            (root/'website/index.html').write_text('<html lang="es"></html>')
            (root/'docs/MANUAL_ST.md').write_text('# Curso\n## Primera lección\nTexto vigente.\n')
            (root/'docs/INSTALACION.md').write_text('Instalación')
            (root/'examples/demo.st').write_text('mostrar(1);')
            (root/'examples/.venv').mkdir()
            (root/'examples/.venv/privado.py').write_text('dato privado')
            (root/'examples/data').mkdir()
            (root/'examples/data/privado.py').write_text('dato privado')
            binary = b'paquete de prueba'
            (root/'dist/packages/demo.deb').write_bytes(binary)
            manifest = {'version': 'prueba', 'artifacts': ['demo.deb'],
                        'sha256': {'demo.deb': hashlib.sha256(binary).hexdigest()}}
            (root/'dist/packages/latest.json').write_text(json.dumps(manifest))
            with patch.object(site, 'ROOT', root), patch.object(site, 'fingerprint', return_value='test'):
                site.build()
                output = root/'dist/site'
                self.assertFalse((output/'backend').exists())
                self.assertFalse((output/'ejemplos/.venv').exists())
                self.assertFalse((output/'ejemplos/data').exists())
                data = json.loads((output/'curso.json').read_text())
                self.assertEqual(len(data['lessons']), 1)
                self.assertEqual((output/'descargas/demo.deb').read_bytes(), binary)
                self.assertTrue(data['vocabulary']['keywords'])
                (root/'docs/MANUAL_ST.md').write_text('# Curso\n## Segunda lección\nContenido actualizado.\n')
                (root/'examples/demo.st').unlink()
                (root/'examples/nuevo.st').write_text('mostrar(2);')
                site.build()
                data = json.loads((output/'curso.json').read_text())
                self.assertEqual(data['lessons'][0]['title'], 'Segunda lección')
                self.assertFalse((output/'ejemplos/demo.st').exists())
                self.assertTrue((output/'ejemplos/nuevo.st').exists())
                (root/'dist/packages/demo.deb').write_bytes(b'corrupto')
                with self.assertRaisesRegex(ValueError, 'huella'):
                    site.build()
                self.assertEqual(json.loads((output/'curso.json').read_text()), data)


class SyntaxColorTests(unittest.TestCase):
    """Los colores no deben ejecutar HTML ni modificar el código original."""
    def test_st_comments_and_text_preservation(self):
        from html.parser import HTMLParser
        class Text(HTMLParser):
            def __init__(self):
                super().__init__(); self.parts = []
            def handle_data(self, data):
                self.parts.append(data)
        code = '/* mostrar(1); */\n"""funcion oculta() {}"""\nvariable nombre = "<script>";\nmostrar(nombre);'
        rendered = site.highlight(code, 'st')
        self.assertEqual(rendered.count('class="tok-comment"'), 2)
        self.assertIn('class="tok-function"', rendered)
        self.assertIn('class="tok-keyword"', rendered)
        self.assertNotIn('<script>', rendered)
        parser = Text(); parser.feed(rendered)
        self.assertEqual(''.join(parser.parts), code)

    def test_python_and_course_blocks(self):
        rendered = site.highlight('def sumar(a, b):\n    # comentario\n    return a + b\n', 'py')
        self.assertIn('tok-function', rendered)
        self.assertIn('tok-comment', rendered)
        self.assertIn('tok-keyword', rendered)
        self.assertIn('tok-function', site.markdown('```st\nmostrar(42);\n```'))
        self.assertNotIn('tok-', site.markdown('```bash\ncargo build\n```'))

if __name__ == '__main__':
    unittest.main()
