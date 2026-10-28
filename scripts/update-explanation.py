#!/usr/bin/env python3
"""Actualiza explicacion.md desde el manual del curso, conservando una única fuente."""
from pathlib import Path
import re


def main():
    """Copia todas las lecciones y adapta los enlaces a la ubicación del documento."""
    root = Path(__file__).resolve().parents[1]
    manual = (root / 'docs/MANUAL_ST.md').read_text(encoding='utf-8')
    document = manual.replace('# Manual unificado de ST y PyForge', '# Explicación del lenguaje ST y su compilador', 1)
    # Ajusta enlaces relativos desde docs/ a la raíz, incluidos fragmentos y ejemplos.
    def relocate(match):
        target = match[1]
        if target.startswith('../'):
            target = target[3:]
        elif re.match(r'(MANUAL_ST|INSTALACION|INTEGRACIONES|README)\.md(?:#|$)', target):
            target = 'docs/' + target
        return '](' + target + ')'
    document = re.sub(r'\]\(([^)]+)\)', relocate, document)
    notice = '<!-- Generado desde docs/MANUAL_ST.md. Para actualizar: python3 scripts/update-explanation.py -->\n\n'
    destination = root / 'explicacion.md'
    content = notice + document
    # No modificar la fecha del archivo si el contenido sigue siendo idéntico.
    if not destination.exists() or destination.read_text(encoding='utf-8') != content:
        destination.write_text(content, encoding='utf-8')


if __name__ == '__main__':
    main()
