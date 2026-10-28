"""Selecciona fuentes y datos públicos de los proyectos; excluye resultados de ejecuciones."""
import json
from pathlib import Path


def project_files(root, collection="proyectos"):
    """Enumera archivos declarados en el catálogo y fuentes ST, con validación de rutas.

    Una salida generada por el usuario no entra al ZIP por aparecer en la carpeta:
    solo se distribuyen README, configuración, pruebas, fuentes y fixtures declarados.
    """
    # Ambas colecciones usan el mismo filtro de recursos públicos.
    if collection not in {'proyectos', 'proyectos_avanzados'}:
        raise ValueError('Colección desconocida')
    base = Path(root) / 'examples' / collection
    manifest = base / 'catalogo.json'
    if not manifest.is_file():
        return []
    selected = {manifest, base / 'README.md'}
    for project in json.loads(manifest.read_text(encoding='utf-8')):
        folder = (base / project['folder']).resolve()
        if not folder.is_relative_to(base.resolve()):
            raise ValueError('Carpeta de proyecto fuera del catálogo')
        for name in ['README.md', 'PyForge.toml', project['stdin'], project['stdout'], *project['fixtures']]:
            path = (folder / name).resolve()
            if not path.is_relative_to(folder) or not path.is_file():
                raise ValueError(f'Archivo de proyecto inválido: {name}')
            selected.add(path)
        selected.update(p.resolve() for p in (folder/'src').rglob('*.st') if p.is_file() and not p.is_symlink())
    return sorted(selected)
