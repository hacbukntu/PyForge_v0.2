#!/usr/bin/env python3
"""Construye paquetes desde una instantánea; --watch regenera al guardar cambios.

No instala en el sistema ni modifica PATH durante la construcción. El .deb coloca
los comandos en /usr/bin, que ya pertenece al PATH habitual de Linux.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import fcntl
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile
import time
import tomllib

# Las pruebas cargan este módulo por ruta; permite importar el selector vecino.
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_catalog import project_files

ROOT = Path(__file__).resolve().parent.parent


def inputs():
    """Selecciona fuentes y recursos; excluye perfiles de navegador, cachés y secretos."""
    for name in ('Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md', 'CHANGELOG.md', 'explicacion.md', 'PyForge.toml.example', 'config.example.toml', 'secrets.example.toml', '.env.example', '.gitignore', 'website/logo.svg'):
        yield ROOT / name
    if (ROOT / 'build.rs').is_file():
        yield ROOT / 'build.rs'
    # Los proyectos tienen una selección explícita para conservar fixtures y excluir resultados.
    yield from project_files(ROOT)
    yield from project_files(ROOT, "proyectos_avanzados")
    for directory in ('src', 'runtime', 'docs', 'examples', 'scripts', 'packaging', 'tests'):
        for path in sorted((ROOT / directory).rglob('*')):
            if any(path.is_relative_to(ROOT / 'examples' / name) for name in ('proyectos', 'proyectos_avanzados')):
                continue
            if path.is_file() and not path.is_symlink() and not set(path.relative_to(ROOT).parts) & {'__pycache__', 'target', '.venv', 'data', 'static-build'}:
                if path.suffix in {'.rs', '.s', '.md', '.st', '.py', '.toml', '.sh', '.nsi', '.ps1', '.cmd', '.json', '.xml', '.desktop'} or (any(path.is_relative_to(ROOT / subtree) for subtree in ('runtime/web_template', 'runtime/web_legacy', 'examples/web')) and path.suffix in {'.html', '.css', '.js', '.txt'}):
                    yield path
    crate = ROOT / 'crates/pyforge_django'
    if crate.exists():
        for name in ('Cargo.toml', 'Cargo.lock', 'README.md'):
            yield crate / name
        for directory in ('src', 'tests', 'include'):
            for path in sorted((crate / directory).rglob('*')):
                if path.is_file() and not path.is_symlink() and path.suffix in {'.rs', '.h'}:
                    yield path
    # El motor 3D conserva su lock independiente y se distribuye como componente opcional.
    engine = ROOT / 'crates/pyforge_engine3d'
    if engine.exists():
        for name in ('Cargo.toml', 'Cargo.lock', 'README.md'):
            yield engine / name
        # `assets` contiene solo recursos públicos incrustados por `include_bytes!`;
        # deben entrar en la instantánea o una compilación limpia no reproduciría
        # el mismo editor que el árbol de desarrollo.
        for directory in ('src', 'tests', 'examples', 'assets'):
            for path in sorted((engine / directory).rglob('*')):
                if path.is_file() and not path.is_symlink() and path.suffix in {'.rs', '.wgsl', '.png'}:
                    yield path
    # La extensión pública no incorpora preferencias privadas de .vscode.
    for path in sorted((ROOT / 'extensions/vscode-st').rglob('*')):
        if path.is_file() and not path.is_symlink() and path.suffix in {'.js', '.json', '.md', '.svg'}:
            yield path
    # Solo recursos del editor: nunca copiar perfiles ni sesiones locales.
    for path in sorted((ROOT / 'ide').iterdir()):
        if path.is_file() and not path.is_symlink() and path.suffix in {'.py', '.js', '.css', '.html', '.svg', '.md', '.json'}:
            yield path


def snapshot():
    """Lee bytes una vez y calcula una huella que detecta altas, bajas y modificaciones."""
    files = {str(path.relative_to(ROOT)): path.read_bytes() for path in inputs()}
    digest = hashlib.sha256()
    for name, data in sorted(files.items()):
        digest.update(name.encode() + b'\0' + hashlib.sha256(data).digest())
    return files, digest.hexdigest()


def run(*args, cwd=None, env=None):
    """Ejecuta argumentos sin shell; un fallo impide publicar un paquete nuevo."""
    subprocess.run(args, cwd=cwd, env=env, check=True)


def copy_tree(source, destination):
    """Copia únicamente la instantánea seleccionada, sin seguir el árbol de trabajo vivo."""
    shutil.copytree(source, destination, dirs_exist_ok=True, ignore=shutil.ignore_patterns('__pycache__', '*.pyc'))


def libc_requirement(binary):
    """Extrae la versión GLIBC realmente requerida, sin prometer compatibilidad anterior."""
    output = subprocess.check_output(['readelf', '--version-info', str(binary)], text=True)
    versions = [tuple(map(int, v.split('.'))) for v in re.findall(r'GLIBC_(\d+(?:\.\d+)+)', output)]
    return '.'.join(map(str, max(versions))) if versions else '2.17'


def llvm_environment():
    """Localiza LLVM 21 sin instalar nada y devuelve el entorno exigido por llvm-sys.

    Acepta `LLVM_SYS_211_PREFIX`, una instalación del sistema con `llvm-config-21`
    o la copia de desarrollo extraída en `target/tools/llvm-21`. El paquete final
    depende de `libllvm21`; las cabeceras solo se necesitan para reconstruirlo.
    """
    candidates = []
    if os.environ.get('LLVM_SYS_211_PREFIX'):
        candidates.append(Path(os.environ['LLVM_SYS_211_PREFIX']))
    config = shutil.which('llvm-config-21') or shutil.which('llvm-config')
    if config:
        prefix = subprocess.check_output([config, '--prefix'], text=True).strip()
        if prefix:
            candidates.append(Path(prefix))
    candidates.append(ROOT / 'target/tools/llvm-21/usr/lib/llvm-21')
    for prefix in candidates:
        tool = prefix / 'bin/llvm-config'
        if not tool.is_file():
            continue
        version = subprocess.check_output([str(tool), '--version'], text=True).strip()
        if version.split('.', 1)[0] != '21':
            continue
        environment = os.environ | {'LLVM_SYS_211_PREFIX': str(prefix.resolve())}
        # Los mensajes de pánico de Rust pueden conservar rutas de sus fuentes.
        # Remapear el hogar y el checkout evita publicar nombres locales del equipo.
        mappings = (
            f'--remap-path-prefix={Path.home()}=/build/home',
            f'--remap-path-prefix={ROOT}=/build/pyforge',
        )
        previous = environment.get('RUSTFLAGS', '').strip()
        environment['RUSTFLAGS'] = ' '.join((*([previous] if previous else []), *mappings))
        return environment, version
    raise RuntimeError(
        'Falta LLVM 21 de desarrollo. Instala llvm-21-dev o define LLVM_SYS_211_PREFIX.'
    )


def build(files, digest, formats):
    """Compila/prueba una copia congelada y publica paquetes completos con reemplazo atómico."""
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise RuntimeError('La construcción actual requiere Linux x86-64.')
    for tool in ('cargo', 'dpkg-deb', 'readelf', 'desktop-file-validate', 'update-mime-database'):
        if not shutil.which(tool):
            raise RuntimeError(f'Falta la herramienta: {tool}')
    output = ROOT / 'dist' / 'packages'
    output.mkdir(parents=True, exist_ok=True)
    version = tomllib.loads(files['Cargo.toml'].decode())['package']['version']
    stamp = datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')
    package_version = f'{version}+{stamp}.{digest[:12]}'
    llvm_env, llvm_version = llvm_environment()
    with tempfile.TemporaryDirectory(prefix='pyforge-package-') as temporary:
        work = Path(temporary)
        source = work / 'source'
        for name, data in files.items():
            path = source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        # Cache separada: compilar una instantánea no sobrescribe los binarios de desarrollo.
        target = ROOT / 'target' / 'packages'
        # LLVM es obligatorio: una sola matriz prueba exactamente el compilador
        # que se distribuye y evita conservar artefactos del backend retirado.
        run('cargo', 'test', '--locked',
            '--target-dir', str(target), cwd=source, env=llvm_env)
        run('cargo', 'build', '--release', '--locked',
            '--target-dir', str(target), cwd=source, env=llvm_env)
        engine_target = ROOT / 'target' / 'packages-engine'
        run('cargo', 'build', '--release', '--locked', '--features', 'editor', '--bins',
            '--target-dir', str(engine_target), cwd=source / 'crates/pyforge_engine3d',
            env=llvm_env)
        # Las pruebas del IDE buscan el binario bajo target/release del repositorio.
        (source / 'target/release').mkdir(parents=True)
        for name in ('pyforge', 'pyforgec'):
            (source / 'target/release' / name).symlink_to(target / 'release' / name)
        run('python3', '-m', 'unittest', 'discover', '-s', 'tests', '-p', 'test_*.py', '-q', cwd=source)
        run('python3', '-m', 'unittest', 'discover', '-s', 'ide', '-q', cwd=source)
        run('python3', 'scripts/verify-projects.py', '--compiler', str(target / 'release/pyforgec'), cwd=source)
        # Los cuatro juegos visuales comparten protocolo; completar sus reglas evita
        # publicar una ventana que compile pero no pueda ganar o reiniciarse.
        run('python3', 'examples/games/verify_visual_games.py',
            '--compiler', str(target / 'release/pyforgec'), cwd=source)
        # Una única fuente gráfica actualiza Linux y Windows al reconstruir.
        icons = work / 'icons'
        run('python3', 'scripts/build-icons.py', str(source / 'website/logo.svg'), str(icons), cwd=source)
        stage = work / 'root'
        binary_dir = stage / 'usr/bin'
        binary_dir.mkdir(parents=True)
        for name in ('pyforge', 'pyforgec'):
            shutil.copy2(target / 'release' / name, binary_dir / name)
        for name in ('pyforge-engine3d', 'pyforge-editor3d', 'pyforge-game3d'):
            shutil.copy2(engine_target / 'release' / name, binary_dir / name)
        resources = stage / 'usr/share/pyforge'
        resources.mkdir(parents=True)
        for name in ('ide', 'src', 'runtime', 'docs', 'examples', 'crates', 'extensions'):
            copy_tree(source / name, resources / name)
        # La instantánea recrea archivos con modo regular; restaura ejecución solo
        # para los lanzadores públicos de ejemplos, nunca para fuentes `.st`.
        for script in (resources / 'examples/games').rglob('*.sh'):
            script.chmod(0o755)
        # Distribuye el personalizador para actualizar el menú instalado sin sudo.
        (resources / 'scripts').mkdir()
        # El manual del IDE comparte el renderizador del curso y sus dependencias locales.
        for script in ('update-ide-menu.py', 'build-site.py', 'site_syntax.py', 'project_catalog.py', 'bibliotecas.py'):
            shutil.copy2(source / 'scripts' / script, resources / 'scripts' / script)
        for name in ('LICENSE', 'README.md', 'explicacion.md', 'config.example.toml', 'secrets.example.toml', '.env.example'):
            shutil.copy2(source / name, resources / name)
        launcher = binary_dir / 'pyforge-ide'
        launcher.write_text('#!/bin/sh\n# Abre el proyecto actual con los recursos y compilador instalados.\nexport PYFORGE_PROJECT="$PWD"\nexec python3 -B /usr/share/pyforge/ide/server.py "$@" --compiler /usr/bin/pyforgec\n')
        launcher.chmod(0o755)
        # MIME/iconos Linux: no cambia preferencias personales con xdg-mime default.
        mime = stage / 'usr/share/mime/packages'
        applications = stage / 'usr/share/applications'
        mime.mkdir(parents=True)
        applications.mkdir(parents=True)
        shutil.copy2(source / 'packaging/linux/st.xml', mime / 'pyforge-st.xml')
        shutil.copy2(source / 'packaging/linux/pyforge-ide.desktop', applications / 'pyforge-ide.desktop')
        for kind, name in [('mimetypes', 'text-x-st'), ('apps', 'pyforge')]:
            scalable = stage / 'usr/share/icons/hicolor/scalable' / kind
            scalable.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source / 'website/logo.svg', scalable / (name + '.svg'))
            for size in (16, 24, 32, 48, 64, 128, 256):
                directory = stage / f'usr/share/icons/hicolor/{size}x{size}' / kind
                directory.mkdir(parents=True, exist_ok=True)
                shutil.copy2(icons / f'st-{size}.png', directory / (name + '.png'))
        metadata = {'version': package_version, 'source_sha256': digest,
                    'files': sorted(files), 'platform': 'linux-amd64',
                    'backend': 'llvm', 'llvm': llvm_version}
        (resources / 'build.json').write_text(json.dumps(metadata, indent=2) + '\n')
        control = stage / 'DEBIAN'
        control.mkdir()
        for hook in ('postinst', 'postrm'):
            shutil.copy2(source / 'packaging/linux/update-caches.sh', control / hook)
            (control / hook).chmod(0o755)
        glibc = libc_requirement(binary_dir / 'pyforgec')
        (control / 'control').write_text(f'''Package: pyforge
Version: {package_version}
Architecture: amd64
Maintainer: Proyecto PyForge <pyforge@localhost>
Section: devel
Priority: optional
Depends: libc6 (>= {glibc}), libgcc-s1, libllvm21 (>= 1:21.1), binutils, python3 (>= 3.11), python3-tk, python3-cryptography, shared-mime-info, hicolor-icon-theme
Recommends: desktop-file-utils, libvulkan1, mesa-vulkan-drivers, libegl1, libx11-6, libwayland-client0, pipewire-bin, alsa-utils
Description: Compilador nativo ST y subconjunto Python con IDE local
 Incluye comandos pyforge, pyforgec y pyforge-ide, ejemplos y manual en espanol.
''')
        deb = work / f'pyforge_{package_version}_amd64.deb'
        run('dpkg-deb', '--root-owner-group', '--build', str(stage), str(deb))
        # Valida estructura y ejecutables extraídos antes de publicar.
        extracted = work / 'verify'
        run('dpkg-deb', '--extract', str(deb), str(extracted))
        run(str(extracted / 'usr/bin/pyforgec'), 'check', str(extracted / 'usr/share/pyforge/examples/clases.st'))
        run(str(extracted / 'usr/bin/pyforge'), '--version')
        run(str(extracted / 'usr/bin/pyforge-engine3d'), '--version')
        run(str(extracted / 'usr/bin/pyforge-game3d'), '--version')
        # Comprueba el script desde el .deb extraído sin modificar la configuración personal.
        run('python3', str(extracted / 'usr/share/pyforge/scripts/update-ide-menu.py'), '--plantilla', str(work / 'menu-prueba.json'))
        artifacts = [deb]
        if formats == 'all':
            nsis = os.environ.get('MAKENSIS') or shutil.which('makensis')
            nsis_environment = None
            # Caché opcional de herramientas: permite reconstruir aquí sin instalar NSIS globalmente.
            cached_nsis = ROOT / 'target/tools/nsis/usr'
            if nsis is None and (cached_nsis / 'bin/makensis').is_file():
                nsis = str(cached_nsis / 'bin/makensis')
                nsis_environment = os.environ | {'NSISDIR': str(cached_nsis / 'share/nsis')}
            if not nsis or not shutil.which(nsis):
                raise RuntimeError('Falta makensis (paquete nsis). Puedes construir solo Linux con --format deb.')
            windows = work / 'windows'
            windows.mkdir()
            copy_tree(stage / 'usr', windows / 'payload/usr')
            for name in ('launch.py', 'launch.ps1', 'environment.ps1', 'associations.ps1'):
                if name.endswith('.ps1'):
                    # PowerShell 5.1 necesita BOM para conservar los mensajes en español.
                    (windows / name).write_text((source / 'packaging/windows' / name).read_text(encoding='utf-8-sig'), encoding='utf-8-sig')
                else:
                    shutil.copy2(source / 'packaging/windows' / name, windows / name)
            shutil.copy2(source / 'docs/INSTALACION.md', windows / 'INSTALACION.md')
            shutil.copy2(icons / 'st.ico', windows / 'st.ico')
            (windows / 'bin').mkdir()
            for command in ('pyforge', 'pyforgec', 'pyforge-ide'):
                (windows / 'bin' / (command + '.cmd')).write_bytes((
                    '@echo off\r\nrem Puente a WSL; conserva la carpeta actual del usuario.\r\n'
                    f'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\\launch.ps1" {command} %*\r\n'
                    'exit /b %errorlevel%\r\n').encode())
            exe = work / f'PyForge-{package_version}-windows-wsl-setup.exe'
            run(nsis, '-WX', '-V2', f'-DOUTPUT={exe}', f'-DPAYLOAD={windows}',
                f'-DVERSION={package_version}', f'-DGLIBC={glibc}',
                str(source / 'packaging/windows/installer.nsi'), env=nsis_environment)
            if exe.read_bytes()[:2] != b'MZ':
                raise RuntimeError('NSIS no generó un ejecutable Windows válido.')
            artifacts.append(exe)
        for artifact in artifacts:
            destination = output / artifact.name
            pending = output / (artifact.name + '.tmp')
            shutil.copy2(artifact, pending)
            os.replace(pending, destination)
            print(f'Paquete listo: {destination}', flush=True)
        manifest = metadata | {'artifacts': [p.name for p in artifacts], 'sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in artifacts}}
        pending_manifest = output / 'latest.json.tmp'
        pending_manifest.write_text(json.dumps(manifest, indent=2) + '\n')
        os.replace(pending_manifest, output / 'latest.json')


def main():
    """Una construcción por defecto; el vigilante espera estabilidad antes de recompilar."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--format', choices=('deb', 'all'), default='all')
    parser.add_argument('--watch', action='store_true', help='Regenera paquetes tras cambios estables durante dos segundos.')
    args = parser.parse_args()
    previous = None
    while True:
        files, digest = snapshot()
        if digest != previous:
            if args.watch:
                time.sleep(2)
                fresh, fresh_digest = snapshot()
                if fresh_digest != digest:
                    continue
                files = fresh
            try:
                # Evita mezclar binarios de dos vigilantes que compartan caché de Cargo.
                (ROOT / 'target').mkdir(exist_ok=True)
                with (ROOT / 'target/package.lock').open('a') as lock:
                    fcntl.flock(lock, fcntl.LOCK_EX)
                    build(files, digest, args.format)
            except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
                print(f'No se publicó el paquete: {error}', flush=True)
                if not args.watch:
                    raise SystemExit(1)
            previous = digest
        if not args.watch:
            return
        time.sleep(1)


if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        print('\nVigilancia terminada.')
