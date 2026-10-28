"""Regresiones del servidor sin dependencias: python3 -m unittest discover -s ide."""
import tempfile
import io
import json
from types import SimpleNamespace
import time
import unittest
from pathlib import Path
from server import Workspace, Handler, REPO, opening_target


class WorkspaceTests(unittest.TestCase):
    """Comprueba archivos, ejecución real y cancelación en proyectos temporales."""
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='st-ide-test-')
        self.root = Path(self.temp.name)
        self.ws = Workspace(self.root, REPO / 'target/release/pyforgec')

    def tearDown(self):
        self.ws.stop()
        self.wait_finished()
        self.temp.cleanup()

    def wait_finished(self):
        deadline = time.monotonic() + 5
        while self.ws.running and time.monotonic() < deadline:
            time.sleep(.02)
        self.assertFalse(self.ws.running, 'El proceso no terminó a tiempo')

    def test_native_picker_cancel_and_authentication(self):
        """Cancelar conserva la sesión; una petición ajena no abre ventanas del sistema."""
        from unittest.mock import patch
        with patch('server.folder_picker.choose', return_value=None) as choose:
            self.assertEqual(self.request('project-select', {}, token='incorrecto')[0], 403)
            choose.assert_not_called()
            self.assertEqual(self.request('project-select', {})[1], {'cancelled': True})
        with patch('server.folder_picker.choose', return_value=self.root):
            self.assertEqual(self.request('project-select', {})[0], 200)
        with patch('server.folder_picker.choose', return_value=self.root / 'desde_selector'):
            self.assertEqual(self.request('project-select', {'create': True})[0], 200)
            self.assertTrue((self.root / 'desde_selector/src/main.st').exists())

    def test_start_menu_and_project_creation(self):
        """Crear reutiliza PyForge y rechaza sobrescritura, rutas relativas y ejecución activa."""
        status, menu = self.request('start-menu', {})
        self.assertEqual(status, 200)
        self.assertEqual(menu['project'], str(self.root))
        target = self.root / 'nuevo'
        self.assertEqual(self.request('project-open', {'path':str(target), 'create':True})[0], 200)
        self.assertTrue((target / 'src/main.st').is_file())
        self.assertEqual(self.request('project-open', {'path':str(target), 'create':True})[0], 400)
        self.assertEqual(self.request('project-open', {'path':'relativo'})[0], 400)
        self.assertEqual(self.request('project-open', {'path':str(target)}, token='incorrecto')[0], 403)
        self.ws.running = True
        try:
            self.assertEqual(self.request('project-open', {'path':str(target)})[0], 400)
        finally:
            self.ws.running = False

    def test_sqlite_readonly(self):
        """Inspeccionar SQLite conserva sus bytes y bloquea guardado por la API."""
        import sqlite3
        path = self.root / 'datos.db'
        with sqlite3.connect(path) as db:
            db.execute('CREATE TABLE datos (nombre TEXT)')
            db.execute('INSERT INTO datos VALUES (?)', ('Ana',))
        before = path.read_bytes()
        self.assertIn('datos.db', self.ws.tree())
        status, result = self.request('read', {'path':'datos.db'})
        self.assertEqual(status, 200)
        self.assertTrue(result['readonly'])
        self.assertIn('CREATE TABLE', result['text'])
        self.assertIn('Ana', result['text'])
        self.assertEqual(self.request('save', {'path':'datos.db', 'original':result['text'], 'text':'roto'})[0], 400)
        self.assertEqual(path.read_bytes(), before)
        (self.root / 'invalida.db').write_text('no es SQLite')
        self.assertEqual(self.request('read', {'path':'invalida.db'})[0], 400)

    def test_permisos_sin_exponer_otras_tablas(self):
        """La consulta no revela secretos ni transforma booleanos falsos en permisos."""
        (self.root / 'PyForge.toml').write_text('[permisos]\nred = false\n[privado]\nvalor = "NO_MOSTRAR"\n')
        status, result = self.request('permissions', {})
        self.assertEqual(status, 200)
        self.assertEqual(result['permisos'], {'red': False})
        self.assertFalse(result['sandbox'])
        self.assertNotIn('NO_MOSTRAR', json.dumps(result))
        (self.root / 'PyForge.toml').write_text('[permisos]\nred = "false"\n')
        self.assertEqual(self.request('permissions', {})[0], 400)

    def test_abrir_fuente_desde_el_sistema(self):
        """Encuentra el proyecto sin ejecutar el archivo ni cambiar el arranque por carpeta."""
        (self.root / 'src').mkdir()
        source = self.root / 'src/hola con espacios.st'
        source.write_text('mostrar("no ejecutar al abrir");')
        self.assertEqual(opening_target(source), (source.parent, source.name))
        (self.root / 'PyForge.toml').write_text('[package]\nname="prueba"\n')
        self.assertEqual(opening_target(source), (self.root, 'src/hola con espacios.st'))
        self.assertEqual(opening_target(self.root), (self.root, None))
        with self.assertRaises(ValueError):
            opening_target(self.root / 'ausente.st')
        self.ws.initial_file = 'src/hola con espacios.st'
        self.assertEqual(self.request('tree', {})[1]['initial_file'], self.ws.initial_file)
        self.assertFalse(self.ws.running)

    def test_rutas_y_enlaces(self):
        with self.assertRaises(ValueError):
            self.ws.path('../fuera.st')
        (self.root / 'enlace').symlink_to('/tmp', target_is_directory=True)
        with self.assertRaises(ValueError):
            self.ws.path('enlace/fuera.st')
        self.assertEqual(self.ws.path('main.st'), self.root / 'main.st')

    def test_explorador(self):
        (self.root / 'main.st').write_text('mostrar(1);')
        (self.root / 'target').mkdir()
        (self.root / 'target/ignorado.st').write_text('')
        self.assertEqual(self.ws.tree(), ['main.st'])

    def test_limite_salida(self):
        self.ws.append('a' * 1_100_000)
        self.assertEqual(len(self.ws.output), 1_000_000)

    def test_ejecucion_y_unicode(self):
        (self.root / 'main.st').write_text('mostrar("Hola ñ 😀"); mostrar(20+22);')
        self.ws.start('main.st')
        self.wait_finished()
        self.assertIn('Hola ñ 😀\n42\n', self.ws.output)
        self.assertIn('código 0', self.ws.output)
        self.assertEqual(sorted(p.name for p in self.root.iterdir()), ['main.st'])

    def test_ejecucion_python_desde_api(self):
        """La API compila Python real, conserva Unicode y no deja binarios en el proyecto."""
        (self.root / 'main.py').write_text('def doble(n: int) -> int:\n    return n * 2\nprint("Hola ñ")\nprint(doble(21))\n')
        self.assertEqual(self.request('run', {'path': 'main.py'})[0], 200)
        self.wait_finished()
        self.assertIn('Hola ñ\n42\n', self.ws.output)
        self.assertIn('código 0', self.ws.output)
        self.assertEqual(sorted(p.name for p in self.root.iterdir()), ['main.py'])

    def test_python_no_admitido(self):
        """Un import de Python no soportado falla en compilación y nunca se ejecuta."""
        (self.root / 'web.py').write_text('from flask import Flask\n')
        self.assertEqual(self.request('run', {'path': 'web.py'})[0], 200)
        self.wait_finished()
        self.assertIn('error', self.ws.output.lower())
        self.assertNotIn('> Ejecutando', self.ws.output)

    def test_extension_no_ejecutable(self):
        """El soporte Python no permite ejecutar otros formatos ni rutas inexistentes."""
        (self.root / 'notas.md').write_text('print(42)')
        for path in ['notas.md', 'ausente.py']:
            self.assertEqual(self.request('run', {'path': path})[0], 400)
        self.assertFalse(self.ws.running)

    def test_escalares_st(self):
        """El IDE compila e imprime los cuatro tipos con el mismo ejecutable nativo."""
        (self.root / 'valores.st').write_text('variable nombre: cadena = "Ana"; variable edad: entero = 20; variable precio: flotante = 12.5; variable activo: booleano = verdadero; mostrar(nombre); mostrar(edad); mostrar(precio); mostrar(activo);')
        self.assertEqual(self.request('run', {'path': 'valores.st'})[0], 200)
        self.wait_finished()
        self.assertIn('Ana\n20\n12.5\nTrue\n', self.ws.output)
        self.assertIn('código 0', self.ws.output)

    def test_diagnostico(self):
        (self.root / 'main.st').write_text('mostrar(;')
        self.ws.start('main.st')
        self.wait_finished()
        self.assertIn('error', self.ws.output.lower())
        self.assertNotIn('> Ejecutando', self.ws.output)

    def test_detener(self):
        (self.root / 'main.st').write_text('mientras verdadero { pasar; }')
        self.ws.start('main.st')
        with self.assertRaises(ValueError):
            self.ws.start('main.st')
        deadline = time.monotonic() + 5
        while '> Ejecutando' not in self.ws.output and time.monotonic() < deadline:
            time.sleep(.02)
        self.assertIn('> Ejecutando', self.ws.output)
        self.ws.stop()
        self.wait_finished()
        self.assertTrue(self.ws.cancelled)

    def request(self, route, body, token=None):
        """Ejercita el manejador HTTP con una petición en memoria, sin abrir puertos."""
        request = object.__new__(Handler)
        payload = json.dumps(body).encode()
        request.server = SimpleNamespace(workspace=self.ws, server_port=8765)
        request.headers = {'Host': '127.0.0.1:8765', 'X-ST-Token': self.ws.token if token is None else token, 'Content-Length': str(len(payload))}
        request.rfile = io.BytesIO(payload)
        request.path = '/api/' + route
        responses = []
        request.reply = lambda status, value: responses.append((status, value))
        request.do_POST()
        return responses[0]

    def test_api_crear_guardar_conflicto(self):
        self.assertEqual(self.request('create', {'path': 'src/nuevo.st'})[0], 200)
        self.assertEqual(self.request('create', {'path': 'src/nuevo.st'})[0], 400)
        status, data = self.request('read', {'path': 'src/nuevo.st'})
        self.assertEqual(status, 200)
        self.assertEqual(self.request('save', {'path':'src/nuevo.st', 'original': data['text'], 'text': 'mostrar(42);'})[0], 200)
        self.assertEqual(self.request('save', {'path':'src/nuevo.st', 'original': data['text'], 'text': 'mostrar(0);'})[0], 400)
        self.assertEqual((self.root / 'src/nuevo.st').read_text(), 'mostrar(42);')

    def test_api_sesion_y_rutas(self):
        self.assertEqual(self.request('tree', {}, token='incorrecto')[0], 403)
        self.assertEqual(self.request('read', {'path':'../fuera.st'})[0], 400)
        self.assertEqual(self.request('create', {'path':'archivo.py'})[0], 400)
        self.assertEqual(self.request('desconocido', {})[0], 404)

    def test_api_reconocimiento_python(self):
        """El coloreado usa la sesión existente y valida el texto sin ejecutarlo."""
        self.assertEqual(self.request('python-language', {})[0], 200)
        status, data = self.request('python-highlight', {'text': 'print(42)'})
        self.assertEqual(status, 200)
        self.assertEqual(''.join(piece['text'] for piece in data['pieces']), 'print(42)')
        self.assertEqual(self.request('python-highlight', {'text': 42})[0], 400)
        self.assertEqual(self.request('python-highlight', {'text': ''}, token='incorrecto')[0], 403)

    def test_crear_siempre_en_src(self):
        """La API normaliza rutas, conserva exclusividad y bloquea escapes/enlaces."""
        status, data = self.request('create', {'path': 'carpeta/programa.st'})
        self.assertEqual(status, 200)
        self.assertEqual(data['path'], 'src/carpeta/programa.st')
        self.assertTrue((self.root / data['path']).is_file())
        self.assertEqual(self.request('create', {'path': 'src/carpeta/programa.st'})[0], 400)
        for path in ['../fuera.st', '/tmp/fuera.st', 'src/../fuera.st']:
            self.assertEqual(self.request('create', {'path': path})[0], 400)
        (self.root / 'src/enlace').symlink_to(self.root, target_is_directory=True)
        self.assertEqual(self.request('create', {'path': 'enlace/fuera.st'})[0], 400)

    def test_eliminar_con_papelera(self):
        """Mover conserva el contenido recuperable y rechaza cambios externos y carpetas."""
        path = self.root / 'programa.st'
        path.write_text('mostrar(42);')
        self.assertEqual(self.request('delete', {'path': 'programa.st', 'original': 'otro'})[0], 400)
        status, data = self.request('delete', {'path': 'programa.st', 'original': 'mostrar(42);'})
        self.assertEqual(status, 200)
        self.assertFalse(path.exists())
        self.assertEqual((self.root / data['trash']).read_text(), 'mostrar(42);')
        self.assertEqual(self.ws.tree(), [])
        self.assertEqual(self.request('delete', {'path': '.', 'original': ''})[0], 400)
        self.assertEqual(self.request('delete', {'path': '../fuera.st', 'original': ''})[0], 400)

    def test_eliminar_sesion_y_enlaces(self):
        """Eliminar requiere sesión, archivo real y ausencia de ejecución activa."""
        (self.root / 'original.st').write_text('')
        (self.root / 'enlace.st').symlink_to(self.root / 'original.st')
        self.assertEqual(self.request('delete', {'path': 'enlace.st', 'original': ''})[0], 400)
        self.assertEqual(self.request('delete', {'path': 'original.st', 'original': ''}, token='incorrecto')[0], 403)
        self.ws.running = True
        try:
            self.assertEqual(self.request('delete', {'path': 'original.st', 'original': ''})[0], 400)
        finally:
            self.ws.running = False
        self.assertTrue((self.root / 'original.st').exists())

    def test_entrada_preparada(self):
        """La entrada estándar preparada llega al programa nativo, seguida de EOF."""
        (self.root / 'entrada.st').write_text('mostrar(entrada()); mostrar(entrada()); mostrar(longitud(entrada()));')
        self.assertEqual(self.request('run', {'path': 'entrada.st', 'stdin': 'Ana ñ\n😀\n'})[0], 200)
        self.wait_finished()
        self.assertIn('Ana ñ\n😀\n0\n', self.ws.output)
        self.assertEqual(self.request('run', {'path': 'entrada.st', 'stdin': 42})[0], 400)

    def wait_prompt(self, prompt):
        """Espera salida real del programa antes de enviar una respuesta interactiva."""
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            with self.ws.lock:
                if self.ws.input_ready and prompt in self.ws.output:
                    return
            time.sleep(.01)
        self.fail('No apareció el mensaje interactivo: ' + self.ws.output)

    def test_entrada_interactiva(self):
        """Dos respuestas Unicode llegan después de sus mensajes; EOF completa la última lectura."""
        (self.root / 'main.st').write_text('mostrar(entrada("Nombre: ")); mostrar(entrada("Otro: ")); mostrar(longitud(entrada("Fin: "))); escribir("ubicacion.txt", "proyecto");')
        self.assertEqual(self.request('run', {'path':'main.st'})[0], 200)
        self.wait_prompt('Nombre: ')
        self.assertTrue(self.ws.running)
        self.assertEqual(self.request('input', {'text':'Ana ñ'})[0], 200)
        self.wait_prompt('Otro: ')
        self.assertEqual(self.request('input', {'text':'😀'})[0], 200)
        self.wait_prompt('Fin: ')
        self.assertEqual(self.request('input', {'eof':True})[0], 200)
        self.wait_finished()
        self.assertIn('Ana ñ', self.ws.output)
        self.assertIn('😀', self.ws.output)
        self.assertIn('0\n', self.ws.output)
        self.assertEqual((self.root/'ubicacion.txt').read_text(), 'proyecto')
        self.assertEqual(self.request('input', {'text':'tarde'})[0], 400)

    def test_biblioteca_estandar_interactiva(self):
        """Los módulos incrustados se ejecutan desde el IDE con entrada viva y cwd del proyecto."""
        (self.root / 'main.st').write_text('importar "io"; importar "json"; importar "os"; importar "math"; importar "time"; variable nombre = io.leer_linea("Nombre estándar: "); io.imprimir(json.decodificar(json.codificar(nombre))); io.escribir_texto("directorio.txt", os.directorio_actual()); mostrar(math.raiz(16.0)); time.dormir(0);')
        self.ws.start('main.st')
        self.wait_prompt('Nombre estándar: ')
        self.assertEqual(self.request('input', {'text':'Ana 🚀'})[0], 200)
        self.wait_finished()
        self.assertIn('Ana 🚀\n', self.ws.output)
        self.assertIn('4.0\n', self.ws.output)
        self.assertIn('código 0', self.ws.output)
        self.assertEqual((self.root / 'directorio.txt').read_text(), str(self.root))

    def test_tipo_e_inferencia(self):
        """El IDE ejecuta consultas de tipo y conversiones con variables sin anotación."""
        (self.root / 'main.st').write_text('variable texto = "20"; variable numero = entero(texto); mostrar(tipo(numero)); mostrar(tipo(numero) == "entero"); mostrar(flotante(numero)); intentar { mostrar(entero("no")); } capturar error { mostrar("capturado"); }')
        self.ws.start('main.st')
        self.wait_finished()
        self.assertIn('entero\nTrue\n20.0\ncapturado\n', self.ws.output)
        self.assertIn('código 0', self.ws.output)

    def test_comprobar_no_ejecuta(self):
        """Comprobar informa diagnósticos sin iniciar procesos persistentes ni ejecutar ST."""
        (self.root / 'main.st').write_text('mostrar(7);')
        status, result = self.request('check', {'path':'main.st'})
        self.assertEqual(status, 200)
        self.assertTrue(result['ok'])
        self.assertFalse(self.ws.running)
        (self.root / 'main.st').write_text('variable x = ;')
        status, result = self.request('check', {'path':'main.st'})
        self.assertEqual(status, 200)
        self.assertFalse(result['ok'])
        self.assertIn('ST línea', result['output'])
        self.assertFalse(self.ws.running)

    def test_cancelar_entrada_interactiva(self):
        """Detener termina también programas que están esperando stdin."""
        (self.root/'main.st').write_text('mostrar(entrada("Esperando: "));')
        self.ws.start('main.st')
        self.wait_prompt('Esperando: ')
        self.ws.stop()
        self.wait_finished()
        self.assertFalse(self.ws.input_ready)

    def test_limites_entrada_interactiva(self):
        """La API rechaza tipos y tamaños inválidos sin bloquear HTTP."""
        self.assertEqual(self.request('input', {'text':42})[0], 400)
        self.assertEqual(self.request('input', {'text':'ñ'*4096})[0], 400)
        self.assertEqual(self.request('input', {'eof':'si'})[0], 400)

    def test_entrada_sin_lector(self):
        """Una tubería llena devuelve error en lugar de retener el servidor."""
        (self.root/'main.st').write_text('mostrar(7); mientras verdadero { pasar; }')
        self.ws.start('main.st')
        self.wait_prompt('7\n')
        for _ in range(128):
            try:
                self.ws.send_input('x'*4095)
            except ValueError as error:
                self.assertIn('espera y reintenta', str(error))
                break
        else:
            self.fail('No se detectó la tubería llena')
        self.ws.stop()
        self.wait_finished()

    def test_compilador_ausente(self):
        (self.root / 'main.st').write_text('mostrar(1);')
        self.ws.compiler = self.root / 'ausente'
        with self.assertRaises(ValueError):
            self.ws.start('main.st')
        self.assertFalse(self.ws.running)


if __name__ == '__main__':
    unittest.main()
