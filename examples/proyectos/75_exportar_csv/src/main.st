// Exportador CSV con comillas
// Escapa comillas y separadores al exportar nombres y cantidades.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
// Entrecomilla siempre los campos y duplica las comillas internas.
funcion campo(texto: cadena) -> cadena { retornar "\"" + reemplazar(texto,"\"","\"\"") + "\""; }
variable productos = ["Cable, USB","Libro \"ST\""];
variable cantidades = [3,2];
variable salida = "producto,cantidad\n";
para i en rango(0, longitud(productos)) { salida += campo(productos[i]) + "," + cantidades[i] + "\n"; }
io.escribir_texto("informe.csv",salida);
mostrar("CSV exportado");
