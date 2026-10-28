#!/usr/bin/env python3
"""Permite publicar fuentes y fixtures declarados, excluyendo resultados locales nuevos."""
from pathlib import Path
import json


def main():
    """Regenera únicamente los .gitignore de las dos colecciones desde sus catálogos."""
    root = Path(__file__).resolve().parents[1]
    for name in ('proyectos', 'proyectos_avanzados'):
        base = root / 'examples' / name
        lines = ['# Generado con scripts/update-example-ignores.py. No incluye salidas locales.',
                 '*', '!*/', '!**/*.st', '!**/README.md', '!**/PyForge.toml', '!README.md', '!catalogo.json', '!.gitignore']
        for item in json.loads((base / 'catalogo.json').read_text()):
            for file in [item['stdin'], item['stdout'], *item['fixtures']]:
                path = Path(item['folder']) / file
                if path.is_absolute() or '..' in path.parts:
                    raise ValueError('Ruta de catálogo inválida')
                lines.append('!/' + path.as_posix())
        (base / '.gitignore').write_text('\n'.join(lines) + '\n')


if __name__ == '__main__':
    main()
