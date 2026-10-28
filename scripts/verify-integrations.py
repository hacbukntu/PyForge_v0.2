#!/usr/bin/env python3
"""Compila todos los ejemplos nuevos y ejecuta los autónomos en copias temporales."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
AUTONOMOUS = {'diagnostico', 'archivos', 'sqlite', 'cifrado_texto', 'contrasenas', 'documentos', 'comandos', 'vpn_diagnostico'}


def main():
    """Los ejemplos interactivos o con dependencias se identifican, nunca se finge ejecutarlos."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', default=str(ROOT / 'target/release/pyforgec'))
    parser.add_argument('--python', help='Intérprete opcional absoluto que se incrusta al compilar')
    parser.add_argument('--documents', action='store_true', help='Ejecuta también Word/PDF con bibliotecas instaladas')
    args = parser.parse_args()
    compiler = str(Path(args.compiler).resolve())
    env = dict(os.environ)
    if args.python:
        env['PYFORGE_PYTHON'] = str(Path(args.python).absolute())
    executed = 0
    projects = sorted(path for path in (ROOT / 'examples/integraciones').iterdir() if (path / 'src/main.st').is_file())
    for project in projects:
        with tempfile.TemporaryDirectory(prefix='st-ejemplo-integracion-') as folder:
            root = Path(folder)
            shutil.copytree(project / 'src', root / 'src')
            shutil.copy2(project / 'PyForge.toml', root / 'PyForge.toml')
            subprocess.run([compiler, 'build', str(root / 'src/main.st'), '-o', str(root / 'programa')],
                           check=True, capture_output=True, timeout=30, env=env)
            execute = project.name in AUTONOMOUS or (args.documents and project.name in {'word_pdf', 'pdf_paginas'})
            if execute:
                result = subprocess.run([str(root / 'programa')], cwd=root, capture_output=True, timeout=20)
                if result.returncode or result.stderr:
                    raise RuntimeError(f'{project.name}: fallo de ejecución (código {result.returncode})')
                if args.documents and project.name == 'word_pdf' and not (root / 'informe.pdf').is_file():
                    raise RuntimeError('word_pdf capturó un error: no se considera ejecución satisfactoria')
                executed += 1
            print(project.name + ': compilado' + (' y ejecutado' if execute else '; ejecución requiere entorno/interacción'))
    print(f'{len(projects)} compilados; {executed} ejecutados en carpetas temporales.')


if __name__ == '__main__':
    main()
