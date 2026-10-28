"""Reconocimiento léxico de Python mediante la biblioteca estándar del servidor.

No ejecuta ni importa el código editado. La versión de Python que inicia el IDE
establece el vocabulario y los tokens reconocidos, independientemente de PyForge.
"""
import builtins
import io
import keyword
import token
import tokenize


def vocabulary():
    """Expone palabras, palabras contextuales y nombres incorporados para sugerencias."""
    return {
        'keywords': sorted(set(keyword.kwlist + keyword.softkwlist)),
        'builtins': sorted(name for name in dir(builtins) if not name.startswith('_')),
    }


def highlight(source):
    """Devuelve fragmentos contiguos; conserva incluso código incompleto al escribir.

    tokenize reconoce números, comentarios, prefijos, cadenas multilínea y las
    construcciones disponibles en el Python anfitrión. Las palabras contextuales
    se destacan léxicamente; esto no constituye análisis de tipos ni validación.
    Los índices se aplican aquí, no en JavaScript, para respetar Unicode.
    """
    lines = source.splitlines(keepends=True)
    offsets = [0]
    for line in lines:
        offsets.append(offsets[-1] + len(line))
    pieces = []
    cursor = 0
    declaration = False
    words = set(keyword.kwlist + keyword.softkwlist)

    def position(location):
        """Convierte la coordenada del token en un desplazamiento del texto original."""
        row, column = location
        return min(len(source), offsets[min(row - 1, len(offsets) - 1)] + column)

    try:
        for item in tokenize.generate_tokens(io.StringIO(source).readline):
            start, end = position(item.start), position(item.end)
            if end <= start or start < cursor:
                continue
            category = ''
            kind = token.tok_name[item.type]
            if item.type == token.COMMENT:
                category = 'comment'
            elif item.type == token.STRING or kind.startswith(('FSTRING_', 'TSTRING_')):
                category = 'string'
            elif item.type == token.NUMBER:
                category = 'number'
            elif item.type == token.NAME:
                if item.string in {'True', 'False', 'None'}:
                    category = 'boolean'
                elif item.string in words:
                    category = 'keyword'
                elif declaration:
                    category = 'function'
                elif hasattr(builtins, item.string):
                    category = 'type'
                elif source[end:].lstrip().startswith('('):
                    category = 'function'
                else:
                    category = 'variable'
                declaration = item.string in {'def', 'class'}
            if start > cursor:
                pieces.append({'text': source[cursor:start], 'category': ''})
            pieces.append({'text': source[start:end], 'category': category})
            cursor = end
    except (tokenize.TokenError, IndentationError, SyntaxError):
        # Un bloque o literal sin terminar es normal mientras se edita: no se pierde texto.
        pass
    if cursor < len(source):
        pieces.append({'text': source[cursor:], 'category': ''})
    return {'pieces': pieces}
