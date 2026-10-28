#!/usr/bin/env python3
"""Convierte el SVG oficial a PNG/ICO al empaquetar; nunca redibuja el logo a mano.

Requiere GdkPixbuf con el cargador SVG y sus enlaces Python en la máquina que construye los paquetes. El equipo que
instala PyForge no necesita esta dependencia para mostrar los iconos del sistema.
"""
import argparse
from pathlib import Path
import struct

SIZES = (16, 24, 32, 48, 64, 128, 256)


def build(source, destination):
    """Renderiza con librsvg el mismo SVG a varias escalas y escribe un ICO con imágenes PNG."""
    try:
        import gi
        gi.require_version('GdkPixbuf', '2.0')
        from gi.repository import GdkPixbuf
    except (ImportError, ValueError) as error:
        raise RuntimeError('Construir iconos requiere python3-gi, gir1.2-gdkpixbuf-2.0 y librsvg2-common.') from error
    source_bytes = source.read_bytes()
    destination.mkdir(parents=True, exist_ok=True)
    images = []
    for size in SIZES:
        # La escala se fija antes de cargar: el renderizador rasteriza el vector a esa medida.
        loader = GdkPixbuf.PixbufLoader.new_with_type('svg')
        loader.set_size(size, size)
        loader.write(source_bytes)
        loader.close()
        pixbuf = loader.get_pixbuf()
        success, data = pixbuf.save_to_bufferv('png', [], [])
        if not success:
            raise RuntimeError('GdkPixbuf no pudo generar el icono PNG')
        data = bytes(data)
        (destination / f'st-{size}.png').write_bytes(data)
        images.append((size, data))
    # Windows admite PNG dentro de ICO. 0 representa 256 píxeles en el directorio ICO.
    offset = 6 + 16 * len(images)
    directory = bytearray(struct.pack('<HHH', 0, 1, len(images)))
    for size, data in images:
        directory.extend(struct.pack('<BBBBHHII', size % 256, size % 256, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    (destination / 'st.ico').write_bytes(bytes(directory) + b''.join(data for _, data in images))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    args = parser.parse_args()
    build(args.source, args.destination)
