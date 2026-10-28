"""Vista acotada de SQLite: nunca abre una base de datos para escribirla."""
import json
import sqlite3
import time


def preview(path):
    """Muestra esquema y hasta 20 filas por tabla ordinaria, con límites de recursos.

    Los datos se representan como comentarios JSON junto al SQL. No se ejecutan
    vistas, disparadores ni tablas virtuales. El URI mode=ro impide crear/escribir
    la base; no se usa immutable porque podría ignorar cambios presentes en WAL.
    """
    if path.stat().st_size > 64 * 1024 * 1024:
        raise ValueError('La vista SQLite admite bases de hasta 64 MB.')
    with path.open('rb') as source:
        if source.read(16) != b'SQLite format 3\x00':
            raise ValueError('Este .db no es una base SQLite compatible.')
    connection = sqlite3.connect(path.as_uri() + '?mode=ro', uri=True, timeout=1)
    try:
        connection.execute('PRAGMA query_only=ON')
        connection.execute('PRAGMA trusted_schema=OFF')
        connection.setlimit(sqlite3.SQLITE_LIMIT_LENGTH, 1_000_000)
        deadline = time.monotonic() + 2
        connection.set_progress_handler(lambda: int(time.monotonic() > deadline), 1000)
        lines = ['-- SQLite · solo lectura', '-- Hasta 30 tablas y 20 filas por tabla. Valores truncados a 200 caracteres.']
        tables = connection.execute("SELECT name, sql FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name LIMIT 31").fetchall()
        for name, schema in tables[:30]:
            lines.extend(['', (schema or '')[:12000] + ';'])
            if not schema or 'VIRTUAL TABLE' in schema.upper():
                lines.append('-- Tabla virtual: no se consultan sus datos.'); continue
            quoted = '"' + name.replace('"', '""') + '"'
            cursor = connection.execute('SELECT * FROM ' + quoted + ' LIMIT 21')
            lines.append('-- Columnas: ' + json.dumps([d[0] for d in cursor.description], ensure_ascii=False))
            rows = cursor.fetchall()
            for row in rows[:20]:
                values = [f'<binario: {len(v)} bytes>' if isinstance(v, bytes) else v[:200] if isinstance(v, str) else v for v in row]
                lines.append('-- ' + json.dumps(values, ensure_ascii=False))
            if len(rows) > 20:
                lines.append('-- Hay más filas; vista limitada.')
            if sum(map(len, lines)) > 100000:
                lines.append('-- Límite de vista alcanzado.'); break
        if len(tables) > 30:
            lines.append('-- Hay más tablas; vista limitada.')
        return {'text': '\n'.join(lines) + '\n', 'readonly': True}
    except sqlite3.Error as error:
        raise ValueError('No se pudo consultar SQLite: ' + str(error)) from error
    finally:
        connection.close()
