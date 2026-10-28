"""Compila y ejecuta ST real: comprueba transporte, imports y excepciones capturables."""
import os
import sys
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
COMPILER = Path(os.environ.get('PYFORGE_COMPILER', ROOT / 'target/release/pyforgec'))


class NativeIntegrations(unittest.TestCase):
    """Necesita el compilador construido; no instala ni modifica el proyecto actual."""
    def compile_run(self, source, manifest, optimize=True, python_runtime=None):
        with tempfile.TemporaryDirectory(prefix='st-native-integration-') as directory:
            root = Path(directory)
            (root / 'main.st').write_text(source)
            (root / 'PyForge.toml').write_text(manifest)
            args = [str(COMPILER), 'build', str(root / 'main.st'), '-o', str(root / 'programa')]
            if not optimize:
                args.append('--no-opt')
            build_env = dict(os.environ)
            if python_runtime:
                build_env['PYFORGE_PYTHON'] = python_runtime
            result = subprocess.run(args, capture_output=True, text=True, timeout=30, env=build_env)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(root / 'programa')], cwd=root, capture_output=True, text=True, timeout=15, env={**os.environ, 'QT_QPA_PLATFORM': 'offscreen'})
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stderr, '')
            return result.stdout

    def test_all_modules_native_and_compatibility(self):
        source = '''
importar "archivos";
importar "nativo";
importar "db";
importar "documentos";
importar "red";
importar "cifrado";
importar "comandos";
importar "vpn";
importar "sistema";
importar "math";
importar "gui";
importar "red.ssl" como tls;
// La ABI Rust y el auxiliar Python conviven en el mismo ejecutable.
variable buffer_nativo = nativo.desde_texto("Hola ñ");
intentar {
    archivos.escribir_texto("nota.txt", nativo.texto(buffer_nativo), falso, "utf-8");
} finalmente { nativo.cerrar(buffer_nativo); }
mostrar(archivos.leer("nota.txt", "utf-8"));
variable conexion = db.conectar("prueba.db");
db.ejecutar(conexion, "CREATE TABLE ejemplo(n INTEGER)", "[]");
db.ejecutar(conexion, "INSERT INTO ejemplo VALUES (?)", "[42]");
mostrar(db.consultar(conexion, "SELECT n FROM ejemplo", "[]"));
db.confirmar(conexion);
db.cerrar(conexion);
mostrar(cifrado.desde_base64(cifrado.base64("Unicode ñ")));
mostrar(math.raiz(9.0));
'''
        manifest = '[permisos]\nleer_archivos=true\nescribir_archivos=true\nbase_datos=true\n'
        for optimize in [True, False]:
            result = self.compile_run(source, manifest, optimize)
            self.assertEqual(result, 'Hola ñ\n[[42]]\nUnicode ñ\n3.0\n')

    def test_denied_permission_is_catchable(self):
        result = self.compile_run('''
importar "archivos";
intentar { archivos.escribir_texto("no.txt", "x", falso, "utf-8"); }
capturar error { mostrar(buscar(error, "PERMISO_DENEGADO") >= 0); }
mostrar("continúa");
''', '')
        self.assertEqual(result, 'True\ncontinúa\n')

    def test_optional_native_documents_and_gui(self):
        """Verifica el recorrido ST→ELF→Python seleccionado→biblioteca real."""
        if any(importlib.util.find_spec(name) is None for name in ['docx', 'pypdf', 'reportlab', 'PySide6']):
            self.skipTest('Bibliotecas opcionales ausentes del intérprete de prueba')
        source = r'''importar "documentos";
importar "gui";
importar "json";
importar "time";
documentos.crear("nota.docx", "ST nativo y Word", falso);
mostrar(documentos.leer("nota.docx"));
documentos.crear("nota.pdf", "ST y PDF", falso);
mostrar(buscar(documentos.leer("nota.pdf"), "ST y PDF") >= 0);
variable ventana = gui.crear("{\"titulo\":\"Prueba\",\"controles\":[{\"tipo\":\"etiqueta\",\"id\":\"saludo\",\"texto\":\"Hola\"}]}");
variable preparada = falso;
para i en rango(0, 100) {
    variable evento = json.decodificar(json.obtener(gui.evento(ventana), "evento"));
    si evento == "lista" { preparada = verdadero; romper; }
    time.dormir(20);
}
mostrar(preparada);
gui.cambiar_texto(ventana, "saludo", "Actualizado");
gui.cerrar(ventana);
'''
        result = self.compile_run(source, '[permisos]\nleer_archivos=true\nescribir_archivos=true\ndocumentos=true\ngui=true\n', python_runtime=sys.executable)
        self.assertEqual(result, 'ST nativo y Word\nTrue\nTrue\n')


if __name__ == '__main__':
    unittest.main()
