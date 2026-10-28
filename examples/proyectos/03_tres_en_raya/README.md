# 03. Tres en raya para dos jugadores

Alterna jugadores, valida casillas y detecta filas, columnas y diagonales.

## Cómo funciona

Lee `src/main.st` de arriba abajo: los comentarios explican los datos, las reglas y las funciones. El programa conserva las comprobaciones de tipos, índices y desbordamiento del compilador.

## Ejecutar

Abre una terminal **en esta carpeta** y ejecuta:

```bash
pyforge run
```

También puedes usar `pyforgec run src/main.st` o abrir esta carpeta con `pyforge-ide .`. Si estás trabajando desde el código fuente, sustituye el comando por la ruta absoluta de `target/release/pyforgec`.

## Caso de prueba

Introduce estas líneas en orden, pulsando Enter después de cada una:

```text
0
3
1
4
2
```

Salida esperada:

```text
Dos jugadores: elige casilla 0..8, por filas.
1 0 0
0 0 0
0 0 0
1 0 0
2 0 0
0 0 0
1 1 0
2 0 0
0 0 0
1 1 0
2 2 0
0 0 0
1 1 1
2 2 0
0 0 0
Ganador: 1
```

## Personalizar

Modifica los datos iniciales, reglas o textos de `src/main.st`. Las anotaciones de variables son opcionales; los parámetros y retornos de funciones mantienen sus tipos. Si cambias el comportamiento, actualiza `pruebas/entrada.txt`, `pruebas/salida.txt` y las expectativas de archivos en `../catalogo.json`.

## Alcance

Aplicación educativa de consola, con datos de ejemplo modificables.

Las rutas y datos de muestra no requieren servicios externos. ST limita la memoria y el tamaño de lecturas; JSON admite documentos de hasta 4096 caracteres. Los errores no capturados terminan la ejecución con un diagnóstico.
