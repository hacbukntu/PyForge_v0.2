#!/usr/bin/env python3
"""Dependencias ST locales: copia explícita de fuentes y verificación SHA-256.

Uso: python3 scripts/bibliotecas.py agregar nombre /ruta/biblioteca --proyecto .
     python3 scripts/bibliotecas.py verificar --proyecto .
No usa red, no ejecuta código de instalación y nunca reemplaza una biblioteca.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import tempfile


def huellas(root):
    """Enumera todo archivo de una biblioteca y rechaza enlaces fuera del proyecto."""
    result = {}
    for path in sorted(root.rglob('*')):
        if path.is_symlink():
            raise ValueError(f'No se admiten enlaces simbólicos: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def main():
    """Despacha operaciones locales; el bloqueo conserva hashes de todas las fuentes copiadas."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('accion', choices=['agregar', 'verificar'])
    parser.add_argument('nombre', nargs='?')
    parser.add_argument('origen', nargs='?')
    parser.add_argument('--proyecto', default='.')
    args = parser.parse_args()
    project = Path(args.proyecto).resolve()
    if not (project / 'PyForge.toml').is_file():
        parser.error('El proyecto debe contener PyForge.toml')
    lock_path = project / 'Bibliotecas.lock.json'
    if lock_path.is_symlink():
        parser.error('El archivo de bloqueo no puede ser un enlace')
    lock = json.loads(lock_path.read_text()) if lock_path.exists() else {}
    libraries = project / 'bibliotecas'
    if libraries.is_symlink():
        parser.error('bibliotecas no puede ser un enlace')
    if args.accion == 'verificar':
        for name, hashes in lock.items():
            if not re.fullmatch(r'[A-Za-z0-9_-]+', name):
                parser.error('Nombre inválido en el bloqueo')
            directory = libraries / name
            if directory.is_symlink() or not directory.is_dir() or huellas(directory) != hashes:
                parser.error(f'Biblioteca modificada o ausente: {name}')
        actual = {p.name for p in libraries.iterdir()} if libraries.exists() else set()
        if actual != set(lock):
            parser.error('Hay bibliotecas sin registrar en el bloqueo')
        print(f'Bibliotecas verificadas: {len(lock)}')
        return
    if not args.nombre or not re.fullmatch(r'[A-Za-z0-9_-]+', args.nombre) or not args.origen:
        parser.error('Indica nombre válido y directorio de origen')
    source = Path(args.origen).resolve()
    if not (source / 'src/lib.st').is_file():
        parser.error('La biblioteca necesita src/lib.st')
    destination = libraries / args.nombre
    if destination.exists() or destination.is_symlink() or args.nombre in lock:
        parser.error('La biblioteca ya existe; no se sobrescribe')
    libraries.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=project, prefix='.biblioteca-') as temporary:
        staging = Path(temporary) / 'fuentes'
        staging.mkdir()
        for path in sorted((source / 'src').rglob('*.st')):
            if path.resolve() != path.absolute() or not path.is_file():
                parser.error('No se admiten fuentes mediante enlaces simbólicos')
            target = staging / path.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
        lock[args.nombre] = huellas(staging)
        pending = Path(temporary) / 'bloqueo.json'
        pending.write_text(json.dumps(lock, indent=2, sort_keys=True) + '\n')
        staging.rename(destination)
        try:
            pending.replace(lock_path)
        except OSError:
            shutil.rmtree(destination)
            raise
    print(f'Biblioteca agregada: @{args.nombre}')


if __name__ == '__main__':
    main()
