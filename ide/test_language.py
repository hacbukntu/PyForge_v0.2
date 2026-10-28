"""Pruebas del vocabulario automático: cambios de Rust se reflejan sin listas duplicadas."""
import tempfile
import unittest
from pathlib import Path
from language import vocabulary


class LanguageTests(unittest.TestCase):
    """Usa una tabla temporal para no modificar las palabras reales del compilador."""
    def test_cambios_alias_y_categorias(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'lexer.rs'
            path.write_text('pub const KEYWORDS: &[(&str, Keyword)] = &[("mostrar", Keyword::Mostrar)];')
            first = vocabulary(path)
            path.write_text('''pub const KEYWORDS: &[(&str, Keyword)] = &[
                ("escribir", Keyword::Mostrar), ("imprimir", Keyword::Mostrar),
                ("coleccion", Keyword::Lista), ("cierto", Keyword::Verdadero),
                ("nuevo", Keyword::MiNuevaOperacion),
                // ("ignorado", Keyword::Mostrar),
                /* exterior /* anidado */ ("otro", Keyword::Mostrar) */
            ];''')
            second = vocabulary(path)
            self.assertNotEqual(first['version'], second['version'])
            self.assertEqual([x['word'] for x in second['keywords']], ['escribir','imprimir','coleccion','cierto','nuevo'])
            self.assertEqual([x['category'] for x in second['keywords']], ['keyword','keyword','type','boolean','keyword'])

    def test_tablas_invalidas(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'lexer.rs'
            for source in ['/* sin cierre', 'pub const KEYWORDS: &[(&str, Keyword)] = &[',
                'pub const KEYWORDS: &[(&str, Keyword)] = &[("x", Keyword::Si),("x", Keyword::Sino)];']:
                path.write_text(source)
                with self.assertRaises(ValueError): vocabulary(path)

    def test_vocabulario_real(self):
        data = vocabulary()
        kinds = {x['kind'] for x in data['keywords']}
        self.assertTrue({'Importar','Lista','Diccionario','Agregar','Longitud','Cadena','Flotante'} <= kinds)

        self.assertTrue(all(x["category"] == "type" for x in data["keywords"] if x["kind"] in {"Cadena", "Flotante"}))
        contextual = {x['word']: x['category'] for x in data['keywords'] if x['kind'] == 'Contextual'}
        self.assertEqual(contextual, {
            'usar': 'keyword', 'buffer_nativo': 'type', 'archivo_nativo': 'type'
        })
