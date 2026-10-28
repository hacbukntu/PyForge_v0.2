// Copia verificada de texto
// Copia un archivo UTF-8 y compara origen y destino.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
importar "os";
si no os.existe("original.txt") { lanzar "Falta original.txt"; }
variable contenido = io.leer_texto("original.txt");
io.escribir_texto("copia.txt", contenido);
mostrar("Copia correcta: " + (contenido == io.leer_texto("copia.txt")));
mostrar("Caracteres: " + longitud(contenido));
