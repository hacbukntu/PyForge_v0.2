# 55. Serie de Fibonacci iterativa

Calcula una serie con memoria constante y limita el índice al rango i64.

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
8
```

Salida esperada:

```text
Cantidad de términos (1..90):
0
1
1
2
3
5
8
13
```

## Personalizar

Modifica los datos iniciales, reglas o textos de `src/main.st`. Las anotaciones de variables son opcionales; los parámetros y retornos de funciones mantienen sus tipos. Si cambias el comportamiento, actualiza `pruebas/entrada.txt`, `pruebas/salida.txt` y las expectativas de archivos en `../catalogo.json`.

## Alcance

Aplicación educativa de consola, con datos de ejemplo modificables.

Las rutas y datos de muestra no requieren servicios externos. ST limita la memoria y el tamaño de lecturas; JSON admite documentos de hasta 4096 caracteres. Los errores no capturados terminan la ejecución con un diagnóstico.
