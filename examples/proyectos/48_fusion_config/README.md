# 48. Combinar configuraciones conocidas

Combina un nombre base y un límite local, validando las claves esperadas.

## Cómo funciona

Lee `src/main.st` de arriba abajo: los comentarios explican los datos, las reglas y las funciones. El programa conserva las comprobaciones de tipos, índices y desbordamiento del compilador.

## Ejecutar

Abre una terminal **en esta carpeta** y ejecuta:

```bash
pyforge run
```

También puedes usar `pyforgec run src/main.st` o abrir esta carpeta con `pyforge-ide .`. Si estás trabajando desde el código fuente, sustituye el comando por la ruta absoluta de `target/release/pyforgec`.

## Caso de prueba

No necesita entrada por teclado.

Salida esperada:

```text
{"nombre":"ST","limite":50}
```

## Archivos incluidos

- `base.json`: datos de entrada del caso de demostración.
- `local.json`: datos de entrada del caso de demostración.

## Archivos que genera

- `resultado.json`

Se crean o reemplazan en el directorio de ejecución. Conserva una copia si has añadido datos propios. El verificador comprueba su contenido exacto en una carpeta temporal.

## Personalizar

Modifica los datos iniciales, reglas o textos de `src/main.st`. Las anotaciones de variables son opcionales; los parámetros y retornos de funciones mantienen sus tipos. Si cambias el comportamiento, actualiza `pruebas/entrada.txt`, `pruebas/salida.txt` y las expectativas de archivos en `../catalogo.json`.

## Alcance

Combinación de un esquema conocido; no implementa una mezcla recursiva genérica.

Las rutas y datos de muestra no requieren servicios externos. ST limita la memoria y el tamaño de lecturas; JSON admite documentos de hasta 4096 caracteres. Los errores no capturados terminan la ejecución con un diagnóstico.
