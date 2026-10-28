"""Selector nativo aislado: Tk vive en su propio proceso e hilo principal."""
import json
from pathlib import Path
import re
import subprocess
import sys


def choose(initial, create=False):
    """Espera la selección sin ejecutar shell; cancelar conserva el proyecto abierto."""
    result = subprocess.run([sys.executable, '-I', str(Path(__file__).resolve()),
                             str(initial), create if isinstance(create,str) else 'crear' if create else 'abrir'],
                            capture_output=True, text=True, timeout=300)
    if result.returncode:
        raise ValueError('No se pudo abrir el selector. Se necesita Python Tk y una sesión gráfica (en WSL, WSLg).')
    value = json.loads(result.stdout)
    return Path(value) if value is not None else None


def dialog(initial, create):
    """Selecciona carpeta; al crear pide solo un nombre, nunca una ruta escrita."""
    import tkinter as tk
    from tkinter import filedialog, simpledialog, messagebox
    root = tk.Tk()
    root.withdraw()
    try:
        if create in {'vault-create','vault-open'}:
            method = filedialog.asksaveasfilename if create == 'vault-create' else filedialog.askopenfilename
            return method(parent=root, initialdir=str(initial), title='Proyecto cifrado ST', defaultextension='.stvault', filetypes=[('Proyecto cifrado ST','*.stvault')]) or None
        selected = filedialog.askdirectory(parent=root, initialdir=str(initial), mustexist=True,
                   title='Selecciona dónde crear el proyecto' if create else 'Selecciona el proyecto')
        if not selected:
            return None
        parent = Path(selected)
        if not create:
            return str(parent)
        while True:
            name = simpledialog.askstring('Crear proyecto ST', 'Nombre del proyecto:', parent=root)
            if name is None:
                return None
            if not re.fullmatch(r'[A-Za-z0-9_-]+', name):
                messagebox.showerror('Nombre inválido', 'Usa letras, números, guion o guion bajo.', parent=root)
            elif (parent / name).exists():
                messagebox.showerror('Ya existe', 'Elige otro nombre; no se sobrescriben carpetas.', parent=root)
            else:
                return str(parent / name)
    finally:
        root.destroy()


if __name__ == '__main__':
    print(json.dumps(dialog(Path(sys.argv[1]), sys.argv[2] if sys.argv[2].startswith('vault-') else sys.argv[2] == 'crear')))
