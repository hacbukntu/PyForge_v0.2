"""Pruebas del reconocimiento Python sin ejecutar el programa editado."""
import unittest
from python_language import highlight, vocabulary


class PythonLanguageTests(unittest.TestCase):
    """Comprueba vocabulario, sintaxis moderna y conservación exacta del texto."""
    def test_vocabulario(self):
        """Incluye palabras contextuales, asincronía y funciones incorporadas."""
        data = vocabulary()
        for word in ['async', 'await', 'yield', 'match', 'case', 'class', 'import']:
            self.assertIn(word, data['keywords'])
        self.assertIn('print', data['builtins'])

    def test_colores_y_texto(self):
        """Mantiene Unicode y distingue comentarios, números, cadenas y definiciones."""
        source = '# español 😀\n@decorador\nasync def saludo():\n    texto = r"""hola\nmundo"""\n    return 0xFF + 1.5e2 // 2\n'
        pieces = highlight(source)['pieces']
        self.assertEqual(''.join(p['text'] for p in pieces), source)
        categories = {p['text']: p['category'] for p in pieces}
        self.assertEqual(categories['saludo'], 'function')
        self.assertEqual(categories['async'], 'keyword')
        self.assertEqual(categories['0xFF'], 'number')
        self.assertEqual(categories['//'], '')
        self.assertEqual(categories['# español 😀'], 'comment')
        self.assertEqual(categories['r"""hola\nmundo"""'], 'string')

    def test_edicion_incompleta(self):
        """Nunca pierde texto cuando hay errores de sangría, cadenas o bloques abiertos."""
        for source in ['x = (', '"""sin cerrar', 'if True:\n  x=1\n y=2', 'print("😀")\n', 'x = f"valor {1+2}"', '<script>alert(1)</script>']:
            self.assertEqual(''.join(p['text'] for p in highlight(source)['pieces']), source)


if __name__ == '__main__':
    unittest.main()
