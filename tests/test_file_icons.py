"""Verifica recursos reales del sistema sin instalar ni cambiar preferencias del usuario."""
import importlib.util
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('build_icons', ROOT / 'scripts/build-icons.py')
icons = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(icons)


class FileIcons(unittest.TestCase):
    """Iconos multirresolución, MIME y entrada de escritorio desde una única fuente."""
    def test_svg_to_multiresolution_ico(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            icons.build(ROOT / 'website/logo.svg', root)
            data = (root / 'st.ico').read_bytes()
            self.assertEqual(struct.unpack_from('<HHH', data), (0, 1, 7))
            for index, size in enumerate(icons.SIZES):
                width, height, _, _, planes, bits, length, offset = struct.unpack_from('<BBBBHHII', data, 6 + index * 16)
                self.assertEqual((width or 256, height or 256), (size, size))
                png = data[offset:offset + length]
                self.assertEqual(png, (root / f'st-{size}.png').read_bytes())
                self.assertEqual(png[:8], b'\x89PNG\r\n\x1a\n')
                self.assertEqual(struct.unpack('>II', png[16:24]), (size, size))
            changed = root / 'logo.svg'
            changed.write_text((ROOT / 'website/logo.svg').read_text().replace('#edee63', '#ff0000'))
            icons.build(changed, root / 'nuevo')
            self.assertNotEqual(data, (root / 'nuevo/st.ico').read_bytes())

    def test_linux_mime_database_and_desktop(self):
        namespace = {'m': 'http://www.freedesktop.org/standards/shared-mime-info'}
        doc = ET.parse(ROOT / 'packaging/linux/st.xml')
        definition = doc.find('m:mime-type', namespace)
        self.assertEqual(definition.attrib['type'], 'text/x-st')
        self.assertEqual(definition.find('m:glob', namespace).attrib['pattern'], '*.st')
        self.assertEqual(definition.find('m:icon', namespace).attrib['name'], 'text-x-st')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'packages').mkdir()
            shutil.copy2(ROOT / 'packaging/linux/st.xml', root / 'packages/pyforge-st.xml')
            subprocess.run(['update-mime-database', str(root)], check=True, capture_output=True)
            self.assertIn('text/x-st:*.st', (root / 'globs2').read_text())
        subprocess.run(['desktop-file-validate', str(ROOT / 'packaging/linux/pyforge-ide.desktop')], check=True)

    def test_windows_preserves_other_associations(self):
        """Comprobación estática del script; no sustituye una prueba de registro en Windows."""
        script = (ROOT / 'packaging/windows/associations.ps1').read_text()
        self.assertIn('IsNullOrEmpty($current)', script)
        self.assertIn('OpenWithProgids', script)
        self.assertNotIn('Registry]::LocalMachine', script)
        self.assertIn("$extension.GetValue('', '') -eq $programId", script)
        self.assertIn('SHChangeNotify', script)
        self.assertNotIn('Set-ItemProperty', script)


if __name__ == '__main__':
    unittest.main()
