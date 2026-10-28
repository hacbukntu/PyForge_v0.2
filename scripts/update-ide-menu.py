#!/usr/bin/env python3
"""Actualiza el menú de la instalación para el usuario actual, sin permisos de administrador."""
import argparse
import json
from pathlib import Path
import sys

# Funciona tanto desde las fuentes como desde /usr/share/pyforge/scripts.
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'ide'))
from start_menu import config_path, validate


def main():
    """Valida antes de sustituir el archivo y permite exportar la plantilla inicial."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archivo', type=Path, help='JSON con título, mensaje y opciones.')
    parser.add_argument('--plantilla', action='store_true', help='Crea una plantilla nueva en la ruta indicada.')
    args = parser.parse_args()
    if args.plantilla:
        source = Path(__file__).resolve().parents[1] / 'ide/menu.json'
        with args.archivo.open('x', encoding='utf-8') as output:
            output.write(source.read_text('utf-8'))
        print('Plantilla creada:', args.archivo)
        return
    if args.archivo.stat().st_size > 16384:
        parser.error('El menú supera 16 KiB.')
    try:
        data = validate(json.loads(args.archivo.read_text('utf-8')))
    except ValueError as error:
        parser.error(str(error))
    target = config_path()
    target.parent.mkdir(parents=True, exist_ok=True)
    import tempfile
    # Archivo temporal exclusivo; un fallo nunca deja el menú escrito parcialmente.
    with tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', dir=target.parent, delete=False) as output:
        temporary = Path(output.name)
        json.dump(data, output, ensure_ascii=False, indent=2)
        output.write('\n')
    try:
        temporary.replace(target)
    finally:
        temporary.unlink(missing_ok=True)
    print('Menú actualizado. Recarga la página de inicio:', target)


if __name__ == '__main__':
    main()
