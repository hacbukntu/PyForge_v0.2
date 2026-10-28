#!/usr/bin/env python3
"""Puente ejecutado dentro de WSL; conserva argumentos sin evaluarlos como shell."""
import os
import ctypes.util
from pathlib import Path
import re
import shutil
import subprocess
import sys


def linux_path(argument):
    """Convierte rutas absolutas Windows; opciones y rutas relativas quedan intactas."""
    if re.match(r'^[A-Za-z]:[\\/]', argument) or argument.startswith('\\\\'):
        return subprocess.check_output(['wslpath', '-u', argument], text=True).rstrip('\n')
    return argument


def main():
    """Selecciona exclusivamente los tres comandos instalados y mantiene su código de salida."""
    root = Path(__file__).resolve().parent / 'payload/usr'
    name = sys.argv[1]
    args = [linux_path(arg) for arg in sys.argv[2:]]
    if name == 'pyforge-ide':
        os.environ['PYFORGE_PROJECT'] = os.getcwd()
        os.environ['PYFORGE_WINDOWS_BROWSER'] = '1'
        os.execv(sys.executable, [sys.executable, '-B', str(root / 'share/pyforge/ide/server.py'), *args,
                                '--compiler', str(root / 'bin/pyforgec')])
    if name not in ('pyforge', 'pyforgec'):
        raise SystemExit('Comando no permitido.')
    if not shutil.which('as') or not shutil.which('ld'):
        raise SystemExit('Falta binutils en WSL. Instala: sudo apt install binutils')
    # Los ejecutables distribuidos usan LLVM. El cargador necesita la biblioteca
    # durante el arranque antes de que PyForge pueda analizar un archivo;
    # este diagnóstico evita el mensaje poco claro del enlazador dinámico de WSL.
    if not ctypes.util.find_library('LLVM-21'):
        raise SystemExit('Falta LLVM 21 en WSL. Instala: sudo apt install libllvm21')
    executable = str(root / 'bin' / name)
    os.execv(executable, [name, *args])


if __name__ == '__main__':
    main()
