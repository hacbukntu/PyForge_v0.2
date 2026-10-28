// Lista de tareas
// Gestiona tareas pendientes y marca una como terminada mediante su índice.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable tareas = ["Responder correos","Preparar entrega","Revisar pruebas"];
variable hechas = [falso,falso,falso];
mostrar("Índice de tarea completada (0..2):");
variable indice = entero(entrada(""));
si indice < 0 o indice >= longitud(tareas) { lanzar "Tarea inexistente"; }
hechas[indice] = verdadero;
para i en rango(0, longitud(tareas)) {
    variable marca = "[ ] ";
    si hechas[i] { marca = "[x] "; }
    mostrar(marca + tareas[i]);
}
