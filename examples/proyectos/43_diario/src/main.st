// Diario con conservación de entradas
// Añade una entrada conservando el contenido previo mediante lectura y reescritura.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "os";
variable previo = "";
si os.existe("diario.txt") { previo = io.leer_texto("diario.txt"); }
mostrar("Nueva entrada:");
variable nueva = entrada("");
io.escribir_texto("diario.txt", previo + "- " + nueva + "\n");
mostrar("Diario actualizado");
