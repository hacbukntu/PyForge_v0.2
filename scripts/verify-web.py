#!/usr/bin/env python3
"""Prueba local de extremo a extremo: genera una web, registra usuario y consulta ST nativo.

No publica en Internet. Usa una carpeta temporal, credenciales ficticias y loopback.
"""
import argparse
import json
import http.cookiejar
import os
from pathlib import Path
import re
import signal
import socket
import subprocess
import tempfile
import time
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def main():
    """Ejercita el CLI y HTTP reales; elimina únicamente sus recursos temporales."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--python',required=True,help='Intérprete con web/requirements.txt instalado.')
    parser.add_argument('--compiler',type=Path,default=ROOT/'target/release/pyforge')
    args=parser.parse_args()
    compiler=args.compiler.resolve()
    with tempfile.TemporaryDirectory(prefix='st-web-http-') as directory:
        root=Path(directory)
        subprocess.run([str(compiler),'new','sitio','--web'],cwd=root,check=True,capture_output=True)
        with socket.socket() as socket_test:
            socket_test.bind(('127.0.0.1',0)); port=socket_test.getsockname()[1]
        base=f'http://127.0.0.1:{port}'
        with tempfile.TemporaryFile() as output:
            process=subprocess.Popen([str(compiler),'web','--port',str(port)],cwd=root/'sitio',env={**os.environ,'PYFORGE_WEB_PYTHON':str(Path(args.python).absolute())},stdout=output,stderr=output,start_new_session=True)
            try:
                opener=urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
                deadline=time.monotonic()+25
                while True:
                    try:
                        response=opener.open(base+'/vista-st/',timeout=1)
                        page=response.read().decode();break
                    except OSError:
                        if time.monotonic()>deadline or process.poll() is not None:
                            output.seek(0)
                            detail=output.read(8192).decode('utf-8','replace')
                            raise RuntimeError('El servidor de prueba no quedó preparado: ' + detail[-2000:]) from None
                        time.sleep(.1)
                csrf=re.search(r'name="csrfmiddlewaretoken" value="([^"]+)"',page).group(1)
                page=opener.open(base+'/vista-st/').read().decode()
                csrf=re.search(r'name="csrfmiddlewaretoken" value="([^"]+)"',page).group(1)
                result=opener.open(base+'/vista-st/',urllib.parse.urlencode({'csrfmiddlewaretoken':csrf,'nombre':'Ana <script>'}).encode(),timeout=5).read().decode()
                assert 'Hola, Ana &lt;script&gt;' in result
                assert '<script>.' not in result
                game=opener.open(base+'/juego/').read().decode()
                token=re.search(r'name="csrfmiddlewaretoken" value="([^"]+)"',game).group(1)
                def step(action):
                    request=urllib.request.Request(base+'/api/snake/',json.dumps({'accion':action,'direccion':0}).encode(),headers={'Content-Type':'application/json','X-CSRFToken':token})
                    return json.loads(opener.open(request,timeout=5).read())
                initial=step('reiniciar')
                following=step('paso')
                assert initial['cuerpo'][0]==208 and following['cuerpo'][0]==209
                assert following['puntos']==0
                print('Web real: migraciones, CSRF, respuesta ST y movimiento nativo de culebrita correctos.')
            finally:
                try: os.killpg(process.pid,signal.SIGTERM)
                except ProcessLookupError: pass
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid,signal.SIGKILL);process.wait()


if __name__=='__main__':
    main()
