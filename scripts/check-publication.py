#!/usr/bin/env python3
"""Revisa archivos publicables sin imprimir valores secretos. No sustituye una revisión humana."""
import argparse
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from project_catalog import project_files

# Patrones de alta confianza. No se imprime la coincidencia ni el contenido.
PATTERNS = {
    'clave privada': rb'-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----',
    'token GitHub': rb'\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,})\b',
    'clave AWS': rb'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b',
    'token Slack': rb'\bxox[baprs]-[A-Za-z0-9-]{20,}\b',
    'credencial en URL': rb'https?://[^\s/@:]+:[^\s/@]{8,}@',
}


def git(*args):
    """Lee Git sin shell; nunca modifica índice, historial ni configuración."""
    return subprocess.check_output(['git', *args], cwd=ROOT)


def private_name(name):
    """Bloquea archivos sensibles incluso si alguien los añadió con git add -f."""
    path = Path(name)
    parts = path.parts
    if any(p in {'.codex', '.agents', '.st-ide-trash', '.ssh', '.aws', '.azure', '.gnupg', 'backups', 'target', 'dist', '.venv', '__pycache__'} or p.startswith('firefox-test-') for p in parts):
        return True
    leaf = path.name.lower()
    if leaf in {'.env.example', 'secrets.example.toml'}:
        return False
    return (leaf == '.env' or leaf.startswith(('.env.', 'credentials.', 'secrets.')) or
            leaf in {'id_rsa', 'id_ed25519', 'known_hosts', '.netrc', '.npmrc', '.pypirc'} or
            leaf.endswith(('.pem', '.key', '.p12', '.pfx', '.db', '.sqlite', '.sqlite3', '.db-wal', '.db-shm', '.stenc', '.stvault')))


def main():
    """Inspecciona candidatos o el índice real; las eliminaciones no necesitan lectura."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--staged', action='store_true', help='Revisa todos los blobs del índice que formarán el commit.')
    args = parser.parse_args()
    command = ['ls-files', '-z'] if args.staged else ['ls-files', '-z', '--cached', '--others', '--exclude-standard']
    names = sorted(set(git(*command).decode('utf-8').split('\0')) - {''})
    curated = {str(p.relative_to(ROOT)) for collection in ('proyectos', 'proyectos_avanzados') for p in project_files(ROOT, collection)}
    issues = []
    for name in names:
        path = ROOT / name
        if not args.staged and not path.exists():
            continue
        if private_name(name):
            issues.append((name, 'nombre o carpeta privada'))
            continue
        if name.startswith(('examples/proyectos/', 'examples/proyectos_avanzados/')) and name not in curated and not name.endswith('.gitignore'):
            issues.append((name, 'recurso no declarado en el catálogo de ejemplos'))
            continue
        if args.staged:
            size = int(git('cat-file', '-s', ':' + name))
        else:
            if path.is_symlink():
                issues.append((name, 'enlace simbólico: revisar destino antes de publicar')); continue
            size = path.stat().st_size
        if size > 2_000_000:
            issues.append((name, 'archivo mayor de 2 MB: revisar antes de publicar')); continue
        data = git('show', ':' + name) if args.staged else path.read_bytes()
        if data.startswith(b'\x7fELF'):
            issues.append((name, 'ejecutable compilado')); continue
        for kind, pattern in PATTERNS.items():
            if re.search(pattern, data):
                issues.append((name, kind))
    for name, kind in issues:
        print(f'REVISAR: {name} — {kind}')
    print(f'Revisados {len(names)} archivos; {len(issues)} hallazgos. No se muestran valores privados.')
    return bool(issues)


if __name__ == '__main__':
    raise SystemExit(main())
