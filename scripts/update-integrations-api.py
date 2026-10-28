#!/usr/bin/env python3
"""Regenera firmas y comentarios desde ST: documentación e IDE comparten la fuente."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
MODULES = ('integraciones', 'archivos', 'db', 'documentos', 'red', 'cifrado', 'comandos', 'sistema', 'vpn', 'gui')


def generate():
    """Extrae declaraciones públicas; los auxiliares no forman parte de la API."""
    lines = [
             'Generada desde `src/stdlib/*.st` por `scripts/update-integrations-api.py`.',
             'Las funciones que devuelven JSON lo indican en su comentario; siguen siendo cadenas ST.', '']
    for module in MODULES:
        lines.extend(['### ' + module, '', '| Firma | Descripción |', '|---|---|'])
        source = (ROOT / 'src/stdlib' / (module + '.st')).read_text('utf-8')
        pattern = r'((?://[^\n]*\n)+)funcion\s+([^\n{]+)\s*\{'
        for comment, signature in re.findall(pattern, source):
            if signature.startswith('aux_'):
                continue
            description = ' '.join(line.removeprefix('//').strip() for line in comment.splitlines())
            lines.append(f'| `{signature.strip()}` | {description.replace("|", "/")} |')
        lines.append('')
    # Conserva la guía escrita a mano y reemplaza únicamente el bloque generado.
    destination = ROOT / 'docs/INTEGRACIONES.md'
    source = destination.read_text(encoding='utf-8')
    start, end = '<!-- API-INTEGRACIONES:INICIO -->', '<!-- API-INTEGRACIONES:FIN -->'
    if source.count(start) != 1 or source.count(end) != 1:
        raise ValueError('Faltan los marcadores únicos de API en INTEGRACIONES.md')
    before, remainder = source.split(start, 1)
    _, after = remainder.split(end, 1)
    content = before + start + '\n' + '\n'.join(lines) + '\n' + end + after
    if content != source:
        destination.write_text(content, encoding='utf-8')


if __name__ == '__main__':
    generate()
