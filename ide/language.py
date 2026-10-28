"""Lee el diccionario Rust de ST sin ejecutar Rust ni mantener otra lista de palabras."""
import hashlib
import re
from pathlib import Path

DEFAULT_SOURCE = Path(__file__).resolve().parent.parent / 'src/st/lexer.rs'


def without_comments(source):
    """Quita comentarios Rust, incluso bloques anidados, conservando cadenas intactas."""
    out = []
    i = 0
    while i < len(source):
        if source.startswith('//', i):
            end = source.find('\n', i)
            i = len(source) if end < 0 else end
        elif source.startswith('/*', i):
            depth = 1
            i += 2
            while i < len(source) and depth:
                if source.startswith('/*', i): depth += 1; i += 2
                elif source.startswith('*/', i): depth -= 1; i += 2
                else: i += 1
            if depth: raise ValueError('Comentario Rust sin cerrar en KEYWORDS.')
            out.append(' ')
        elif source[i] == '"':
            start = i
            i += 1
            while i < len(source):
                if source[i] == '\\': i += 2
                elif source[i] == '"': i += 1; break
                else: i += 1
            out.append(source[start:i])
        else:
            out.append(source[i]); i += 1
    return ''.join(out)


def vocabulary(path=DEFAULT_SOURCE):
    """Devuelve textos, significados y categorías desde KEYWORDS en cada consulta.

    La forma admitida es la tabla literal ("palabra", Keyword::Variante).
    Un cambio inválido devuelve error para que el navegador conserve la última versión.
    Los significados desconocidos reciben color de palabra y autocompletado igualmente.
    """
    source = without_comments(Path(path).read_text())
    match = re.search(r'pub\s+const\s+KEYWORDS\s*:[^=]*=\s*&\[(.*?)\];', source, re.S)
    if not match: raise ValueError('No se encontró la tabla literal KEYWORDS en lexer.rs.')
    table = match.group(1)
    pattern = r'\(\s*"([A-Za-z_][A-Za-z_0-9]*)"\s*,\s*Keyword::([A-Za-z_][A-Za-z_0-9]*)\s*\)'
    entries = re.findall(pattern, table)
    if not entries or re.sub(pattern, '', table).replace(',', '').strip():
        raise ValueError('KEYWORDS contiene una entrada incompleta o no admitida.')
    words = set()
    result = []
    for word, kind in entries:
        if word in words: raise ValueError(f'Palabra reservada duplicada: {word}')
        words.add(word)
        category = 'type' if kind in {'Entero','Cadena','Flotante','Booleano','Nada','Lista','Diccionario','Tupla','Conjunto','Nulo','Mixto'} else 'boolean' if kind in {'Verdadero','Falso'} else 'keyword'
        result.append({'word': word, 'kind': kind, 'category': category})
    contextual_match = re.search(
        r'pub\s+const\s+CONTEXTUAL_WORDS\s*:[^=]*=\s*&\[(.*?)\];', source, re.S)
    if contextual_match:
        contextual_pattern = r'\(\s*"([A-Za-z_][A-Za-z_0-9]*)"\s*,\s*"(keyword|type)"\s*\)'
        contextual = re.findall(contextual_pattern, contextual_match.group(1))
        residue = re.sub(contextual_pattern, '', contextual_match.group(1)).replace(',', '').strip()
        if residue:
            raise ValueError('CONTEXTUAL_WORDS contiene una entrada no admitida.')
        for word, category in contextual:
            if word in words:
                raise ValueError(f'Palabra contextual duplicada: {word}')
            words.add(word)
            result.append({'word': word, 'kind': 'Contextual', 'category': category})
    modules = standard_modules(Path(path).parent.parent / 'stdlib')
    version = hashlib.sha256(repr((result, modules)).encode()).hexdigest()
    return {'version': version, 'keywords': result, 'modules': modules, 'builtins': ['tipo']}


def standard_modules(directory):
    """Lee firmas públicas de los mismos archivos ST que include_str! incrusta en el compilador.

    Elimina comentarios/cadenas antes de buscar declaraciones para no sugerir código de ejemplo.
    Los auxiliares aux_ no se exportan. No ejecuta las fuentes del lenguaje.
    """
    result = {}
    for path in sorted(Path(directory).glob('*.st')):
        source = without_comments(path.read_text(encoding='utf-8'))
        # Las cadenas no pueden contener declaraciones; sus contenidos se descartan.
        source = re.sub(r"\"(?:\\.|[^\"\\])*\"|'(?:\\.|[^'\\])*'", '""', source)
        functions = []
        for match in re.finditer(r'^\s*(?:publica\s+)?funcion\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(([^)]*)\)\s*->\s*([A-Za-z_][A-Za-z_0-9]*(?:\[[^]]+\])?)', source, re.M):
            name, parameters, returns = match.groups()
            if not name.startswith('aux_'):
                functions.append({'name': name, 'signature': f'{name}({parameters.strip()}) -> {returns}'})
        result[path.stem] = functions
    # Los alias oficiales también se leen del registro Rust; no hay lista paralela.
    registry = Path(directory).parent / 'st/stdlib.rs'
    if registry.is_file():
        for alias, filename in re.findall(r'"([A-Za-z_.]+)"\s*=>\s*include_str!\("\.\./stdlib/([A-Za-z_.]+)\.st"\)', registry.read_text()):
            if filename in result:
                result[alias] = result[filename]
    return result
