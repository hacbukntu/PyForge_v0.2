# Laberinto con paredes y salida

Juego. Proyecto didáctico completo para las capacidades actuales de ST; no es un producto certificado para producción.

## Qué resuelve

Explora un mapa con controles WASD, colisiones, salida voluntaria y límite de pasos. Listas de cadenas demuestran colecciones homogéneas e indexación Unicode sin acceder fuera del tablero.

## Ejecutar

Copia esta carpeta a tu espacio de trabajo. Desde esa copia:

```bash
pyforge run
```

En Windows utiliza WSL. Para un entorno Python propio con las dependencias necesarias:

```bash
PYFORGE_PYTHON="$PWD/.venv/bin/python" pyforge run
```

Esa ruta debe apuntar a un intérprete existente; crea el entorno y autoriza la instalación de dependencias primero. Consulta la [guía](../../../docs/INTEGRACIONES.md). La biblioteca nativa no requiere Python; los módulos de integración sí.

## Diseño y permisos

- Entrada: `src/main.st`. Las funciones y decisiones están comentadas en español.
- Módulos: `io`.
- Permisos: ninguno del auxiliar.
- Configuración: `PyForge.toml`; revisa permisos antes de ejecutar.
- Datos de prueba: `pruebas/entrada.txt`; comprobaciones: `pruebas/resultado.json`.
- Los recursos abiertos se cierran en `finalmente` y las salidas nuevas no sobrescriben documentos existentes.

## Pruebas

Desde el repositorio:

```bash
python3 scripts/verify-advanced-projects.py --project 13_laberinto_terminal
```

Modalidad: **local**. El verificador siempre compila y distingue ejecución, dependencia ausente y laboratorio pendiente; no presenta una omisión como éxito. Los juegos usan entradas reproducibles y conservan interacción manual. Todo se prueba en copias temporales.

## Cambiarlo

Edita los datos y funciones de `src/main.st`; conserva las validaciones y el cierre de recursos. Actualiza los resultados esperados si cambias las reglas. Para ampliar APIs consulta `src/stdlib` y `docs/INTEGRACIONES.md`.
