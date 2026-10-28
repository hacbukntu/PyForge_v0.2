"""Regresión de firmas estándar: el IDE lee las fuentes públicas del compilador."""
from pathlib import Path
import tempfile
import unittest
from language import standard_modules, vocabulary


class StandardLibraryTests(unittest.TestCase):
    """Comprueba exposición pública y actualización sin mantener otra tabla de funciones."""
    def test_public_functions(self):
        """Conserva los módulos originales y expone las nuevas firmas sin auxiliares privados."""
        modules = vocabulary()['modules']
        self.assertTrue({'time', 'os', 'math', 'io', 'json', 'integraciones', 'archivos', 'db', 'documentos', 'red', 'cifrado', 'gui', 'comandos', 'sistema', 'vpn'} <= set(modules))
        self.assertIn('abrir_archivo', [f['name'] for f in modules['nativo']])
        self.assertIn('abrir_archivo', [f['name'] for f in modules['recursos']])
        self.assertIn('desde_texto', [f['name'] for f in modules['recursos']])
        self.assertEqual(modules['red.tcp'], modules['red'])
        self.assertIn('escribir_texto', [f['name'] for f in modules['archivos']])
        self.assertIn('raiz', [f['name'] for f in modules['math']])
        self.assertIn('objeto', [f['name'] for f in modules['json']])
        self.assertFalse(any(f['name'].startswith('aux_') for functions in modules.values() for f in functions))

    def test_changes_and_comments(self):
        """Agregar una firma se refleja al releer; comentarios y auxiliares no son API pública."""
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'demo.st'
            source.write_text('// funcion falsa() -> entero {}\nfuncion real(x: entero) -> entero { retornar x; }\nfuncion aux_privada() -> nada {}\n')
            first = standard_modules(directory)
            self.assertEqual([f['name'] for f in first['demo']], ['real'])
            source.write_text(source.read_text() + '\nfuncion nueva() -> cadena { retornar "hola"; }\n')
            self.assertEqual([f['name'] for f in standard_modules(directory)['demo']], ['real', 'nueva'])
