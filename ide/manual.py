"""Presenta el manual local con el mismo renderizador y colores del curso web."""
import importlib.util
from pathlib import Path
import re
import sys

# Reutiliza el generador sin llamar a build(): leer el manual no construye ni ejecuta ST.
SCRIPTS = Path(__file__).resolve().parents[1] / 'scripts'
sys.path.insert(0, str(SCRIPTS))
_spec = importlib.util.spec_from_file_location('ide_course_renderer', SCRIPTS / 'build-site.py')
_renderer = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_renderer)


def course_source(source):
    """Comparte con la web la selección de lecciones destinada a estudiantes."""
    return _renderer.course_source(source)


def page(source):
    """Muestra el curso de uso, sin arquitectura ni mantenimiento del proyecto."""
    content = _renderer.markdown(course_source(source))
    sections = []

    def heading(match):
        """Numera encabezados sin convertir títulos del documento en atributos HTML."""
        level, title = match.groups()
        anchor = f'seccion-{len(sections) + 1}'
        sections.append((anchor, re.sub('<[^>]+>', '', title), level))
        return f'<h{level} id="{anchor}">{title}</h{level}>'

    content = re.sub(r'<h([2-4])>(.*?)</h\1>', heading, content)
    index = ''.join(f'<a href="#{anchor}" class="level-{level}">{title}</a>' for anchor, title, level in sections)
    return '''<!doctype html>
<html lang="es"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Curso de ST · PyForge</title><link rel="icon" href="/st.svg"><link rel="stylesheet" href="/manual.css"></head>
<body><header><a href="/inicio.html"><img src="/st.svg" alt="ST" width="32" height="32">Inicio</a>
<a href="/">Editor</a><a href="/manual-fuente.md" download="CURSO_ST.md">Descargar curso</a></header>
<div class="reading-layout"><aside><details open><summary>Lecciones del curso</summary><nav aria-label="Índice">''' + index + '''</nav></details></aside>
<main id="contenido"><p class="eyebrow">ST · CURSO Y EJEMPLOS</p>''' + content + '''</main></div></body></html>'''
