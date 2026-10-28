"""Integridad del catálogo y de los recursos distribuidos de los proyectos ST."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from project_catalog import project_files


class ProjectCatalogTests(unittest.TestCase):
    """Detecta proyectos incompletos antes de empaquetar o publicar la web."""
    def test_catalog_and_resources(self):
        """Exige 80 proyectos distintos, entrada ejecutable, instrucciones y pruebas reproducibles."""
        base = ROOT / 'examples/proyectos'
        catalog = json.loads((base/'catalogo.json').read_text())
        self.assertEqual(len(catalog), 80)
        self.assertEqual(len({p['folder'] for p in catalog}), 80)
        selected = set(project_files(ROOT))
        for project in catalog:
            folder = base / project['folder']
            for name in [project['entry'], project['stdin'], project['stdout'], 'README.md', 'PyForge.toml', *project['fixtures']]:
                self.assertIn(folder / name, selected)
            self.assertTrue((folder/project['entry']).read_text().startswith('//'))

    def test_only_declared_fixtures(self):
        """Una salida o archivo ajeno al catálogo no se incorpora a los paquetes de proyectos."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            base = root/'examples/proyectos'; folder=base/'01_demo'
            (folder/'src').mkdir(parents=True)
            for name in ['README.md','PyForge.toml','stdin.txt','stdout.txt','dato.log','src/main.st','resultado.html','secreto.txt']:
                (folder/name).write_text('demo')
            (base/'README.md').write_text('Catálogo')
            (base/'catalogo.json').write_text(json.dumps([{'folder':'01_demo','stdin':'stdin.txt','stdout':'stdout.txt','fixtures':['dato.log']}]))
            selected = project_files(root)
            self.assertIn(folder/'dato.log', selected)
            self.assertNotIn(folder/'resultado.html', selected)
            self.assertNotIn(folder/'secreto.txt', selected)

    def test_advanced_catalog(self):
        """Exige veinte proyectos completos y comprueba su selección para distribución."""
        base = ROOT / 'examples/proyectos_avanzados'
        catalog = json.loads((base / 'catalogo.json').read_text())
        self.assertEqual(len(catalog), 20)
        self.assertEqual(len({p['folder'] for p in catalog}), 20)
        self.assertEqual(sum(p['category'] == 'Juego' for p in catalog), 6)
        selected = set(project_files(ROOT, 'proyectos_avanzados'))
        for project in catalog:
            folder = base / project['folder']
            for name in [project['entry'], project['stdin'], project['stdout'], 'README.md', 'PyForge.toml', *project['fixtures']]:
                self.assertIn(folder / name, selected)
            self.assertTrue(json.loads((folder / project['stdout']).read_text())['contains'])

    def test_advanced_excludes_generated_data(self):
        """Un resultado JSON o una clave añadida localmente no se publica por accidente."""
        import shutil
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            base = root / 'examples/proyectos_avanzados'
            shutil.copytree(ROOT / 'examples/proyectos_avanzados', base)
            folder = base / '05_diario_cifrado'
            for name in ['resultado.json', 'secrets.toml', 'diario.stenc']:
                (folder / name).write_text('privado')
            selected = set(project_files(root, 'proyectos_avanzados'))
            for name in ['resultado.json', 'secrets.toml', 'diario.stenc']:
                self.assertNotIn(folder / name, selected)
