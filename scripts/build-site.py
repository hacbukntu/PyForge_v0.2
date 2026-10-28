#!/usr/bin/env python3
"""Genera una web estática desde el manual, ejemplos y paquetes verificados.

No ejecuta ejemplos ni interpreta HTML de la documentación. --watch vuelve a
publicar el directorio generado cuando cambian sus fuentes. No requiere Node.
"""
import argparse
import hashlib
import html
import json
from pathlib import Path
import re
import shutil
import tempfile
import time
import zipfile
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from site_syntax import highlight
from project_catalog import project_files

ROOT = Path(__file__).resolve().parents[1]

# El curso enseña a usar ST. Arquitectura del compilador, rutas internas,
# empaquetado y guías para modificar PyForge permanecen en la documentación de
# desarrollo, pero no distraen a quien está aprendiendo el lenguaje desde el IDE.
COURSE_EXCLUDED_SECTIONS = {14, 15, 16, 17, 23, 32, 33, 36, 37, 39}
COURSE_TECHNICAL_SUBSECTIONS = re.compile(
    r'implementaci[oó]n|c[oó]mo est[aá]n conectad|d[oó]nde (?:ampliar|extender)|'
    r'agregar funciones y m[oó]dulos|comprobaci[oó]n de esta ampliaci[oó]n|'
    r'verificaci[oó]n y distribuci[oó]n|ejemplos y mantenimiento|'
    r'archivos de implementaci[oó]n|construir el compilador|'
    r'compatibilidad privada|agregar sintaxis', re.I)


def course_source(source):
    """Devuelve solo lecciones de uso y ejemplos, sin capítulos internos.

    Los títulos no numerados se conservan para que esta función también sirva a
    documentos pequeños y a las pruebas del renderizador local.
    """
    pieces = re.split(r'(^## .+$)', source, flags=re.M)
    # El encabezado original enlaza arquitectura y ampliación del repositorio.
    # La vista Curso lo reemplaza por una bienvenida centrada en aprender y
    # practicar. El manual fuente completo permanece disponible en docs/.
    if re.search(r'^##\s+\d+\.', source, flags=re.M):
        output = [
            '# Curso completo de ST\n\n'
            'Aprende el lenguaje desde la instalación hasta proyectos completos. '
            'Cada lección presenta sintaxis, comportamiento y ejemplos prácticos.\n\n'
        ]
    else:
        output = [pieces[0]]
    for index in range(1, len(pieces), 2):
        heading = pieces[index]
        body = pieces[index + 1] if index + 1 < len(pieces) else ''
        match = re.match(r'^##\s+(\d+)\.', heading)
        if match and int(match.group(1)) in COURSE_EXCLUDED_SECTIONS:
            continue
        subsections = re.split(r'(^### .+$)', body, flags=re.M)
        cleaned = [subsections[0]]
        for child in range(1, len(subsections), 2):
            subtitle = subsections[child]
            content = subsections[child + 1] if child + 1 < len(subsections) else ''
            if COURSE_TECHNICAL_SUBSECTIONS.search(subtitle):
                continue
            cleaned.extend((subtitle, content))
        output.extend((heading, ''.join(cleaned)))
    return ''.join(output)


def inline(value):
    """Escapa texto y admite énfasis/código; los enlaces conservan su etiqueta legible."""
    value = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', value)
    value = html.escape(value)
    value = re.sub(r'`([^`]+)`', r'<code>\1</code>', value)
    value = re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', value)
    return value


def markdown(text):
    """Renderiza el subconjunto usado por el manual, escapando siempre HTML de origen."""
    out, paragraph, code = [], [], None
    table = False
    code_language = "text"
    def flush():
        if paragraph:
            out.append('<p>' + inline(' '.join(paragraph)) + '</p>')
            paragraph.clear()
    for line in text.splitlines():
        if line.startswith('```'):
            flush()
            if code is None:
                code = []
                code_language = line[3:].strip().lower()
            else:
                out.append('<pre><code>' + highlight('\n'.join(code), code_language) + '</code></pre>')
                code = None
            continue
        if code is not None:
            code.append(line)
            continue
        if line.startswith('|'):
            flush()
            if not table:
                out.append('<div class="table-scroll"><table>')
                table = True
            cells = line.strip().strip('|').split('|')
            if not all(re.fullmatch(r'\s*:?-+:?\s*', c) for c in cells):
                out.append('<tr>' + ''.join('<td>' + inline(c.strip()) + '</td>' for c in cells) + '</tr>')
            continue
        if table:
            out.append('</table></div>')
            table = False
        match = re.match(r'^(#{1,6}) (.*)', line)
        if match:
            flush()
            level = max(2, min(4, len(match[1])))
            out.append(f'<h{level}>' + inline(match[2]) + f'</h{level}>')
        elif not line.strip():
            flush()
        elif re.match(r'^\s*(?:- |\d+\. )', line):
            flush()
            out.append('<p class="list-item">' + inline(line.strip()) + '</p>')
        else:
            paragraph.append(line)
    flush()
    if table:
        out.append('</table></div>')
    if code is not None:
        out.append('<pre><code>' + highlight('\n'.join(code), code_language) + '</code></pre>')
    return '\n'.join(out)


def public_website_files():
    """Publica recursos de presentación de la raíz, nunca backend ni datos."""
    return sorted(p for p in (ROOT / 'website').iterdir()
                  if p.is_file() and not p.is_symlink()
                  and p.suffix in {'.html', '.css', '.js', '.svg', '.png', '.ico'})


def public_examples():
    """Selecciona código de ejemplo, sin entornos, datos, resultados ni enlaces."""
    root = ROOT / 'examples'
    excluded = {'.venv', 'venv', 'env', '__pycache__', 'target', 'build', 'dist',
                'node_modules', 'data', 'static-build', 'site-packages'}
    return sorted(p for p in root.rglob('*')
                  if p.is_file() and p.suffix in {'.st', '.py'}
                  and not any(part.startswith('.') or part in excluded for part in p.relative_to(root).parts)
                  and p.resolve() == p.absolute())


def sources():
    """Selecciona entradas públicas explícitas, incluyendo paquetes de la última versión."""
    files = list((ROOT / 'src/stdlib').glob('*.st')) + [ROOT / 'docs/MANUAL_ST.md', ROOT / 'docs/INSTALACION.md', ROOT / 'src/st/lexer.rs', ROOT / 'ide/language.py', ROOT / 'ide/python_language.py', Path(__file__).resolve().with_name('site_syntax.py'), Path(__file__).resolve()]
    files += sorted((ROOT / 'docs').glob('*.md'))
    files += [ROOT / 'src/st/stdlib.rs']
    files += sorted(p for p in (ROOT / 'examples/integraciones').rglob('*') if p.is_file() and p.suffix in {'.md', '.st', '.toml'})
    files += project_files(ROOT)
    files += project_files(ROOT, "proyectos_avanzados")
    files += [Path(__file__).resolve().with_name('project_catalog.py')]
    files += public_website_files()
    files += public_examples()
    manifest = ROOT / 'dist/packages/latest.json'
    if manifest.exists():
        files.append(manifest)
    return files


def fingerprint():
    """Detecta cambios de contenido y nombres para regenerar también altas y bajas."""
    digest = hashlib.sha256()
    for path in sources():
        digest.update(str(path.relative_to(ROOT)).encode() + b'\0' + path.read_bytes())
    return digest.hexdigest()


def build():
    """Publica recursos, lecciones y descargas; verifica las sumas antes de enlazarlas."""
    output = ROOT / 'dist/site'
    output.mkdir(parents=True, exist_ok=True)
    manifest_path = ROOT / 'dist/packages/latest.json'
    if manifest_path.exists():
        manifest = json.loads(manifest_path.read_text())
        sha = manifest.get('sha256', {})
        for artifact in manifest.get('artifacts', []):
            binary = ROOT / 'dist/packages' / artifact
            if not binary.is_file():
                raise ValueError(f'huella faltante: {artifact}')
            expected = sha.get(artifact)
            actual = hashlib.sha256(binary.read_bytes()).hexdigest()
            if expected and actual != expected:
                raise ValueError(f'huella no coincide para {artifact}: {actual} != {expected}')
    with tempfile.TemporaryDirectory(prefix='st-web-') as temp:
        stage = Path(temp)
        for public_file in public_website_files():
            shutil.copy2(public_file, stage / public_file.name)
        manual = course_source((ROOT / 'docs/MANUAL_ST.md').read_text())
        sections = re.split(r'^## (.+)$', manual, flags=re.M)
        lessons = [{'id': 'leccion-' + hashlib.sha256(re.sub(r'^\d+\.\s*', '', sections[i]).encode()).hexdigest()[:12], 'title': sections[i],
                    'html': markdown(sections[i+1]), 'text': sections[i+1]}
                   for i in range(1, len(sections), 2)]
        examples = []
        for path in public_examples():
            if path.is_file() and path.suffix in {'.st', '.py'}:
                relative = path.relative_to(ROOT / 'examples').as_posix()
                target = stage / 'ejemplos' / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(path, target)
                examples.append({'name': relative, 'code': path.read_text(), 'url': 'ejemplos/' + relative, 'html': highlight(path.read_text(), path.suffix[1:])})
        version = 'Documentación y ejemplos publicados'
        downloads = []
        if manifest_path.exists():
            manifest = json.loads(manifest_path.read_text())
            downloads_dir = stage / 'descargas'
            downloads_dir.mkdir(exist_ok=True)
            for artifact in manifest.get('artifacts', []):
                source_binary = ROOT / 'dist/packages' / artifact
                if not source_binary.is_file():
                    continue
                target_binary = downloads_dir / artifact
                shutil.copy2(source_binary, target_binary)
                downloads.append({
                    'name': artifact,
                    'url': 'descargas/' + artifact,
                    'sha256': manifest.get('sha256', {}).get(artifact, hashlib.sha256(source_binary.read_bytes()).hexdigest()),
                })
        # El vocabulario se lee de la misma fuente que usa el IDE.
        import sys
        sys.path.insert(0, str(ROOT / 'ide'))
        from language import vocabulary
        words = vocabulary(ROOT / 'src/st/lexer.rs')
        data = {'lessons': lessons, 'examples': examples, 'downloads': downloads,
                'version': version, 'vocabulary': words, 'sourceHash': fingerprint()}
        (stage / 'curso.json').write_text(json.dumps(data, ensure_ascii=False))
        # Las guías y referencias del repositorio se distribuyen sin duplicados ni paquetes de descarga.
        (stage / 'docs').mkdir(exist_ok=True)
        for guide in (ROOT / 'docs').glob('*.md'):
            shutil.copy2(guide, stage / 'docs' / guide.name)
        shutil.copy2(ROOT / 'docs/MANUAL_ST.md', stage / 'MANUAL_ST.md')
        shutil.copy2(ROOT / 'docs/INSTALACION.md', stage / 'INSTALACION.md')
        # Solo se eliminan archivos previamente generados: preserva archivos ajenos.
        inventory = output / '.generated-files.json'
        old = set(json.loads(inventory.read_text())) if inventory.exists() else set()
        fresh = {str(p.relative_to(stage)) for p in stage.rglob('*') if p.is_file()}
        # Cada archivo se reemplaza completo; curso.json se publica al final.
        for path in sorted(stage.rglob('*'), key=lambda p: p.name == 'curso.json'):
            if path.is_file():
                target = output / path.relative_to(stage)
                target.parent.mkdir(parents=True, exist_ok=True)
                pending = target.with_name(target.name + '.tmp')
                shutil.copy2(path, pending)
                pending.replace(target)
        for name in old - fresh:
            path = (output / name).resolve()
            if not path.is_relative_to(output.resolve()):
                continue
            if path.is_dir():
                shutil.rmtree(path, ignore_errors=True)
            elif path.is_file():
                path.unlink()
        inventory.write_text(json.dumps(sorted(fresh)))
        # Archivo listo para entregar a un alojamiento estático, con index.html en la raíz.
        print(f'Web lista: {output} — {len(lessons)} lecciones, {len(examples)} ejemplos, {len(downloads)} paquetes descargables', flush=True)


def main():
    """Construye una vez o vigila fuentes públicas; nunca modifica el compilador."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    previous = None
    while True:
        current = fingerprint()
        if current != previous:
            try:
                build()
            except (OSError, ValueError, KeyError) as error:
                print(f'La web anterior se conserva: {error}', flush=True)
                if not args.watch:
                    raise SystemExit(1)
            previous = current
        if not args.watch:
            break
        time.sleep(2)


if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        print('\nVigilancia terminada.')
