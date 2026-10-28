# 43. Diario con conservación de entradas

Añade una entrada conservando el contenido previo mediante lectura y reescritura.

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
Probar ST
```

Salida esperada:

```text
Nueva entrada:
Diario actualizado
```

## Archivos incluidos

- `diario.txt`: datos de entrada del caso de demostración.

## Archivos que genera

- `diario.txt`

Se crean o reemplazan en el directorio de ejecución. Conserva una copia si has añadido datos propios. El verificador comprueba su contenido exacto en una carpeta temporal.

## Personalizar

Modifica los datos iniciales, reglas o textos de `src/main.st`. Las anotaciones de variables son opcionales; los parámetros y retornos de funciones mantienen sus tipos. Si cambias el comportamiento, actualiza `pruebas/entrada.txt`, `pruebas/salida.txt` y las expectativas de archivos en `../catalogo.json`.

## Alcance

Reescritura de texto pequeño; sin bloqueo concurrente ni actualización atómica.

Las rutas y datos de muestra no requieren servicios externos. ST limita la memoria y el tamaño de lecturas; JSON admite documentos de hasta 4096 caracteres. Los errores no capturados terminan la ejecución con un diagnóstico.
