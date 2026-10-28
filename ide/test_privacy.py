"""Regresiones del cifrado; solo usa bóvedas y contraseñas ficticias temporales."""
import io
import json
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace
from vault import Vault
from privacy import Access
from server import Handler, Workspace, REPO


class VaultTests(unittest.TestCase):
    """Comprueba cifrado autenticado, guardado, corrupción y bloqueo de la API."""
    def test_roundtrip_and_tamper(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'prueba.stvault'
            protected=Vault(path,'Contraseña-ficticia-123',True)
            try:
                (protected.root/'src/main.st').write_text('mostrar("CONTENIDO-PRIVADO");')
                protected.save()
                first=path.read_bytes()
                self.assertNotIn(b'CONTENIDO-PRIVADO',first)
                protected.save()
                self.assertNotEqual(first,path.read_bytes())
                self.assertFalse(protected.verify('Contraseña-incorrecta-456'))
            finally:
                protected.close()
            self.assertFalse(protected.root.exists())
            reopened=Vault(path,'Contraseña-ficticia-123')
            self.assertIn('CONTENIDO-PRIVADO',(reopened.root/'src/main.st').read_text())
            reopened.close()
            altered=bytearray(path.read_bytes()); altered[-1]^=1; path.write_bytes(altered)
            with self.assertRaisesRegex(ValueError,'alterado'):
                Vault(path,'Contraseña-ficticia-123')

    def test_native_execution_stays_in_memory(self):
        """Compila en tmpfs y ejecuta por memfd aunque /dev/shm esté montado noexec."""
        import time
        with tempfile.TemporaryDirectory() as directory:
            protected=Vault(Path(directory)/'ejecutar.stvault','Clave-de-prueba-789',True)
            try:
                ws=Workspace(protected.root,REPO/'target/release/pyforgec');ws.vault=protected
                ws.start('src/main.st')
                deadline=time.monotonic()+10
                while ws.running and time.monotonic()<deadline:
                    time.sleep(.02)
                self.assertFalse(ws.running)
                self.assertIn('Hola desde tu proyecto privado',ws.output)
                self.assertIn('código 0',ws.output)
            finally:
                ws.stop()
                protected.close()

    def test_lock_unlock_and_encrypted_save(self):
        """La API guarda cifrado, limpia RAM al bloquear y reabre con una cookie nueva."""
        with tempfile.TemporaryDirectory() as directory:
            protected=Vault(Path(directory)/'sesion.stvault','Clave-local-ficticia-123',True)
            ws=Workspace(protected.root,REPO/'target/release/pyforgec');ws.vault=protected
            server=SimpleNamespace(workspace=ws,access=Access(protected),server_port=8765)
            token=server.access.login('Clave-local-ficticia-123')
            cookie='st-private='+token
            def request(route, body, cookie_value='', origin='http://127.0.0.1:8765'):
                """Ejercita el mismo manejador sin puertos ni navegador externo."""
                h=object.__new__(Handler);h.server=server;h.path='/api/'+route
                raw=json.dumps(body).encode()
                h.headers={'Host':'127.0.0.1:8765','Origin':origin,'Content-Type':'application/json','Cookie':cookie_value,'X-ST-Token':server.workspace.token,'Content-Length':str(len(raw))}
                h.rfile=io.BytesIO(raw);result=[];h.reply=lambda *args:result.append(args)
                h.do_POST()
                return result[0],getattr(h,'session_cookie','')
            try:
                original=(protected.root/'src/main.st').read_text()
                result,_=request('save',{'path':'src/main.st','original':original,'text':'mostrar("DATOS-PRIVADOS");'},cookie)
                self.assertEqual(result[0],200)
                self.assertNotIn(b'DATOS-PRIVADOS',protected.path.read_bytes())
                self.assertEqual(request('lock',{},cookie)[0][0],200)
                self.assertFalse(protected.root.exists())
                self.assertEqual(request('read',{'path':'src/main.st'},cookie)[0][0],401)
                result,_=request('unlock',{'password':'Clave-local-ficticia-123'},origin='https://ajeno.invalid')
                self.assertEqual(result[0],403)
                result,new_cookie=request('unlock',{'password':'Clave-local-ficticia-123'})
                self.assertEqual(result[0],200)
                self.assertIn('HttpOnly',new_cookie)
                result,_=request('read',{'path':'src/main.st'},new_cookie.split(';')[0])
                self.assertEqual(result[0],200)
                self.assertIn('DATOS-PRIVADOS',result[1]['text'])
            finally:
                server.workspace.vault.close()

    def test_access_and_api_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            protected=Vault(Path(directory)/'privado.stvault','Otra-contraseña-456',True)
            try:
                access=Access(protected)
                self.assertFalse(access.valid({}))
                token=access.login('Otra-contraseña-456')
                self.assertTrue(access.valid({'Cookie':'st-private='+token}))
                ws=Workspace(protected.root,REPO/'target/release/pyforgec');ws.vault=protected
                request=object.__new__(Handler)
                request.server=SimpleNamespace(workspace=ws,access=access,server_port=8765)
                request.path='/api/read'
                data=json.dumps({'path':'src/main.st'}).encode()
                request.headers={'Host':'127.0.0.1:8765','X-ST-Token':ws.token,'Content-Length':str(len(data))}
                request.rfile=io.BytesIO(data)
                result=[];request.reply=lambda *args:result.append(args)
                request.do_POST()
                self.assertEqual(result[0][0],401)  # Un token API sin cookie ya no basta.
                access.lock_all()
                self.assertFalse(access.valid({'Cookie':'st-private='+token}))
            finally:
                protected.close()
