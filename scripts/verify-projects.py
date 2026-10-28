#!/usr/bin/env python3
"""Comprueba los proyectos ST en copias temporales, con y sin optimización.

La entrada, salida y archivos esperados pertenecen al catálogo. Nunca se ejecuta
sobre las carpetas originales ni se sobrescriben sus datos de demostración.
"""
import argparse
import json
from pathlib import Path
import resource
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / 'examples/proyectos'


def limits():
    """Acota CPU y salida de cada programa de prueba; no altera el compilador."""
    resource.setrlimit(resource.RLIMIT_CPU, (4, 4))
    resource.setrlimit(resource.RLIMIT_FSIZE, (4 * 1024 * 1024, 4 * 1024 * 1024))


def within(root, relative):
    """Exige rutas relativas del catálogo que permanezcan dentro de su carpeta."""
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError(f'Ruta fuera del proyecto: {relative}')
    return path


def verify(project, compiler, optimize):
    """Compila con LLVM, ejecuta el caso y compara salida y artefactos."""
    source = within(CATALOG, project['folder'])
    with tempfile.TemporaryDirectory(prefix='st-proyecto-') as directory:
        work = Path(directory)
        shutil.copytree(source / 'src', work / 'src')
        for name in project['fixtures']:
            target = within(work, name)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(within(source, name), target)
        command = [str(compiler), 'build', str(within(work, project['entry'])), '-o', str(work / 'programa')]
        if not optimize:
            command.append('--no-opt')
        result = subprocess.run(command, capture_output=True, timeout=30)
        if result.returncode:
            raise ValueError(result.stderr.decode(errors='replace').strip())
        with within(source, project['stdin']).open('rb') as stdin, (work/'stdout').open('w+b') as stdout, (work/'stderr').open('w+b') as stderr:
            result = subprocess.run([str(work/'programa')], cwd=work, stdin=stdin, stdout=stdout, stderr=stderr, timeout=6, preexec_fn=limits)
            stdout.seek(0); actual = stdout.read()
            stderr.seek(0); error = stderr.read()
        if result.returncode or error:
            raise ValueError(f'Ejecución terminó con {result.returncode}: {error.decode(errors="replace")[:600]}')
        expected = within(source, project['stdout']).read_bytes()
        if actual != expected:
            raise ValueError(f'Salida distinta. Esperada: {expected[:600]!r}; obtenida: {actual[:600]!r}')
        for name, content in project['artifacts'].items():
            target = within(work, name)
            if not target.is_file() or target.read_bytes() != content.encode('utf-8'):
                raise ValueError(f'Archivo generado incorrecto: {name}')


def main():
    """Permite filtrar por prefijo; el modo normal comprueba las dos optimizaciones."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=ROOT/'target/release/pyforgec')
    parser.add_argument('--project', default='', help='Prefijo de carpeta, por ejemplo 01_ o 70_.')
    args = parser.parse_args()
    compiler = args.compiler.resolve()
    if not compiler.is_file():
        parser.error('No se encontró pyforgec; ejecuta cargo build --release o indica --compiler.')
    all_projects = json.loads((CATALOG/'catalogo.json').read_text(encoding='utf-8'))
    projects = [p for p in all_projects if p['folder'].startswith(args.project)]
    if not projects:
        parser.error('No hay proyectos que coincidan con el prefijo.')
    failures = []
    for project in projects:
        try:
            for optimize in (True, False):
                verify(project, compiler, optimize)
        except (OSError, ValueError, subprocess.TimeoutExpired) as error:
            failures.append(project['folder'])
            print(f'ERROR {project["folder"]}: {error}', flush=True)
    print(f'Proyectos correctos: {len(projects)-len(failures)}/{len(projects)}; '
          'LLVM, dos modos de optimización.', flush=True)
    raise SystemExit(bool(failures))


if __name__ == '__main__':
    main()
