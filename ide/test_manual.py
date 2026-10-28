"""Comprueba presentación, índice y escape del manual sin ejecutar los ejemplos."""
import unittest
from manual import course_source, page


class ManualTests(unittest.TestCase):
    """El documento debe mantener el código como texto y ofrecer navegación local."""
    def test_colors_and_navigation(self):
        rendered = page('# Manual\n## Variables\n```st\nvariable nombre = "Ana";\nmostrar(nombre);\n```\n')
        self.assertIn('href="#seccion-2"', rendered)
        self.assertIn('id="seccion-2"', rendered)
        self.assertIn('tok-variable', rendered)
        self.assertIn('tok-string', rendered)
        self.assertIn('href="/inicio.html"', rendered)
        self.assertIn('/manual.css', rendered)

    def test_source_is_not_html(self):
        rendered = page('## <script>alert(1)</script>\n```python\nprint("<img>")\n```\n')
        self.assertNotIn('<script>', rendered)
        self.assertNotIn('<img>', rendered)
        self.assertIn('&lt;script&gt;', rendered)
        self.assertIn('tok-string', rendered)

    def test_course_removes_project_internals(self):
        source = ('## 8. Listas\nEjemplo para estudiantes.\n'
                  '## 14. Cómo está conectado el código\nsrc/compiler.rs\n'
                  '## 20. Excepciones\nUso normal.\n### Cómo están conectadas las funciones\nDetalle interno.\n')
        course = course_source(source)
        self.assertIn('Ejemplo para estudiantes', course)
        self.assertIn('Uso normal', course)
        self.assertNotIn('Cómo está conectado', course)
        self.assertNotIn('src/compiler.rs', course)
        self.assertNotIn('Detalle interno', course)
        self.assertIn('# Curso completo de ST', course)
