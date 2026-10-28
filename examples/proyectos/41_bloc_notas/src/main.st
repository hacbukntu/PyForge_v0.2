// Bloc de notas persistente
// Lee una nota desde la terminal, la guarda y verifica su contenido.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
mostrar("Escribe una nota:");
variable nota = io.leer_linea("");
io.escribir_texto("nota.txt", nota + "\n");
mostrar("Guardada: " + io.leer_texto("nota.txt"));
