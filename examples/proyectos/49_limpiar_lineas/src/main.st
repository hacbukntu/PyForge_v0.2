// Limpiar líneas vacías
// Normaliza CRLF y elimina líneas completamente vacías de un archivo.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
variable contenido = reemplazar(io.leer_texto("entrada.txt"),"\r","");
variable salida = "";
variable linea = "";
para i en rango(0, longitud(contenido)) {
    si contenido[i] == "\n" { si longitud(linea) > 0 { salida += linea + "\n"; } linea = ""; }
    sino { linea += contenido[i]; }
}
si longitud(linea) > 0 { salida += linea + "\n"; }
io.escribir_texto("limpio.txt",salida);
mostrar("Limpieza terminada");
