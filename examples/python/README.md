# Ejemplos de Python en PyForge

Esta carpeta reúne ejemplos de compatibilidad con Python puro dentro del compilador, sin tocar la ruta de ST.

## Importante
No los ejecutes con `python archivo.py` ni con un binario `pyforgec` antiguo o desactualizado.
La ruta de Python del compilador se usa con el binario del proyecto, compilando el archivo como `.py`.

## Ejecución correcta

Desde la raíz del proyecto:

```bash
cargo run --bin pyforgec -- check examples/python/01_basico.py
cargo run --bin pyforgec -- build examples/python/01_basico.py -o dist/01_basico
cargo run --bin pyforgec -- run examples/python/01_basico.py
```

También puedes compilarlo una sola vez antes de usarlo:

```bash
cargo build --bin pyforgec
./target/debug/pyforgec check examples/python/03_control_flujo.py
./target/debug/pyforgec run examples/python/04_imports_for.py
```

## Incluye
- `01_basico.py`: listas, diccionarios, índices, `None` y `is not`
- `02_funciones.py`: funciones sin anotación, uso de `append()` y acceso por clave
- `03_control_flujo.py`: `if`, `for ... in range(...)` y `while`
- `04_imports_for.py`: `import`, `from ... import ...` y `for value in values`
- `05_pertenencia.py`: `in`, `not in` e `is not`
- `06_biblioteca.py`: `len`, `abs` y `sqrt`

Todos los ejemplos son compatibles con la ruta Python del compilador y no alteran la sintaxis ST.
