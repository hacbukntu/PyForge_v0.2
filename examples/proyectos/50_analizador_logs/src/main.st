// Resumen de registros de aplicación
// Cuenta mensajes por nivel a partir de prefijos de líneas.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
variable texto = io.leer_texto("app.log");
variable linea = "";
variable errores = 0;
variable avisos = 0;
para i en rango(0, longitud(texto)+1) {
    variable c = "\n";
    si i < longitud(texto) { c = texto[i]; }
    si c == "\n" {
        si buscar(linea,"ERROR ") == 0 { errores += 1; }
        si buscar(linea,"WARN ") == 0 { avisos += 1; }
        linea = "";
    } sino { linea += c; }
}
mostrar("Errores: " + errores); mostrar("Avisos: " + avisos);
