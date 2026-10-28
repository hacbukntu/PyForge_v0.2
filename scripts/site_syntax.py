"""Colores del código publicado: solo analiza texto, nunca lo ejecuta."""
import html
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'ide'))
from language import vocabulary
import python_language

ST_TOKENS = re.compile(r'/\*[\s\S]*?(?:\*/|$)|"""[\s\S]*?(?:"""|$)|//[^\n]*|#[^\n]*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b|\b[A-Za-z_][A-Za-z_0-9]*\b')


def span(text, category):
    """Escapa el contenido antes de envolverlo en una clase de color controlada."""
    escaped = html.escape(text)
    return f'<span class="tok-{category}">{escaped}</span>' if category else escaped


def highlight(source, language='st'):
    """Colorea ST desde KEYWORDS y Python con el analizador léxico del IDE.

    Las etiquetas st-module-* del manual también representan código ST. Los
    bloques de texto solo se colorean si parecen ejemplos ST con delimitadores.
    """
    if language in ('py', 'python'):
        return ''.join(span(p['text'], p['category']) for p in python_language.highlight(source)['pieces'])
    words = {entry['word']: entry for entry in vocabulary()['keywords']}
    if language == 'text' and (';' in source or '{' in source):
        if any(re.search(r'\b' + re.escape(word) + r'\b', source) for word in words):
            language = 'st'
    if not language.startswith('st'):
        return html.escape(source)
    result, last = [], 0
    for match in ST_TOKENS.finditer(source):
        result.append(html.escape(source[last:match.start()]))
        token = match[0]
        if token.startswith(('/*', '"""', '//', '#')):
            category = 'comment'
        elif token.startswith(('"', "'")):
            category = 'string'
        elif token[0].isdigit():
            category = 'number'
        elif token in words:
            category = words[token]['category']
            if re.match(r'\s*\(', source[match.end():]) and words[token]['kind'] not in {'Si','Mientras','Para','Coincidir'} and category == 'keyword':
                category = 'function'
        elif re.match(r'\s*\(', source[match.end():]):
            category = 'function'
        else:
            category = 'variable'
        result.append(span(token, category))
        last = match.end()
    result.append(html.escape(source[last:]))
    return ''.join(result)
